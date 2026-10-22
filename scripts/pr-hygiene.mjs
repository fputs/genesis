import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { changeReferences } from "./spec-drift-check.mjs";

const UNRELEASED = "## [Unreleased]";
const DATED_ARCHIVE = /^(\d{4})-(\d{2})-(\d{2})-(.+)$/;

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
  // Every pull_request action shares these checks. Reading the action honors
  // the override without a second code path for an edited body.
  void resolveAction(payload);

  const root = process.env.PR_HYGIENE_ROOT || defaultRoot();
  const baseSha = requiredSha("PR_HYGIENE_BASE_SHA", payload.pull_request?.base?.sha, "base");
  const headSha = requiredSha("PR_HYGIENE_HEAD_SHA", payload.pull_request?.head?.sha, "head");

  // The other side of every pull request diff is the merge-base, not the event
  // base sha. A commit that landed on main after the branch point is not this
  // pull request's change. The checkout HEAD may be a runner merge commit, so
  // archive-last reads the event head and that commit's first parent.
  // The body is read only to find this pull request's own change id.
  const mergeBaseSha = requireMergeBase(root, baseSha, headSha);
  const parentSha = requireParent(root, headSha);
  const paths = changedPaths(root, mergeBaseSha, headSha);

  const failures = [];
  if (missingBullet(root, mergeBaseSha, headSha, paths)) {
    failures.push(MISSING_BULLET);
  }
  // A draft is still being fixed, so archive-last waits until the pull request
  // is ready. The bullet check above still runs.
  if (!isDraft(payload)) {
    const removed = removedChangeIds(root, mergeBaseSha, headSha);
    const archiveFailed = archiveNotLast(root, mergeBaseSha, headSha, parentSha, removed);
    const ownActive = ownChangeStillActive(root, headSha, paths, pullRequestBody(payload));
    if (archiveFailed || ownActive) {
      failures.push(ARCHIVE_NOT_LAST);
    }
  }
  return failures;
}

function isDraft(payload) {
  return payload.pull_request?.draft === true;
}

function pullRequestBody(payload) {
  if (process.env.PR_HYGIENE_BODY !== undefined) {
    return process.env.PR_HYGIENE_BODY;
  }
  const body = payload.pull_request?.body;
  return typeof body === "string" ? body : "";
}

function ownChangeStillActive(root, headSha, paths, body) {
  if (!nonExemptImplementation(paths)) {
    return false;
  }
  const active = new Set(activeChangeIdsAt(root, headSha));
  const { ids } = changeReferences(unfencedBody(body));
  return ids.some((id) => active.has(id));
}

function nonExemptImplementation(paths) {
  return paths.some(isCodePath) && needsBullet(paths);
}

function unfencedBody(body) {
  // spec-drift counts a fenced OpenSpec-Change line. This own-change id does not.
  let fenced = false;
  const kept = [];
  for (const raw of body.split("\n")) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    if (isFence(line.trim())) {
      fenced = !fenced;
      continue;
    }
    if (!fenced) {
      kept.push(line);
    }
  }
  return kept.join("\n");
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

function gitResult(root, args) {
  // Git's missing-path phrases follow the locale. C keeps the English matchers.
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, LC_ALL: "C" },
  });
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

function isExemptPath(path) {
  return (
    path.startsWith("docs/") ||
    path.startsWith("openspec/") ||
    path.startsWith(".cursor/") ||
    path === "AGENTS.md" ||
    path === "CHANGELOG.md" ||
    path === "README.md" ||
    path === "Cargo.lock" ||
    path === "rust-toolchain.toml"
  );
}

function needsBullet(paths) {
  if (paths.length === 0 || paths.every(isExemptPath)) {
    return false;
  }
  return paths.some(isCodePath) || paths.some((path) => path.startsWith("scripts/"));
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
  const baseEmpty = baseLog.missing || !baseLog.sawHeading;
  const baseTexts = baseEmpty ? new Set() : new Set(baseLog.all);
  const baseCount = baseEmpty ? 0 : baseLog.unreleased.length;
  if (headLog.unreleased.length <= baseCount) {
    return true;
  }
  return !headLog.unreleased.some((text) => !baseTexts.has(text));
}

function readChangelog(root, sha) {
  const blob = blobAt(root, sha, "CHANGELOG.md");
  if (blob.missing) {
    return { missing: true, sawHeading: false, unreleased: [], all: [] };
  }
  return { missing: false, ...parseChangelog(blob.text) };
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

function parseChangelog(text) {
  let fenced = false;
  let inUnreleased = false;
  let sawHeading = false;
  const unreleased = [];
  const all = [];
  for (const raw of text.split("\n")) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    const trimmed = line.trim();
    if (isFence(trimmed)) {
      fenced = !fenced;
      continue;
    }
    if (fenced) {
      continue;
    }
    if (!sawHeading && trimmed === UNRELEASED) {
      inUnreleased = true;
      sawHeading = true;
      continue;
    }
    if (inUnreleased && trimmed.startsWith("## ")) {
      inUnreleased = false;
      continue;
    }
    const item = bulletText(trimmed);
    if (item === null) {
      continue;
    }
    all.push(item);
    if (inUnreleased) {
      unreleased.push(item);
    }
  }
  return { sawHeading, unreleased, all };
}

function isFence(trimmed) {
  return /^`{3,}$/.test(trimmed);
}

function bulletText(trimmed) {
  const match = /^([-*+]|\d+[.)])\s+(\S.*)$/.exec(trimmed);
  return match ? match[2].trim() : null;
}

function removedChangeIds(root, mergeBaseSha, headSha) {
  const onHead = new Set(activeChangeIdsAt(root, headSha));
  const removed = [];
  for (const id of activeChangeIdsAt(root, mergeBaseSha)) {
    if (!onHead.has(id)) {
      removed.push(id);
    }
  }
  removed.sort();
  return removed;
}

function activeChangeIdsAt(root, sha) {
  return childTrees(root, sha, "openspec/changes").filter((name) => name !== "archive");
}

function childTrees(root, sha, dirPath) {
  if (!treeExists(root, sha, dirPath)) {
    return [];
  }
  const result = gitResult(root, ["ls-tree", "-z", `${sha}:${dirPath}`]);
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "git ls-tree failed").trim();
    throw new Error(`pr-hygiene: ${detail}`);
  }
  const names = [];
  for (const field of (result.stdout ?? "").split("\0")) {
    if (field === "") {
      continue;
    }
    const tab = field.indexOf("\t");
    if (tab === -1) {
      throw new Error("pr-hygiene: ls-tree record could not be parsed");
    }
    const meta = field.slice(0, tab);
    const name = field.slice(tab + 1);
    const type = meta.split(" ")[1];
    if (type !== "tree") {
      continue;
    }
    if (!name || name.includes("/") || name.includes("\n")) {
      throw new Error("pr-hygiene: ls-tree record could not be parsed");
    }
    names.push(name);
  }
  return names;
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

function archiveNotLast(root, mergeBaseSha, headSha, parentSha, removedIds) {
  if (removedIds.length === 0) {
    return false;
  }
  if (commitSubject(root, headSha) !== expectedSubject(removedIds)) {
    return true;
  }
  const paths = changedPaths(root, parentSha, headSha);
  if (!paths.every((path) => isAllowedArchivePath(path, removedIds))) {
    return true;
  }
  for (const id of removedIds) {
    if (!headCommitDeletes(root, parentSha, headSha, id)) {
      return true;
    }
    if (!archiveSatisfied(root, mergeBaseSha, parentSha, headSha, id)) {
      return true;
    }
  }
  return false;
}

function expectedSubject(ids) {
  return ids
    .slice()
    .sort()
    .map((id) => `openspec archive ${id} --yes`)
    .join(" && ");
}

function commitSubject(root, sha) {
  const result = gitResult(root, ["log", "-1", "--format=%s", sha]);
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "git log failed").trim();
    throw new Error(`pr-hygiene: ${detail}`);
  }
  return (result.stdout ?? "").trim();
}

function headCommitDeletes(root, parentSha, headSha, id) {
  const dir = `openspec/changes/${id}`;
  return treeExists(root, parentSha, dir) && !treeExists(root, headSha, dir);
}

function archiveSatisfied(root, mergeBaseSha, parentSha, headSha, id) {
  const names = childTrees(root, headSha, "openspec/changes/archive").filter(
    (name) => datedRemainder(name) === id,
  );
  if (names.length === 0) {
    return false;
  }
  return names.some((name) => {
    const dir = `openspec/changes/archive/${name}`;
    if (treeExists(root, mergeBaseSha, dir)) {
      return true;
    }
    return !treeExists(root, parentSha, dir) && treeExists(root, headSha, dir);
  });
}

function datedRemainder(name) {
  const match = DATED_ARCHIVE.exec(name);
  return match ? match[4] : null;
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
