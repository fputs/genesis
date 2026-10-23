import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { readFileSync, realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { changeReferences, diffFromMergeBase, allPaths, isCodePath, qualifyingRefactorExemption } from "./spec-drift-check.mjs";
import { unfencedBody } from "./pr-hygiene.mjs";

const DATED_ARCHIVE = /^(\d{4})-(\d{2})-(\d{2})-(.+)$/;
const APPROVAL_LINE = /^spec-approved: [0-9a-f]{40}$/;
const APPROVAL_PREFIX = "spec-approved:";
const AUTHORS_PATH = "openspec/spec-approved-authors.txt";
const SCRIPT_PATH = "scripts/spec-approved.mjs";
const WORKFLOW_PATH = ".github/workflows/spec-approved.yml";

// These paths require a change line the way a code path does. Other scripts do not.
const APPROVAL_PATHS = new Set([
  "scripts/spec-approved.mjs",
  "scripts/spec-approved.test.mjs",
  "scripts/spec-drift-check.mjs",
  "scripts/pr-hygiene.mjs",
  "openspec/spec-approved-authors.txt",
]);

class CheckFailure extends Error {
  constructor(lines) {
    super(lines.join("\n"));
    this.lines = lines;
  }
}

async function main() {
  try {
    const result = await evaluate();
    for (const line of result.lines) {
      process.stdout.write(`${line}\n`);
    }
    process.exitCode = result.ok ? 0 : 1;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
    process.exitCode = 1;
  }
}

export async function evaluate() {
  const root = process.env.SPEC_APPROVED_ROOT || process.cwd();
  const payload = loadPayload();
  const base = requiredSha("SPEC_APPROVED_BASE_SHA", payload.pull_request?.base?.sha);
  const head = requiredSha("SPEC_APPROVED_HEAD_SHA", payload.pull_request?.head?.sha);
  if (!base || !head) {
    return fail(["spec-approved: pull request unavailable"]);
  }

  const hasScript = blobExists(root, base, SCRIPT_PATH);
  const hasWorkflow = blobExists(root, base, WORKFLOW_PATH);
  if (!hasScript && !hasWorkflow) {
    return pass(["bootstrap: no base script"]);
  }
  if (!hasScript || !hasWorkflow) {
    return fail(["spec-approved: partial base"]);
  }

  let records;
  try {
    const diff = diffFromMergeBase(root, base, head);
    if (diff.error === "no-merge-base") {
      return fail(["spec-approved: no merge-base"]);
    }
    records = diff.records;
  } catch {
    return fail(["spec-approved: no merge-base"]);
  }

  let authors;
  try {
    authors = loadAuthors(root, base);
  } catch (error) {
    if (error instanceof CheckFailure) {
      return fail(error.lines);
    }
    return fail(["spec-approved: authors unavailable"]);
  }

  const body = pullRequestBody(payload);
  const paths = allPaths(records);
  const { ids, invalidTokens } = changeReferences(body);
  if (invalidTokens.length > 0) {
    return fail(invalidTokens.map((token) => `spec-approved: invalid change id${escapeRemainder(token)}`));
  }

  const needsApproval = paths.some((path) => isCodePath(path) || APPROVAL_PATHS.has(path));
  if (!needsApproval || qualifyingRefactorExemption(body, records)) {
    return pass();
  }
  if (ids.length === 0) {
    return fail(["spec-approved: missing OpenSpec-Change"]);
  }

  for (const id of ids) {
    try {
      resolveChangeDir(root, head, id);
    } catch (error) {
      if (error instanceof CheckFailure) {
        return fail(error.lines);
      }
      return fail([`spec-approved: unknown change ${id}`]);
    }
  }

  let reviews;
  try {
    reviews = await loadReviews(payload);
  } catch (error) {
    if (error instanceof CheckFailure) {
      return fail(error.lines);
    }
    return fail(["spec-approved: reviews unavailable"]);
  }

  let latest;
  try {
    latest = pickLatest(reviews, authors);
  } catch (error) {
    if (error instanceof CheckFailure) {
      return fail(error.lines);
    }
    return fail(["spec-approved: reviews unavailable"]);
  }
  if (!latest) {
    return fail(["spec-approved: approval missing"]);
  }
  const attempts = approvalAttempts(latest.body);
  if (attempts.length === 0) {
    return fail(["spec-approved: approval missing"]);
  }
  if (attempts.length !== 1 || !APPROVAL_LINE.test(attempts[0])) {
    return fail(["spec-approved: approval malformed"]);
  }
  const approvedSha = attempts[0].slice(`${APPROVAL_PREFIX} `.length);
  const ancestor = gitText(root, ["merge-base", "--is-ancestor", approvedSha, head]);
  if (ancestor.status !== 0) {
    return fail(["spec-approved: approved sha unavailable"]);
  }

  try {
    for (const id of ids) {
      const atHead = digestChange(root, head, id);
      const atApproved = digestChange(root, approvedSha, id);
      if (atHead !== atApproved) {
        return fail(["spec-approved: spec hash mismatch"]);
      }
    }
  } catch (error) {
    if (error instanceof CheckFailure) {
      return fail(error.lines);
    }
    throw error;
  }
  return pass();
}

function pass(lines = []) {
  return { ok: true, lines };
}

function fail(lines) {
  return { ok: false, lines };
}

function requiredSha(envName, fallback) {
  if (process.env[envName] !== undefined) {
    const value = process.env[envName].trim();
    return value || null;
  }
  if (typeof fallback === "string" && fallback.trim()) {
    return fallback.trim();
  }
  return null;
}

function pullRequestBody(payload) {
  if (process.env.SPEC_APPROVED_BODY !== undefined) {
    return process.env.SPEC_APPROVED_BODY;
  }
  const body = payload.pull_request?.body;
  return typeof body === "string" ? body : "";
}

function loadPayload() {
  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath) {
    return {};
  }
  try {
    const parsed = JSON.parse(readFileSync(eventPath, "utf8"));
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return parsed;
    }
  } catch {
    return {};
  }
  return {};
}

function blobExists(root, sha, path) {
  return gitText(root, ["cat-file", "-e", `${sha}:${path}`]).status === 0;
}

function loadAuthors(root, sha) {
  const blob = readBlob(root, sha, AUTHORS_PATH);
  if (!blob || blob.kind !== "blob") {
    throw new CheckFailure(["spec-approved: authors unavailable"]);
  }
  const ids = new Set();
  for (const raw of blob.bytes.toString("utf8").split("\n")) {
    let line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    if (line.trim() === "") {
      continue;
    }
    if (/^\s*#/.test(line)) {
      continue;
    }
    const trimmed = line.trim();
    if (!/^[0-9]+$/.test(trimmed)) {
      throw new CheckFailure(["spec-approved: authors unavailable"]);
    }
    ids.add(Number(trimmed));
  }
  if (ids.size === 0) {
    throw new CheckFailure(["spec-approved: authors unavailable"]);
  }
  return ids;
}

function readBlob(root, sha, path) {
  const typed = gitText(root, ["cat-file", "-t", `${sha}:${path}`]);
  if (typed.status !== 0) {
    return null;
  }
  const kind = (typed.stdout ?? "").trim();
  if (kind !== "blob") {
    return { kind, bytes: Buffer.alloc(0) };
  }
  const data = gitBytes(root, ["cat-file", "blob", `${sha}:${path}`]);
  if (data.status !== 0) {
    return null;
  }
  return { kind, bytes: data.stdout };
}

function approvalAttempts(body) {
  const text = unfencedBody(typeof body === "string" ? body : "");
  const attempts = [];
  for (const raw of text.split("\n")) {
    let line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    line = line.trim();
    if (line.startsWith(APPROVAL_PREFIX)) {
      attempts.push(line);
    }
  }
  return attempts;
}

function pickLatest(reviews, authors) {
  if (!Array.isArray(reviews)) {
    throw new CheckFailure(["spec-approved: reviews unavailable"]);
  }
  const eligible = [];
  for (const review of reviews) {
    if (!review || typeof review !== "object" || Array.isArray(review)) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    if (review.state === "DISMISSED" || review.state === "PENDING") {
      continue;
    }
    const body = typeof review.body === "string" ? review.body : "";
    if (review.state === "COMMENTED" && body.trim() === "") {
      continue;
    }
    const userId = numericId(review.user?.id);
    if (!authors.has(userId)) {
      continue;
    }
    if (typeof review.submitted_at !== "string" || Number.isNaN(Date.parse(review.submitted_at))) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    eligible.push(review);
  }
  eligible.sort((left, right) => {
    const submitted = Date.parse(right.submitted_at) - Date.parse(left.submitted_at);
    if (submitted !== 0) {
      return submitted;
    }
    return Number(BigInt(right.id) - BigInt(left.id));
  });
  return eligible[0] ?? null;
}

function numericId(value) {
  if (typeof value === "number" && Number.isInteger(value)) {
    return value;
  }
  if (typeof value === "string" && /^[0-9]+$/.test(value)) {
    return Number(value);
  }
  return Number.NaN;
}

async function loadReviews(payload) {
  if (process.env.SPEC_APPROVED_REVIEW_PAGES !== undefined) {
    let pages;
    try {
      pages = JSON.parse(process.env.SPEC_APPROVED_REVIEW_PAGES);
    } catch {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    return readInjectedPages(pages);
  }
  return fetchReviewPages(payload);
}

function readInjectedPages(pages) {
  if (!Array.isArray(pages)) {
    throw new CheckFailure(["spec-approved: reviews unavailable"]);
  }
  const reviews = [];
  let index = 0;
  const seen = new Set();
  while (true) {
    if (seen.has(index)) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    seen.add(index);
    const page = pages[index];
    if (!page || page.status !== 200 || !Array.isArray(page.reviews)) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    reviews.push(...page.reviews);
    if (page.next === null || page.next === undefined) {
      return reviews;
    }
    if (typeof page.next !== "number") {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    index = page.next;
  }
}

async function fetchReviewPages(payload) {
  const token = process.env.GITHUB_TOKEN;
  const repository = process.env.GITHUB_REPOSITORY;
  const number = payload.pull_request?.number ?? process.env.SPEC_APPROVED_PR_NUMBER;
  if (!token || !repository || number === undefined || number === null || number === "") {
    throw new CheckFailure(["spec-approved: reviews unavailable"]);
  }
  const [owner, repo] = String(repository).split("/");
  if (!owner || !repo) {
    throw new CheckFailure(["spec-approved: reviews unavailable"]);
  }
  let url = `https://api.github.com/repos/${owner}/${repo}/pulls/${number}/reviews?per_page=100`;
  const reviews = [];
  const seen = new Set();
  while (url) {
    if (seen.has(url)) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    seen.add(url);
    let response;
    try {
      response = await fetch(url, {
        headers: {
          Accept: "application/vnd.github+json",
          Authorization: `Bearer ${token}`,
          "User-Agent": "spec-approved",
          "X-GitHub-Api-Version": "2022-11-28",
        },
      });
    } catch {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    if (!response.ok) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    const page = await response.json();
    if (!Array.isArray(page)) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    reviews.push(...page);
    const next = nextLink(response.headers.get("link"));
    if (next && next !== url) {
      url = next;
      continue;
    }
    if (next === url) {
      throw new CheckFailure(["spec-approved: reviews unavailable"]);
    }
    url = null;
  }
  return reviews;
}

function nextLink(header) {
  if (!header) {
    return null;
  }
  for (const part of header.split(",")) {
    const match = part.match(/<([^>]+)>\s*;\s*rel="?next"?/);
    if (match) {
      return match[1];
    }
  }
  return null;
}

function resolveChangeDir(root, sha, id) {
  if (id === "archive") {
    throw new CheckFailure([`spec-approved: unknown change ${id}`]);
  }
  const active = `openspec/changes/${id}`;
  const activeExists = gitType(root, sha, active) === "tree";
  const archives = archiveMatches(root, sha, id);
  if ((activeExists && archives.length > 0) || archives.length > 1) {
    throw new CheckFailure(["spec-approved: ambiguous change path"]);
  }
  if (activeExists) {
    return active;
  }
  if (archives.length === 1) {
    return archives[0];
  }
  throw new CheckFailure([`spec-approved: unknown change ${id}`]);
}

function archiveMatches(root, sha, id) {
  if (gitType(root, sha, "openspec/changes/archive") !== "tree") {
    return [];
  }
  const matches = [];
  for (const entry of listTree(root, sha, "openspec/changes/archive", false)) {
    if (entry.type !== "tree") {
      continue;
    }
    const name = entry.path;
    const dated = DATED_ARCHIVE.exec(name);
    if (dated) {
      if (dated[4] === id) {
        matches.push(`openspec/changes/archive/${name}`);
      }
      continue;
    }
    if (name === id) {
      matches.push(`openspec/changes/archive/${name}`);
    }
  }
  return matches;
}

export function digestChange(root, sha, id) {
  const dir = resolveChangeDir(root, sha, id);
  const files = [];
  for (const entry of listTree(root, sha, dir, true)) {
    const relative = entry.path;
    if (relative !== "proposal.md" && relative !== "design.md" && !relative.startsWith("specs/")) {
      continue;
    }
    if (entry.type !== "blob") {
      throw new CheckFailure(["spec-approved: hashed path is not a blob"]);
    }
    files.push({ path: relative, bytes: readGitBlob(root, entry.object) });
  }
  if (!files.some((file) => file.path === "proposal.md")) {
    throw new CheckFailure(["spec-approved: proposal missing"]);
  }
  return specDigest(files);
}

function listTree(root, sha, dir, recursive) {
  const args = ["ls-tree", "-z"];
  if (recursive) {
    args.push("-r");
  }
  // SHA:dir lists names relative to that directory. A path argument would keep the repo prefix.
  args.push(`${sha}:${dir}`);
  const result = gitText(root, args);
  if (result.status !== 0) {
    throw new CheckFailure(["spec-approved: proposal missing"]);
  }
  const entries = [];
  for (const field of (result.stdout ?? "").split("\0")) {
    if (field === "") {
      continue;
    }
    const tab = field.indexOf("\t");
    if (tab === -1) {
      throw new CheckFailure(["spec-approved: proposal missing"]);
    }
    const meta = field.slice(0, tab).split(" ");
    entries.push({
      mode: meta[0],
      type: meta[1],
      object: meta[2],
      path: field.slice(tab + 1),
    });
  }
  return entries;
}

function readGitBlob(root, object) {
  const result = gitBytes(root, ["cat-file", "blob", object]);
  if (result.status !== 0 || !Buffer.isBuffer(result.stdout)) {
    throw new CheckFailure(["spec-approved: proposal missing"]);
  }
  return result.stdout;
}

export function specDigest(entries) {
  const sorted = [...entries].sort((left, right) =>
    Buffer.compare(Buffer.from(left.path), Buffer.from(right.path)),
  );
  const hash = createHash("sha256");
  for (const entry of sorted) {
    const pathBytes = Buffer.from(entry.path);
    const bytes = Buffer.isBuffer(entry.bytes) ? entry.bytes : Buffer.from(entry.bytes);
    const length = Buffer.alloc(8);
    length.writeBigUInt64BE(BigInt(bytes.length));
    hash.update(pathBytes);
    hash.update(Buffer.from([0]));
    hash.update(length);
    hash.update(bytes);
  }
  return hash.digest("hex");
}

function gitType(root, sha, path) {
  const result = gitText(root, ["cat-file", "-t", `${sha}:${path}`]);
  if (result.status !== 0) {
    return null;
  }
  return (result.stdout ?? "").trim();
}

function gitEnv() {
  return { ...process.env, LC_ALL: "C" };
}

function gitText(root, args) {
  return spawnSync("git", args, { cwd: root, encoding: "utf8", env: gitEnv() });
}

function gitBytes(root, args) {
  return spawnSync("git", args, { cwd: root, env: gitEnv() });
}

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

function runningAsScript() {
  const entry = process.argv[1];
  if (!entry) {
    return false;
  }
  try {
    return realpathSync(entry) === realpathSync(fileURLToPath(import.meta.url));
  } catch {
    return false;
  }
}

if (runningAsScript()) {
  main();
}
