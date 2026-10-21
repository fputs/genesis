import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { mkdtemp, mkdir, rm, writeFile, chmod } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./spec-drift-check.mjs", import.meta.url));
const workflowPath = fileURLToPath(
  new URL("../.github/workflows/ci.yml", import.meta.url),
);
const editedWorkflowPath = fileURLToPath(
  new URL("../.github/workflows/spec-drift-edited.yml", import.meta.url),
);
const workflow = readFileSync(workflowPath, "utf8");
const editedWorkflow = readFileSync(editedWorkflowPath, "utf8");
const realGit = spawnSync("which", ["git"], { encoding: "utf8" }).stdout.trim();

const STUB_LINE = "stub-openspec: validate --all --strict";
const OPEN = "- [ ] 1.1 Open\n";
const DONE = "- [x] 1.1 Done\n";
const MISSING = "spec-drift: code change missing OpenSpec-Change";

function completed(id) {
  return `spec-drift: completed unarchived change ${id}`;
}

function unknown(id) {
  return `spec-drift: unknown change ${id}`;
}

function passing(name, fields = {}) {
  return {
    event: "push",
    validateStatus: 0,
    excludes: ["spec-drift:"],
    base: {},
    ...fields,
    name,
    status: 0,
    includes: fields.includes ?? [],
  };
}

function failing(name, lines, fields = {}) {
  return {
    event: "push",
    validateStatus: 0,
    excludes: [],
    base: {},
    ...fields,
    name,
    status: 1,
    includes: lines,
  };
}

const cases = [
  failing("A missing reference needs no network lookup", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "crates/rules/src/lib.rs": "old\n" },
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("Invalid tree fails validation", ["spec-drift: validate failed"], {
    validateStatus: 1,
  }),
  passing("Valid tree passes validation"),
  passing("Archive children are not active", {
    base: {
      "openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md": DONE,
    },
    excludes: ["spec-drift:", completed("add-project-foundation")],
  }),
  failing("A sibling of archive is active", [completed("add-dice-and-checks")], {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": DONE,
      "openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md": DONE,
    },
  }),
  failing("A titled box counts", [completed("add-dice-and-checks")], {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [x] 1.1 Record the decision\n",
    },
  }),
  failing("Space around the x still counts as checked", [completed("add-dice-and-checks")], {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [ x] 1.1 Done\n",
    },
  }),
  passing("An open box is not fully checked", {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [x] 1.1 Done\n- [ ] 1.2 Left\n",
    },
  }),
  passing("No checkboxes is not fully checked", {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md":
        "This change has prose and no task checkbox.\n",
    },
  }),
  passing("A missing tasks file is not fully checked", {
    base: {
      "openspec/changes/add-dice-and-checks/proposal.md": "# Proposal\n",
    },
  }),
  passing("Other list markers are not checkboxes", {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "* [x] done\n1. [x] done\n",
    },
  }),
  passing("A checkbox needs whitespace before the title", {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [x]title\n",
    },
  }),
  passing("A non-x marker is unchecked", {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [~] 1.1 Done\n",
    },
  }),
  passing("An empty box is unchecked", {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [] 1.1 Done\n",
    },
  }),
  failing(
    "A pull request that leaves every box checked fails",
    [completed("add-dice-and-checks")],
    {
      event: "pull_request",
      body: "",
      base: { "openspec/changes/add-dice-and-checks/tasks.md": OPEN },
      head: { "openspec/changes/add-dice-and-checks/tasks.md": DONE },
    },
  ),
  failing("Push backstop fails that same tree", [completed("add-dice-and-checks")], {
    base: { "openspec/changes/add-dice-and-checks/tasks.md": DONE },
  }),
  passing("An open task passes the push", {
    base: { "openspec/changes/add-dice-and-checks/tasks.md": "- [ ] 1.1 Left\n" },
  }),
  passing("An archived completed change passes the push", {
    base: {
      "openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md": DONE,
    },
  }),
  failing("One of two active changes is complete", [completed("add-dice-and-checks")], {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": DONE,
      "openspec/changes/add-spec-drift-check/tasks.md": OPEN,
    },
    excludes: [completed("add-spec-drift-check")],
  }),
  failing("Uppercase checked boxes fail the push", [completed("add-dice-and-checks")], {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "- [X] 1.1 Done\n",
    },
  }),
  failing("Indented boxes fail the push", [completed("add-dice-and-checks")], {
    base: {
      "openspec/changes/add-dice-and-checks/tasks.md": "  - [x] 1.1 Done\n",
    },
  }),
  failing(
    "A later pull request still has the completed change",
    [completed("add-dice-and-checks")],
    {
      event: "pull_request",
      body: "",
      base: { "openspec/changes/add-dice-and-checks/tasks.md": DONE },
    },
  ),
  failing(
    "Copying to archive without removing the active change fails",
    [completed("add-dice-and-checks")],
    {
      event: "pull_request",
      body: "",
      base: {
        "openspec/changes/add-dice-and-checks/tasks.md": DONE,
        "openspec/changes/archive/2026-10-21-add-dice-and-checks/tasks.md": DONE,
      },
    },
  ),
  failing(
    "Checked tasks still active fails the pull request",
    [completed("add-dice-and-checks")],
    {
      event: "pull_request",
      body: "",
      base: { "openspec/changes/add-dice-and-checks/tasks.md": OPEN },
      head: { "openspec/changes/add-dice-and-checks/tasks.md": DONE },
    },
  ),
  failing("A new fully checked active change fails", [completed("add-dice-and-checks")], {
    event: "pull_request",
    body: "",
    head: { "openspec/changes/add-dice-and-checks/tasks.md": DONE },
  }),
  passing("The last commit is the archive", {
    event: "pull_request",
    body: "",
    commitMessage: "openspec archive add-dice-and-checks --yes",
    base: { "openspec/changes/add-dice-and-checks/tasks.md": OPEN },
    remove: ["openspec/changes/add-dice-and-checks"],
    head: {
      "openspec/changes/archive/2026-10-21-add-dice-and-checks/tasks.md": DONE,
    },
    assertRepo(dir) {
      assert.equal(existsSync(join(dir, "openspec/changes/add-dice-and-checks")), false);
      const tasks = readFileSync(
        join(dir, "openspec/changes/archive/2026-10-21-add-dice-and-checks/tasks.md"),
        "utf8",
      );
      assert.match(tasks, /^- \[x] 1\.1 Done$/m);
    },
  }),
  passing("The archived result passes the pull request", {
    event: "pull_request",
    body: "",
    commitMessage: "openspec archive add-dice-and-checks --yes",
    base: { "openspec/changes/add-dice-and-checks/tasks.md": OPEN },
    remove: ["openspec/changes/add-dice-and-checks"],
    head: {
      "openspec/changes/archive/2026-10-21-add-dice-and-checks/tasks.md": DONE,
    },
  }),
  passing("The following push stays green", {
    commitMessage: "openspec archive add-dice-and-checks --yes",
    base: { "openspec/changes/add-dice-and-checks/tasks.md": OPEN },
    remove: ["openspec/changes/add-dice-and-checks"],
    head: {
      "openspec/changes/archive/2026-10-21-add-dice-and-checks/tasks.md": DONE,
    },
    excludes: ["spec-drift: completed unarchived change"],
  }),
  failing("A crate file is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "crates/rules/src/lib.rs": "old\n" },
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("A non-ASCII crate path is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "crates/rules/src/naïve.rs": "old\n" },
    head: { "crates/rules/src/naïve.rs": "new\n" },
  }),
  failing("Root Cargo.toml is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "Cargo.toml": "old\n" },
    head: { "Cargo.toml": "new\n" },
  }),
  failing("Cargo.lock is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "Cargo.lock": "old\n" },
    head: { "Cargo.lock": "new\n" },
  }),
  failing("rust-toolchain.toml is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "rust-toolchain.toml": "old\n" },
    head: { "rust-toolchain.toml": "new\n" },
  }),
  failing("A workflow file is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { ".github/workflows/ci.yml": "old\n" },
    head: { ".github/workflows/ci.yml": "new\n" },
  }),
  failing("A quoted .github path is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { '.github/workflows/ci".yml': "old\n" },
    head: { '.github/workflows/ci".yml': "new\n" },
  }),
  failing("Deleting a crate file is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "crates/rules/src/lib.rs": "old\n" },
    remove: ["crates/rules/src/lib.rs"],
  }),
  failing("Renaming into crates is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "docs/note.md": "same\n" },
    rename: [{ from: "docs/note.md", to: "crates/rules/note.md" }],
  }),
  failing("Renaming out of crates is a code path", [MISSING], {
    event: "pull_request",
    body: "",
    base: { "crates/rules/src/lib.rs": "same\n" },
    rename: [{ from: "crates/rules/src/lib.rs", to: "docs/lib.rs" }],
  }),
  passing("A nested Cargo.toml is not a root manifest", {
    event: "pull_request",
    body: "",
    base: { "docs/Cargo.toml": "old\n" },
    head: { "docs/Cargo.toml": "new\n" },
  }),
  passing("A crates prefix without a slash is not a code path", {
    event: "pull_request",
    body: "",
    base: { "crates-extra/foo.rs": "old\n" },
    head: { "crates-extra/foo.rs": "new\n" },
  }),
  passing("Files outside the diff do not count", {
    event: "pull_request",
    body: "",
    base: {
      "README.md": "old\n",
      "crates/rules/src/lib.rs": "fn x() {}\n",
    },
    head: { "README.md": "new\n" },
  }),
  passing("A trimmed line is a reference", {
    event: "pull_request",
    body: "  OpenSpec-Change: add-dice-and-checks  ",
    base: {
      "Cargo.toml": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: { "Cargo.toml": "new\n" },
  }),
  failing("Extra words are not a reference", [MISSING], {
    event: "pull_request",
    body: "OpenSpec-Change: add-dice-and-checks extra",
    base: {
      "Cargo.toml": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: { "Cargo.toml": "new\n" },
    excludes: ["spec-drift: unknown change", "spec-drift: invalid change id"],
  }),
  failing("An uppercase id is not skipped", ["spec-drift: invalid change id Add-Foo"], {
    event: "pull_request",
    body: "OpenSpec-Change: Add-Foo",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/tasks.md": OPEN,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
    excludes: [MISSING],
  }),
  failing(
    "A valid id does not hide an invalid token",
    ["spec-drift: invalid change id Add-Foo"],
    {
      event: "pull_request",
      body: "OpenSpec-Change: add-dice-and-checks\nOpenSpec-Change: Add-Foo",
      base: {
        "Cargo.toml": "old\n",
        "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
      },
      head: { "Cargo.toml": "new\n" },
      excludes: [MISSING],
    },
  ),
  failing(
    "docs-only PR with OpenSpec-Change: Add-Foo fails",
    ["spec-drift: invalid change id Add-Foo"],
    {
      event: "pull_request",
      body: "OpenSpec-Change: Add-Foo",
      base: {
        "docs/architect/decisions.md": "old\n",
        "openspec/changes/add-foo/tasks.md": OPEN,
      },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
    },
  ),
  failing(
    "An empty or spaceless change line is invalid",
    ["spec-drift: invalid change id\n", "spec-drift: invalid change id Add"],
    {
      event: "pull_request",
      body: "OpenSpec-Change:\nOpenSpec-Change:Add",
      base: { "docs/architect/decisions.md": "old\n" },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
    },
  ),
  passing("A scripts-only pull request is not a code path", {
    event: "pull_request",
    body: "",
    base: { "scripts/note.mjs": "old\n" },
    head: { "scripts/note.mjs": "new\n" },
    excludes: [MISSING],
  }),
  passing("Git diff runs with LC_ALL=C", {
    event: "pull_request",
    body: "",
    base: { "docs/note.md": "old\n" },
    head: { "docs/note.md": "new\n" },
    recordGitLocale: true,
  }),
  passing("An active id exists", {
    event: "pull_request",
    body: "OpenSpec-Change: add-dice-and-checks",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  passing("A dated archive id exists", {
    event: "pull_request",
    body: "OpenSpec-Change: add-project-foundation",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md": DONE,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  passing("An undated archive directory exists", {
    event: "pull_request",
    body: "OpenSpec-Change: add-project-foundation",
    base: {
      "Cargo.lock": "old\n",
      "openspec/changes/archive/add-project-foundation/tasks.md": DONE,
    },
    head: { "Cargo.lock": "new\n" },
  }),
  failing("An unknown id fails", [unknown("missing-change")], {
    event: "pull_request",
    body: "OpenSpec-Change: missing-change",
    base: { "rust-toolchain.toml": "old\n" },
    head: { "rust-toolchain.toml": "new\n" },
    excludes: [MISSING],
  }),
  failing("A partial archive suffix does not match", [unknown("foundation")], {
    event: "pull_request",
    body: "OpenSpec-Change: foundation",
    base: {
      ".github/workflows/ci.yml": "old\n",
      "openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md": DONE,
    },
    head: { ".github/workflows/ci.yml": "new\n" },
  }),
  passing("Two existing ids pass", {
    event: "pull_request",
    body: "OpenSpec-Change: add-dice-and-checks\nOpenSpec-Change: add-project-foundation",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
      "openspec/changes/archive/2026-10-21-add-project-foundation/tasks.md": DONE,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("One of two ids is unknown", [unknown("missing-change")], {
    event: "pull_request",
    body: "OpenSpec-Change: add-dice-and-checks\nOpenSpec-Change: missing-change",
    base: {
      "Cargo.toml": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: { "Cargo.toml": "new\n" },
    excludes: [MISSING],
  }),
  passing("A duplicate id counts once", {
    event: "pull_request",
    body: "OpenSpec-Change: add-dice-and-checks\nOpenSpec-Change: add-dice-and-checks",
    base: {
      "Cargo.toml": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: { "Cargo.toml": "new\n" },
  }),
  passing("Docs-only pull request passes without a reference", {
    event: "pull_request",
    body: "",
    base: {
      "docs/architect/decisions.md": "old\n",
      "README.md": "old\n",
    },
    head: {
      "docs/architect/decisions.md": "new\n",
      "README.md": "new\n",
    },
  }),
  passing("Openspec-only pull request passes without a reference", {
    event: "pull_request",
    body: "",
    base: { "openspec/changes/add-spec-drift-check/tasks.md": OPEN },
    head: { "openspec/changes/add-spec-drift-check/tasks.md": "- [ ] 1.1 Still open\n" },
  }),
  passing("An exempt pull request ignores an unknown id", {
    event: "pull_request",
    body: "OpenSpec-Change: missing-change",
    base: { "README.md": "old\n" },
    head: { "README.md": "new\n" },
  }),
  failing("Docs plus a code path still requires a reference", [MISSING], {
    event: "pull_request",
    body: "",
    base: {
      "README.md": "old\n",
      "crates/rules/src/lib.rs": "old\n",
    },
    head: {
      "README.md": "new\n",
      "crates/rules/src/lib.rs": "new\n",
    },
  }),
  failing("Openspec plus a workflow file still requires a reference", [MISSING], {
    event: "pull_request",
    body: "",
    base: {
      "openspec/changes/add-spec-drift-check/tasks.md": OPEN,
      ".github/workflows/ci.yml": "old\n",
    },
    head: {
      "openspec/changes/add-spec-drift-check/tasks.md": "- [ ] 1.1 Still open\n",
      ".github/workflows/ci.yml": "new\n",
    },
  }),
  passing("Docs plus a code path passes with an existing id", {
    event: "pull_request",
    body: "OpenSpec-Change: add-dice-and-checks",
    base: {
      "docs/architect/decisions.md": "old\n",
      "Cargo.lock": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: {
      "docs/architect/decisions.md": "new\n",
      "Cargo.lock": "new\n",
    },
  }),
  passing("A code push has no pull request body", {
    base: { "crates/rules/src/lib.rs": "old\n" },
    head: { "crates/rules/src/lib.rs": "new\n" },
    excludes: ["spec-drift: code change missing OpenSpec-Change"],
  }),
  passing("Archived implementation pull request passes", {
    event: "pull_request",
    body: "OpenSpec-Change: add-spec-drift-check",
    commitMessage: "openspec archive add-spec-drift-check --yes",
    base: {
      ".github/workflows/ci.yml": "name: CI\n",
      "openspec/changes/add-spec-drift-check/tasks.md": OPEN,
    },
    remove: ["openspec/changes/add-spec-drift-check"],
    head: {
      ".github/workflows/ci.yml": "name: CI\njobs:\n",
      "openspec/changes/archive/2026-10-21-add-spec-drift-check/tasks.md": DONE,
    },
  }),
  passing("Push of that merge stays green", {
    commitMessage: "openspec archive add-spec-drift-check --yes",
    base: {
      ".github/workflows/ci.yml": "name: CI\n",
      "openspec/changes/add-spec-drift-check/tasks.md": OPEN,
    },
    remove: ["openspec/changes/add-spec-drift-check"],
    head: {
      ".github/workflows/ci.yml": "name: CI\njobs:\n",
      "openspec/changes/archive/2026-10-21-add-spec-drift-check/tasks.md": DONE,
    },
    excludes: ["spec-drift: completed unarchived change"],
  }),
  failing(
    "Validation and a completed change both fail",
    ["spec-drift: validate failed", completed("add-dice-and-checks")],
    {
      validateStatus: 1,
      base: { "openspec/changes/add-dice-and-checks/tasks.md": DONE },
    },
  ),
  passing("A clean tree passes", {
    event: "pull_request",
    body: "",
    base: { "openspec/changes/add-spec-drift-check/proposal.md": "old\n" },
    head: { "openspec/changes/add-spec-drift-check/proposal.md": "new\n" },
  }),
];

for (const row of cases) {
  test(row.name, async () => {
    await runCase(row);
  });
}

test("Job triggers match the Cargo workflow", () => {
  const header = workflow.slice(0, workflow.indexOf("jobs:"));
  assert.match(header, /push:/);
  assert.match(header, /pull_request:/);
  assert.match(header, /branches:\s*\[main]/);
  assert.doesNotMatch(workflow, /edited/);
  assert.doesNotMatch(workflow, /\bif:/);
  assert.match(jobBlock(workflow, "ci"), /cargo fmt --check/);
  const job = jobBlock(workflow, "spec-drift");
  assert.match(job, /fetch-depth:\s*0/);
  assert.match(job, /node scripts\/spec-drift-check\.mjs/);
  assert.match(editedWorkflow, /pull_request:/);
  assert.match(editedWorkflow, /types:\s*\[edited]/);
  assert.match(editedWorkflow, /branches:\s*\[main]/);
  assert.match(editedWorkflow, /node scripts\/spec-drift-check\.mjs/);
  assert.doesNotMatch(editedWorkflow, /node --test/);
  assert.doesNotMatch(editedWorkflow, /cargo /);
  const editedJobs = editedWorkflow
    .slice(editedWorkflow.indexOf("jobs:"))
    .match(/^ {2}[a-z0-9-]+:$/gm);
  assert.deepEqual(editedJobs, ["  spec-drift:"]);
});

test("Editing the body does not replace a failed Cargo check", () => {
  assert.match(workflow, /pull_request:\n\s+branches: \[main]/);
  assert.doesNotMatch(workflow, /edited/);
  assert.doesNotMatch(workflow, /\bif:/);
  assert.doesNotMatch(workflow, /types:/);
  assert.match(editedWorkflow, /types:\s*\[edited]/);
  assert.match(editedWorkflow, /branches:\s*\[main]/);
  assert.match(editedWorkflow, /node scripts\/spec-drift-check\.mjs/);
  assert.doesNotMatch(editedWorkflow, /cargo fmt/);
  assert.doesNotMatch(editedWorkflow, /^ {2}ci:/m);
});

test("The edited workflow reuses the spec-drift job name", () => {
  assert.match(editedWorkflow, /pull_request:/);
  assert.match(editedWorkflow, /types:\s*\[edited]/);
  assert.match(editedWorkflow, /branches:\s*\[main]/);
  assert.match(editedWorkflow, /permissions:\n  contents: read\n/);
  assert.match(
    editedWorkflow,
    /group: spec-drift-edited-\$\{\{ github\.event\.pull_request\.number \}\}/,
  );
  assert.match(editedWorkflow, /cancel-in-progress:\s*true/);
  const jobs = editedWorkflow
    .slice(editedWorkflow.indexOf("jobs:"))
    .match(/^ {2}[a-z0-9-]+:$/gm);
  assert.deepEqual(jobs, ["  spec-drift:"]);
  const job = jobBlock(editedWorkflow, "spec-drift");
  assert.match(job, /node scripts\/spec-drift-check\.mjs/);
  assert.doesNotMatch(job, /node --test/);
  assert.doesNotMatch(editedWorkflow, /pr-hygiene/);
});

test("Dropping the change line on edit fails", async () => {
  const fields = {
    event: "pull_request",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
  };
  await runCase(
    passing("kept the line", {
      ...fields,
      body: "OpenSpec-Change: add-dice-and-checks",
    }),
  );
  await runCase(failing("edited drops the line", [MISSING], { ...fields, body: "" }));
});

test("Cargo steps stay", () => {
  const job = jobBlock(workflow, "ci");
  assert.match(job, /cargo fmt --check/);
  assert.match(job, /cargo clippy --workspace --all-targets -- -D warnings/);
  assert.match(job, /cargo build --workspace/);
  assert.match(job, /cargo test --workspace/);
});

test("Workflow pins the CLI", () => {
  const job = jobBlock(workflow, "spec-drift");
  const pins = job.match(/@fission-ai\/openspec@\S+/g) ?? [];
  assert.deepEqual(pins, ["@fission-ai/openspec@1.14.3"]);
  assert.doesNotMatch(job, /@latest/);
  assert.doesNotMatch(job, /@fission-ai\/openspec@(?:\^|~|>=|\*)/);
  assert.match(job, /node-version:\s*["']?22["']?/);
});

test("The job runs the suite first", () => {
  const job = jobBlock(workflow, "spec-drift");
  const suite = job.indexOf("node --test scripts/spec-drift-check.test.mjs");
  const script = job.indexOf("node scripts/spec-drift-check.mjs");
  assert.ok(suite >= 0);
  assert.ok(script > suite);
  assert.doesNotMatch(job, /continue-on-error/);
});

test("Each case is a fixture repository", async () => {
  assert.ok(cases.length > 0);
  const names = cases.map((row) => row.name);
  assert.equal(new Set(names).size, names.length);
  for (const row of cases) {
    assert.equal(typeof row.status, "number");
    assert.ok(Array.isArray(row.includes));
    await runCase(row);
  }
});

function jobBlock(yaml, id) {
  const lines = yaml.split("\n");
  const start = lines.findIndex((line) => line === `  ${id}:`);
  assert.ok(start >= 0, `missing job ${id}`);
  let end = lines.length;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (/^ {2}[A-Za-z0-9_-]+:$/.test(lines[index])) {
      end = index;
      break;
    }
  }
  return lines.slice(start, end).join("\n");
}

async function runCase(row) {
  const dir = await mkdtemp(join(tmpdir(), "spec-drift-"));
  try {
    git(dir, ["init", "-b", "main"]);
    git(dir, ["config", "user.email", "spec-drift@example.com"]);
    git(dir, ["config", "user.name", "Spec Drift"]);
    git(dir, ["config", "commit.gpgsign", "false"]);
    git(dir, ["config", "core.hooksPath", "/dev/null"]);

    const baseFiles = { "README.md": "base\n", ...(row.base ?? {}) };
    await writeTree(dir, baseFiles);
    git(dir, ["add", "-A"]);
    git(dir, ["commit", "-m", "base"]);
    const baseSha = git(dir, ["rev-parse", "HEAD"]).stdout.trim();

    const renames = row.rename ?? [];
    const removals = row.remove ?? [];
    const hasHead = row.head !== undefined || removals.length > 0 || renames.length > 0;
    if (hasHead) {
      for (const rel of removals) {
        await rm(join(dir, rel), { recursive: true, force: true });
      }
      for (const { from, to } of renames) {
        await mkdir(dirname(join(dir, to)), { recursive: true });
        git(dir, ["mv", from, to]);
      }
      if (row.head) {
        await writeTree(dir, row.head);
      }
      git(dir, ["add", "-A"]);
      git(dir, ["commit", "-m", row.commitMessage ?? "head"]);
    }

    if (row.commitMessage) {
      const subject = git(dir, ["log", "-1", "--format=%s"]).stdout.trim();
      assert.equal(subject, row.commitMessage);
    }

    assert.equal(git(dir, ["rev-parse", "--is-inside-work-tree"]).stdout.trim(), "true");

    const bin = join(dir, ".stub-bin");
    await mkdir(bin);
    const stub = join(bin, "openspec");
    const code = row.validateStatus ?? 0;
    await writeFile(stub, `#!/bin/sh\necho "${STUB_LINE}"\nexit ${code}\n`);
    await chmod(stub, 0o755);
    const localeLog = join(dir, "git-locale.txt");
    if (row.recordGitLocale) {
      const gitStub = join(bin, "git");
      await writeFile(
        gitStub,
        "#!/bin/sh\nif [ \"$1\" = \"diff\" ]; then\n  printf '%s\\n' \"$LC_ALL\" > \"$GIT_LOCALE_LOG\"\nfi\nexec \"$REAL_GIT\" \"$@\"\n",
      );
      await chmod(gitStub, 0o755);
    }

    const env = {
      PATH: `${bin}${delimiter}${process.env.PATH ?? ""}`,
      HOME: process.env.HOME ?? "",
      LANG: process.env.LANG ?? "C",
      SPEC_DRIFT_EVENT: row.event,
      SPEC_DRIFT_ROOT: dir,
      SPEC_DRIFT_BASE_SHA: baseSha,
    };
    if (row.recordGitLocale) {
      env.LC_ALL = "fr_FR.UTF-8";
      env.GIT_LOCALE_LOG = localeLog;
      env.REAL_GIT = realGit;
    }
    if (row.body !== undefined) {
      env.SPEC_DRIFT_BODY = row.body;
    } else if (row.event === "pull_request") {
      env.SPEC_DRIFT_BODY = "";
    }

    const result = spawnSync(process.execPath, [scriptPath], {
      cwd: dir,
      env,
      encoding: "utf8",
    });
    const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
    assert.equal(result.status, row.status, output);
    for (const line of [STUB_LINE, ...(row.includes ?? [])]) {
      assert.ok(output.includes(line), `missing ${line}\n${output}`);
    }
    for (const line of row.excludes ?? []) {
      assert.ok(!output.includes(line), `unexpected ${line}\n${output}`);
    }
    if (row.recordGitLocale) {
      assert.equal(readFileSync(localeLog, "utf8").trim(), "C");
    }
    if (row.assertRepo) {
      row.assertRepo(dir);
    }
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

function git(cwd, args) {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: "Spec Drift",
      GIT_AUTHOR_EMAIL: "spec-drift@example.com",
      GIT_COMMITTER_NAME: "Spec Drift",
      GIT_COMMITTER_EMAIL: "spec-drift@example.com",
    },
  });
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed\n${result.stderr ?? ""}\n${result.stdout ?? ""}`);
  }
  return result;
}

async function writeTree(root, files) {
  for (const [rel, contents] of Object.entries(files)) {
    const full = join(root, rel);
    await mkdir(dirname(full), { recursive: true });
    await writeFile(full, contents);
  }
}
