import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
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
      failures.push(...referenceFailures(root, body, baseSha));
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

function referenceFailures(root, body, baseSha) {
  if (!baseSha) {
    throw new Error("pull request base sha is required for the code-path diff");
  }
  const paths = changedPaths(root, baseSha);
  const { ids, invalidTokens } = changeReferences(body);
  // An invalid token is a mistake on every pull request, including docs-only.
  // It must not also be reported as a missing reference.
  const failures = invalidTokens.map((token) => invalidChangeId(token));
  if (!paths.some(isCodePath)) {
    return failures;
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

function changedPaths(root, baseSha) {
  const result = spawnSync(
    "git",
    ["diff", "-z", "--name-status", "--find-renames", baseSha, "HEAD"],
    { cwd: root, encoding: "utf8", env: gitEnv() },
  );
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "git diff failed").trim();
    throw new Error(detail);
  }
  // quotePath would escape non-ASCII and quotes, and those code paths would miss the reference check.
  const fields = (result.stdout ?? "").split("\0");
  const paths = [];
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
      if (from) {
        paths.push(from);
      }
      if (to) {
        paths.push(to);
      }
      index += 3;
      continue;
    }
    const path = fields[index + 1];
    if (path) {
      paths.push(path);
    }
    index += 2;
  }
  return paths;
}

function isCodePath(path) {
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

function changeReferences(body) {
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

main();
