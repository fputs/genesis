import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const CHANGE_ID = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const UNRELEASED = "## [Unreleased]";

// Failure lines are the spec's whole contract for a red log. Keep them stable.
const MISSING_BULLET = "implementation PR missing CHANGELOG bullet";
const ARCHIVE_NOT_LAST = "archive commit is not last";

function main() {
  try {
    const failures = evaluate();
    for (const line of failures) {
      process.stdout.write(`${line}\n`);
    }
    process.exitCode = failures.length > 0 ? 1 : 0;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
    process.exitCode = 1;
  }
}

function evaluate() {
  const payload = loadPayload();
  const event = process.env.PR_HYGIENE_EVENT || process.env.GITHUB_EVENT_NAME || "";
  if (event !== "pull_request") {
    throw new Error("pr-hygiene: pull request event is required");
  }
  // opened, synchronize, reopened, and edited share these checks. Reading the
  // action honors the override without a second code path for an edited body.
  void resolveAction(payload);

  const root = process.env.PR_HYGIENE_ROOT || defaultRoot();
  const baseSha = requiredSha("PR_HYGIENE_BASE_SHA", payload.pull_request?.base?.sha, "base");
  const headSha = requiredSha("PR_HYGIENE_HEAD_SHA", payload.pull_request?.head?.sha, "head");
  const body = resolveBody(payload);

  // The other side of every pull request diff is the merge-base, not the event
  // base sha. A commit that landed on main after the branch point is not this
  // pull request's change. The checkout HEAD may be a runner merge commit, so
  // archive-last reads the event head and that commit's first parent.
  const mergeBaseSha = requireMergeBase(root, baseSha, headSha);
  const parentSha = requireParent(root, headSha);
  const paths = changedPaths(root, mergeBaseSha, headSha);

  const failures = [];
  if (missingBullet(root, mergeBaseSha, headSha, paths)) {
    failures.push(MISSING_BULLET);
  }
  const removed = removedChangeIds(root, mergeBaseSha, headSha, changeReferences(body));
  if (archiveNotLast(root, headSha, parentSha, removed)) {
    failures.push(ARCHIVE_NOT_LAST);
  }
  return failures;
}

function defaultRoot() {
  return join(dirname(fileURLToPath(import.meta.url)), "..");
}

function loadPayload() {
  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath) {
    return {};
  }
  let parsed;
  try {
    parsed = JSON.parse(readFileSync(eventPath, "utf8"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`pr-hygiene: event payload could not be parsed: ${message}`);
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("pr-hygiene: event payload could not be parsed");
  }
  return parsed;
}

function resolveAction(payload) {
  if (process.env.PR_HYGIENE_ACTION !== undefined) {
    return process.env.PR_HYGIENE_ACTION;
  }
  return typeof payload.action === "string" ? payload.action : "";
}

function requiredSha(envName, fallback, label) {
  if (process.env[envName] !== undefined) {
    const value = process.env[envName].trim();
    if (!value) {
      throw new Error(`pr-hygiene: ${label} sha is required`);
    }
    return value;
  }
  if (typeof fallback === "string" && fallback.trim()) {
    return fallback.trim();
  }
  throw new Error(`pr-hygiene: ${label} sha is required`);
}

function resolveBody(payload) {
  if (process.env.PR_HYGIENE_BODY !== undefined) {
    return process.env.PR_HYGIENE_BODY;
  }
  const body = payload.pull_request?.body;
  return typeof body === "string" ? body : "";
}

function gitResult(root, args) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  if (result.error) {
    throw new Error(`pr-hygiene: ${result.error.message}`);
  }
  return result;
}

function requireMergeBase(root, baseSha, headSha) {
  const result = gitResult(root, ["merge-base", baseSha, headSha]);
  const sha = (result.stdout ?? "").trim();
  if (result.status !== 0 || !sha) {
    const detail = (result.stderr || result.stdout || "no merge-base").trim();
    throw new Error(`pr-hygiene: no merge-base: ${detail}`);
  }
  return sha;
}

function requireParent(root, headSha) {
  const result = gitResult(root, ["rev-parse", "--verify", "--quiet", `${headSha}^1`]);
  const sha = (result.stdout ?? "").trim();
  if (result.status !== 0 || !sha) {
    throw new Error("pr-hygiene: head commit has no parent");
  }
  return sha;
}

function changedPaths(root, from, to) {
  const result = gitResult(root, [
    "diff",
    "-z",
    "--name-status",
    "--find-renames",
    from,
    to,
  ]);
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "git diff failed").trim();
    throw new Error(`pr-hygiene: ${detail}`);
  }
  // quotePath would escape non-ASCII and quotes, and those paths would miss classification.
  return parseNulDiff(result.stdout ?? "");
}

function parseNulDiff(text) {
  const fields = text.split("\0");
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
      if (!from || !to) {
        throw new Error("pr-hygiene: NUL record could not be parsed");
      }
      paths.push(from, to);
      index += 3;
      continue;
    }
    const path = fields[index + 1];
    if (!path) {
      throw new Error("pr-hygiene: NUL record could not be parsed");
    }
    paths.push(path);
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

function isDocsPath(path) {
  return path === "AGENTS.md" || path === "CHANGELOG.md" || path.startsWith("docs/");
}

function isExemptPath(path) {
  return (
    path.startsWith(".cursor/") ||
    path === "README.md" ||
    path === "Cargo.lock" ||
    path === "rust-toolchain.toml"
  );
}

function isExemptDiff(paths) {
  return (
    paths.every(isDocsPath) ||
    paths.every((path) => path.startsWith("openspec/")) ||
    paths.every(isExemptPath)
  );
}

function needsBullet(paths) {
  return paths.some(isCodePath) && !isExemptDiff(paths);
}

function missingBullet(root, mergeBaseSha, headSha, paths) {
  if (!needsBullet(paths)) {
    return false;
  }
  const headLog = readChangelog(root, headSha);
  if (headLog.missing || !headLog.sawHeading) {
    return true;
  }
  const baseLog = readChangelog(root, mergeBaseSha);
  const baseBullets =
    baseLog.missing || !baseLog.sawHeading ? new Set() : baseLog.bullets;
  for (const bullet of headLog.bullets) {
    if (!baseBullets.has(bullet)) {
      return false;
    }
  }
  return true;
}

function readChangelog(root, sha) {
  const blob = blobAt(root, sha, "CHANGELOG.md");
  if (blob.missing) {
    return { missing: true, sawHeading: false, bullets: new Set() };
  }
  return { missing: false, ...unreleasedBullets(blob.text) };
}

function blobAt(root, sha, filePath) {
  const result = gitResult(root, ["show", `${sha}:${filePath}`]);
  if (result.status === 0) {
    return { missing: false, text: result.stdout ?? "" };
  }
  const detail = result.stderr ?? "";
  if (detail.includes("does not exist") || detail.includes("exists on disk, but not in")) {
    return { missing: true, text: "" };
  }
  throw new Error(`pr-hygiene: ${detail.trim() || "git show failed"}`);
}

function unreleasedBullets(text) {
  let inSection = false;
  let sawHeading = false;
  const bullets = new Set();
  for (const raw of text.split("\n")) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    const trimmed = line.trim();
    if (!inSection) {
      if (trimmed === UNRELEASED) {
        inSection = true;
        sawHeading = true;
      }
      continue;
    }
    if (trimmed.startsWith("## ")) {
      break;
    }
    if (isListItem(trimmed)) {
      bullets.add(trimmed);
    }
  }
  return { sawHeading, bullets };
}

function isListItem(trimmed) {
  return /^[-*+] \S/.test(trimmed) || /^\d+[.)] \S/.test(trimmed);
}

function changeReferences(body) {
  const ids = [];
  const seen = new Set();
  for (const line of body.split("\n")) {
    let text = line;
    if (text.endsWith("\r")) {
      text = text.slice(0, -1);
    }
    const match = /^OpenSpec-Change: ([a-z0-9]+(?:-[a-z0-9]+)*)$/.exec(text.trim());
    if (!match || !CHANGE_ID.test(match[1]) || seen.has(match[1])) {
      continue;
    }
    seen.add(match[1]);
    ids.push(match[1]);
  }
  return ids;
}

function removedChangeIds(root, mergeBaseSha, headSha, ids) {
  const removed = [];
  for (const id of ids) {
    const onBase = treeExists(root, mergeBaseSha, `openspec/changes/${id}`);
    const onHead = treeExists(root, headSha, `openspec/changes/${id}`);
    if (onBase && !onHead) {
      removed.push(id);
    }
  }
  return removed;
}

function treeExists(root, sha, dirPath) {
  const result = gitResult(root, ["cat-file", "-t", `${sha}:${dirPath}`]);
  if (result.status === 0) {
    return (result.stdout ?? "").trim() === "tree";
  }
  const detail = result.stderr ?? "";
  if (detail.includes("does not exist") || detail.includes("exists on disk, but not in")) {
    return false;
  }
  throw new Error(`pr-hygiene: ${detail.trim() || "git cat-file failed"}`);
}

function archiveNotLast(root, headSha, parentSha, removedIds) {
  if (removedIds.length === 0) {
    return false;
  }
  const message = commitMessage(root, headSha);
  const paths = changedPaths(root, parentSha, headSha);
  const messageOk = removedIds.every((id) => archivesId(message, id));
  const pathsOk = paths.every((path) => isAllowedArchivePath(path, removedIds));
  return !messageOk || !pathsOk;
}

function commitMessage(root, sha) {
  const result = gitResult(root, ["log", "-1", "--format=%B", sha]);
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "git log failed").trim();
    throw new Error(`pr-hygiene: ${detail}`);
  }
  return result.stdout ?? "";
}

function archivesId(message, id) {
  const needle = `openspec archive ${id}`;
  let from = 0;
  while (from < message.length) {
    const at = message.indexOf(needle, from);
    if (at === -1) {
      return false;
    }
    const next = message[at + needle.length];
    if (next === undefined || /\s/.test(next)) {
      return true;
    }
    from = at + 1;
  }
  return false;
}

function isAllowedArchivePath(filePath, ids) {
  if (
    filePath.startsWith("openspec/changes/archive/") ||
    filePath.startsWith("openspec/specs/")
  ) {
    return true;
  }
  return ids.some((id) => {
    const dir = `openspec/changes/${id}`;
    return filePath === dir || filePath.startsWith(`${dir}/`);
  });
}

main();
