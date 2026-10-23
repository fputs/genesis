import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const CHANGE_ID = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const CHECKBOX_LINE = /^\s*- \[(.*?)](?=$|\s)/;
const CHECKED_TEXT = /^\s*[xX]\s*$/;
const DATED_ARCHIVE = /^(\d{4})-(\d{2})-(\d{2})-(.+)$/;

// Failure lines are the spec's whole contract for a red log. Keep them stable.
const VALIDATE_FAILED = "spec-drift: validate failed";
const MISSING_REFERENCE = "spec-drift: code change missing OpenSpec-Change";
const REFERENCE_PREFIX = "OpenSpec-Change:";
const REFACTOR_PREFIX = "OpenSpec-Refactor:";
const MULTIPLE_REFACTOR = "spec-drift: multiple OpenSpec-Refactor lines";
const REFACTOR_CONFLICT = "spec-drift: refactor line conflicts with OpenSpec-Change";
const NOT_CRATES_ONLY = "spec-drift: refactor is not crates-only";
const CHANGES_OPENSPEC = "spec-drift: refactor changes openspec";
const CHANGES_TESTS = "spec-drift: refactor changes existing tests";

function main() {
  const root = process.env.SPEC_DRIFT_ROOT || defaultRoot();
  const event = process.env.SPEC_DRIFT_EVENT || process.env.GITHUB_EVENT_NAME || "";
  const payload = loadEventPayload();
  const body =
    process.env.SPEC_DRIFT_BODY !== undefined
      ? process.env.SPEC_DRIFT_BODY
      : bodyFromPayload(payload);
  const baseSha = process.env.SPEC_DRIFT_BASE_SHA || baseShaFromPayload(payload);

  const failures = [];

  if (runValidation(root) !== 0) {
    failures.push(VALIDATE_FAILED);
  }

  failures.push(...completedChangeFailures(root));

  let referenceError = false;
  if (event === "pull_request") {
    try {
      failures.push(...referenceFailures(root, body, baseSha, payload));
    } catch (error) {
      referenceError = true;
      const message = error instanceof Error ? error.message : String(error);
      process.stderr.write(`${message}\n`);
    }
  }

  for (const line of failures) {
    process.stdout.write(`${line}\n`);
  }

  process.exitCode = failures.length > 0 || referenceError ? 1 : 0;
}

function defaultRoot() {
  return join(dirname(fileURLToPath(import.meta.url)), "..");
}

function loadEventPayload() {
  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath) {
    return {};
  }
  try {
    const parsed = JSON.parse(readFileSync(eventPath, "utf8"));
    if (parsed && typeof parsed === "object") {
      return parsed;
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
  }
  return {};
}

function bodyFromPayload(payload) {
  const body = payload.pull_request?.body;
  return typeof body === "string" ? body : "";
}

function baseShaFromPayload(payload) {
  const sha = payload.pull_request?.base?.sha;
  return typeof sha === "string" ? sha : "";
}

function runValidation(root) {
  const result = spawnSync("openspec", ["validate", "--all", "--strict"], {
    cwd: root,
    encoding: "utf8",
  });
  writeForwarded(process.stdout, result.stdout);
  writeForwarded(process.stderr, result.stderr);
  if (result.error) {
    process.stderr.write(`${result.error.message}\n`);
    return 1;
  }
  return result.status === 0 ? 0 : 1;
}

function writeForwarded(stream, text) {
  if (!text) {
    return;
  }
  stream.write(text);
  if (!text.endsWith("\n")) {
    stream.write("\n");
  }
}

function completedChangeFailures(root) {
  const failures = [];
  for (const id of activeChangeIds(root)) {
    if (isFullyChecked(join(root, "openspec", "changes", id, "tasks.md"))) {
      failures.push(`spec-drift: completed unarchived change ${id}`);
    }
  }
  return failures;
}

function activeChangeIds(root) {
  const changesDir = join(root, "openspec", "changes");
  if (!isDirectory(changesDir)) {
    return [];
  }
  const ids = [];
  for (const name of readdirSync(changesDir)) {
    if (name === "archive") {
      continue;
    }
    if (isDirectory(join(changesDir, name))) {
      ids.push(name);
    }
  }
  // localeCompare follows the process locale, so the failure list would change between runners.
  ids.sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  return ids;
}

function isFullyChecked(tasksPath) {
  if (!existsSync(tasksPath) || !statSync(tasksPath).isFile()) {
    return false;
  }
  const text = readFileSync(tasksPath, "utf8");
  let tracked = 0;
  for (const line of text.split("\n")) {
    const box = checkboxText(line);
    if (box === null) {
      continue;
    }
    tracked += 1;
    if (!CHECKED_TEXT.test(box)) {
      return false;
    }
  }
  return tracked > 0;
}

function checkboxText(line) {
  let text = line;
  if (text.endsWith("\r")) {
    text = text.slice(0, -1);
  }
  const match = CHECKBOX_LINE.exec(text);
  return match ? match[1] : null;
}

function referenceFailures(root, body, baseSha, payload) {
  if (!baseSha) {
    throw new Error("pull request base sha is required for the code-path diff");
  }
  const headSha = resolveHeadSha(root, payload);
  const diff = diffFromMergeBase(root, baseSha, headSha);
  if (diff.error === "no-merge-base") {
    throw new Error("spec-drift: no merge-base");
  }
  const records = diff.records;
  const paths = allPaths(records);
  const { ids, invalidTokens } = changeReferences(body);
  const refactor = collectRefactor(body);
  // Invalid tokens and malformed refactor items fail on every pull request,
  // including docs-only. They must not also be reported as a missing reference.
  const failures = invalidTokens.map((token) => invalidChangeId(token));
  for (const remainder of refactor.invalid) {
    failures.push(invalidRefactorItem(remainder));
  }
  if (refactor.lines.length > 1) {
    failures.push(MULTIPLE_REFACTOR);
  }
  const hasChangeLine = invalidTokens.length > 0 || ids.length > 0;
  if (refactor.lines.length > 0 && hasChangeLine) {
    failures.push(REFACTOR_CONFLICT);
  }
  const tokenFailure =
    refactor.invalid.length > 0 ||
    refactor.lines.length > 1 ||
    (refactor.lines.length > 0 && hasChangeLine);
  if (!paths.some(isCodePath)) {
    return failures;
  }
  if (tokenFailure) {
    for (const id of ids) {
      if (!changeExists(root, id)) {
        failures.push(`spec-drift: unknown change ${id}`);
      }
    }
    return failures;
  }
  if (refactor.lines.length === 1 && refactor.invalid.length === 0 && !hasChangeLine) {
    return refactorPathFailures(records);
  }
  if (invalidTokens.length === 0 && ids.length === 0) {
    return [MISSING_REFERENCE];
  }
  for (const id of ids) {
    if (!changeExists(root, id)) {
      failures.push(`spec-drift: unknown change ${id}`);
    }
  }
  return failures;
}

function resolveHeadSha(root, payload) {
  if (process.env.SPEC_DRIFT_HEAD_SHA !== undefined && process.env.SPEC_DRIFT_HEAD_SHA.trim()) {
    return process.env.SPEC_DRIFT_HEAD_SHA.trim();
  }
  const fromPayload = payload?.pull_request?.head?.sha;
  if (typeof fromPayload === "string" && fromPayload.trim()) {
    return fromPayload.trim();
  }
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
    env: gitEnv(),
  });
  if (result.status !== 0 || !(result.stdout ?? "").trim()) {
    throw new Error("spec-drift: no merge-base");
  }
  return result.stdout.trim();
}

export function diffFromMergeBase(root, baseSha, headSha) {
  const merge = spawnSync("git", ["merge-base", baseSha, headSha], {
    cwd: root,
    encoding: "utf8",
    env: gitEnv(),
  });
  const mergeBase = (merge.stdout ?? "").trim();
  if (merge.status !== 0 || !mergeBase) {
    return { error: "no-merge-base", records: [] };
  }
  const result = spawnSync(
    "git",
    ["diff", "-z", "--name-status", "--find-renames", mergeBase, headSha],
    { cwd: root, encoding: "utf8", env: gitEnv() },
  );
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "git diff failed").trim();
    throw new Error(detail);
  }
  return { records: parseStatusRecords(result.stdout ?? "") };
}

export function allPaths(records) {
  const paths = [];
  for (const record of records) {
    if (record.path) {
      paths.push(record.path);
    }
    if (record.from) {
      paths.push(record.from);
    }
    if (record.to) {
      paths.push(record.to);
    }
  }
  return paths;
}

export function qualifyingRefactorExemption(body, records) {
  const refactor = collectRefactor(body);
  const { ids, invalidTokens } = changeReferences(body);
  if (refactor.lines.length !== 1 || refactor.invalid.length !== 0) {
    return false;
  }
  if (ids.length !== 0 || invalidTokens.length !== 0) {
    return false;
  }
  const paths = allPaths(records);
  if (!paths.every((path) => path.startsWith("crates/") || path === "CHANGELOG.md")) {
    return false;
  }
  if (records.some(changesExistingTest)) {
    return false;
  }
  return true;
}

function collectRefactor(body) {
  const lines = [];
  const invalid = [];
  const invalidSeen = new Set();
  for (const raw of body.split("\n")) {
    let text = raw;
    if (text.endsWith("\r")) {
      text = text.slice(0, -1);
    }
    text = text.trim();
    if (!text.startsWith(REFACTOR_PREFIX)) {
      continue;
    }
    const remainder = text.slice(REFACTOR_PREFIX.length);
    const item = refactorItem(remainder);
    lines.push(remainder);
    if (item === null && !invalidSeen.has(remainder)) {
      invalidSeen.add(remainder);
      invalid.push(remainder);
    }
  }
  return { lines, invalid };
}

function refactorItem(remainder) {
  if (remainder.charCodeAt(0) !== 0x20) {
    return null;
  }
  const item = remainder.slice(1);
  if (!CHANGE_ID.test(item)) {
    return null;
  }
  return item;
}

function invalidRefactorItem(remainder) {
  return `spec-drift: invalid refactor item${escapeRemainder(remainder)}`;
}

function refactorPathFailures(records) {
  const paths = allPaths(records);
  const failures = [];
  if (paths.some((path) => !path.startsWith("crates/") && path !== "CHANGELOG.md")) {
    failures.push(NOT_CRATES_ONLY);
  }
  if (paths.some((path) => path.startsWith("openspec/"))) {
    failures.push(CHANGES_OPENSPEC);
  }
  if (records.some(changesExistingTest)) {
    failures.push(CHANGES_TESTS);
  }
  return failures;
}

function isTestFile(path) {
  return path.startsWith("crates/") && path.includes("/tests/");
}

function changesExistingTest(record) {
  if (record.status.startsWith("R")) {
    return isTestFile(record.from ?? "");
  }
  return (
    (record.status === "M" || record.status === "D" || record.status === "T") &&
    isTestFile(record.path ?? "")
  );
}

function parseStatusRecords(text) {
  const fields = text.split("\0");
  const records = [];
  let index = 0;
  while (index < fields.length) {
    const status = fields[index];
    if (status === "") {
      index += 1;
      continue;
    }
    if (status.startsWith("R") || status.startsWith("C")) {
      const from = fields[index + 1];
      const to = fields[index + 2];
      if (from || to) {
        records.push({ status, from, to });
      }
      index += 3;
      continue;
    }
    const path = fields[index + 1];
    if (path) {
      records.push({ status, path });
    }
    index += 2;
  }
  return records;
}

function invalidChangeId(remainder) {
  // JSON.stringify leaves U+200B raw, quotes some whitespace, and hides a typed backslash.
  return `spec-drift: invalid change id${escapeRemainder(remainder)}`;
}

// Each UTF-16 code unit outside printable ASCII, and a backslash, is visible as \uXXXX.
function escapeRemainder(text) {
  let escaped = "";
  for (let index = 0; index < text.length; index += 1) {
    const unit = text.charCodeAt(index);
    if (unit < 0x20 || unit > 0x7e || unit === 0x5c) {
      escaped += `\\u${unit.toString(16).padStart(4, "0")}`;
    } else {
      escaped += text.charAt(index);
    }
  }
  return escaped;
}

export function isCodePath(path) {
  return (
    path === "Cargo.toml" ||
    path === "Cargo.lock" ||
    path === "rust-toolchain.toml" ||
    path.startsWith("crates/") ||
    path.startsWith(".github/workflows/")
  );
}

function gitEnv() {
  // Git's own error text follows the process locale. The C locale keeps it stable.
  return { ...process.env, LC_ALL: "C" };
}

export function changeReferences(body) {
  const ids = [];
  const seen = new Set();
  const invalidTokens = [];
  const invalidSeen = new Set();
  for (const line of body.split("\n")) {
    let text = line;
    if (text.endsWith("\r")) {
      text = text.slice(0, -1);
    }
    text = text.trim();
    if (!text.startsWith(REFERENCE_PREFIX)) {
      continue;
    }
    const remainder = text.slice(REFERENCE_PREFIX.length);
    const id = referenceId(remainder);
    if (id === null) {
      // A prefix line that is not one kebab id is a mistake on every pull request.
      if (!invalidSeen.has(remainder)) {
        invalidSeen.add(remainder);
        invalidTokens.push(remainder);
      }
      continue;
    }
    if (!seen.has(id)) {
      seen.add(id);
      ids.push(id);
    }
  }
  return { ids, invalidTokens };
}

// Valid only as exactly one U+0020 and one kebab id. The remainder is otherwise logged whole.
function referenceId(remainder) {
  if (remainder.charCodeAt(0) !== 0x20) {
    return null;
  }
  const id = remainder.slice(1);
  if (!CHANGE_ID.test(id)) {
    return null;
  }
  return id;
}

function changeExists(root, id) {
  if (id === "archive") {
    return false;
  }
  if (isDirectory(join(root, "openspec", "changes", id))) {
    return true;
  }
  const archiveDir = join(root, "openspec", "changes", "archive");
  if (!isDirectory(archiveDir)) {
    return false;
  }
  for (const name of readdirSync(archiveDir)) {
    if (!isDirectory(join(archiveDir, name))) {
      continue;
    }
    const dated = DATED_ARCHIVE.exec(name);
    if (dated) {
      // The folder name is storage. Only the remainder after YYYY-MM-DD- is an id.
      if (dated[4] === id) {
        return true;
      }
      continue;
    }
    if (name === id) {
      return true;
    }
  }
  return false;
}

function isDirectory(path) {
  if (!existsSync(path)) {
    return false;
  }
  return statSync(path).isDirectory();
}

function runningAsScript() {
  const entry = process.argv[1];
  if (!entry) {
    return false;
  }
  try {
    // pr-hygiene imports changeReferences. Loading this file must not validate.
    return realpathSync(entry) === realpathSync(fileURLToPath(import.meta.url));
  } catch {
    return false;
  }
}

if (runningAsScript()) {
  main();
}
