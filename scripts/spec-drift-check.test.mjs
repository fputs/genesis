import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { mkdtemp, mkdir, rm, writeFile, chmod, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { changeReferences } from "./spec-drift-check.mjs";

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

const SPEC_DRIFT_GROUP =
  "spec-drift-${{ github.event.pull_request.number || github.sha }}";
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

function shownInvalid(token) {
  return `spec-drift: invalid change id ${escapeRemainder(token)}`;
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

function whitespaceReferenceCases() {
  const tokens = [
    ["a tab", "\tadd-foo"],
    ["a nbsp", "\u00a0add-foo"],
    ["an em space", "\u2003add-foo"],
    ["a tabbed Add-Foo", "\tAdd-Foo"],
  ];
  const docs = {
    event: "pull_request",
    base: {
      "docs/architect/decisions.md": "old\n",
      "openspec/changes/add-foo/tasks.md": OPEN,
    },
    head: { "docs/architect/decisions.md": "new\n" },
    excludes: [MISSING],
  };
  const code = {
    event: "pull_request",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/tasks.md": OPEN,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
    excludes: [MISSING],
  };
  const rows = [];
  for (const [label, token] of tokens) {
    const body = `OpenSpec-Change: ${token}`;
    const line = shownInvalid(token);
    rows.push(
      failing(`Docs-only PR with ${label} before the id is invalid`, [line], { ...docs, body }),
    );
    rows.push(failing(`Code PR with ${label} before the id is invalid`, [line], { ...code, body }));
  }
  const hidden = shownInvalid("\tadd-foo");
  const mixed = "OpenSpec-Change: add-dice-and-checks\nOpenSpec-Change: \tadd-foo";
  rows.push(
    failing("A valid id does not hide a whitespace token", [hidden], {
      event: "pull_request",
      body: mixed,
      base: {
        "Cargo.toml": "old\n",
        "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
        "openspec/changes/add-foo/tasks.md": OPEN,
      },
      head: { "Cargo.toml": "new\n" },
      excludes: [MISSING],
    }),
  );
  rows.push(
    failing("Docs-only valid id does not hide a whitespace token", [hidden], {
      event: "pull_request",
      body: mixed,
      base: {
        "docs/architect/decisions.md": "old\n",
        "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
        "openspec/changes/add-foo/tasks.md": OPEN,
      },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
    }),
  );
  return rows;
}

const LIB = "crates/rules/src/lib.rs";
const DOCS = "docs/architect/decisions.md";
const REFACTOR = "OpenSpec-Refactor: dice-and-checks";

function crateChange(body, extra = {}) {
  return {
    event: "pull_request",
    body,
    base: { [LIB]: "old\n" },
    head: { [LIB]: "new\n" },
    ...extra,
  };
}

function docsChange(body, extra = {}) {
  return {
    event: "pull_request",
    body,
    base: { [DOCS]: "old\n" },
    head: { [DOCS]: "new\n" },
    ...extra,
  };
}

function exactLine(text) {
  return (output) => {
    assert.ok(output.split("\n").includes(text), output);
  };
}

function refactorCases() {
  return [
    passing("The diff starts at the merge-base", {
      event: "pull_request",
      body: "",
      base: { [LIB]: "at-merge\n" },
      advanceBase: { [LIB]: "on-base\n" },
      captureDiff: true,
    }),
    failing("No merge-base fails closed", ["spec-drift: no merge-base"], {
      event: "pull_request",
      body: "",
      unrelatedBase: true,
      base: { [LIB]: "old\n" },
    }),
    passing("A qualifying refactor does not log a missing change line", {
      ...crateChange(REFACTOR),
      assertCodePath: true,
    }),
    passing("A trimmed refactor line passes", crateChange("  OpenSpec-Refactor: dice-and-checks  \r")),
    passing("An item absent from the decision log passes", {
      ...crateChange("OpenSpec-Refactor: not-a-real-item"),
      excludes: ["spec-drift: unknown change not-a-real-item", "spec-drift:"],
    }),
    passing(
      "A fenced refactor line counts",
      crateChange("```\nOpenSpec-Refactor: grid-movement\n```\n"),
    ),
    failing("A lowercase refactor prefix is ignored", [MISSING], {
      ...crateChange("openspec-refactor: dice-and-checks"),
      excludes: [
        "spec-drift: invalid refactor item",
        "spec-drift: invalid change id",
        "spec-drift: unknown change",
      ],
    }),
    passing("A push ignores a malformed refactor line", {
      body: "OpenSpec-Refactor:",
      base: { [LIB]: "old\n" },
      head: { [LIB]: "new\n" },
      excludes: ["spec-drift: invalid refactor item", "spec-drift:"],
    }),
    failing("An empty refactor line is invalid", [], {
      ...docsChange("OpenSpec-Refactor:"),
      assertOutput: exactLine("spec-drift: invalid refactor item"),
      excludes: [MISSING],
    }),
    failing(
      "Extra words are an invalid refactor item",
      ["spec-drift: invalid refactor item dice-and-checks extra"],
      { ...docsChange("OpenSpec-Refactor: dice-and-checks extra"), excludes: [MISSING] },
    ),
    failing(
      "A tab after the refactor colon is escaped",
      ["spec-drift: invalid refactor item\\u0009dice"],
      { ...docsChange("OpenSpec-Refactor:\tdice"), excludes: [MISSING] },
    ),
    failing(
      "An uppercase refactor item is invalid",
      ["spec-drift: invalid refactor item NOT-AN-ITEM"],
      { ...docsChange("OpenSpec-Refactor: NOT-AN-ITEM"), excludes: [MISSING] },
    ),
    failing(
      "A slash in a refactor item is invalid",
      ["spec-drift: invalid refactor item dice/checks"],
      { ...docsChange("OpenSpec-Refactor: dice/checks"), excludes: [MISSING] },
    ),
    failing(
      "A double hyphen refactor item is invalid",
      ["spec-drift: invalid refactor item dice--checks"],
      { ...docsChange("OpenSpec-Refactor: dice--checks"), excludes: [MISSING] },
    ),
    failing(
      "A repeated refactor line fails",
      ["spec-drift: multiple OpenSpec-Refactor lines"],
      {
        ...crateChange(`${REFACTOR}\n${REFACTOR}`),
        excludes: ["spec-drift: refactor is not crates-only", MISSING],
      },
    ),
    failing(
      "Both lines fail on a docs-only pull request",
      ["spec-drift: refactor line conflicts with OpenSpec-Change"],
      {
        ...docsChange(`${REFACTOR}\nOpenSpec-Change: add-foo`),
        base: {
          [DOCS]: "old\n",
          "openspec/changes/add-foo/tasks.md": OPEN,
        },
        excludes: [MISSING],
      },
    ),
    passing(
      "A crate file with a changelog passes",
      crateChange(REFACTOR, {
        base: { [LIB]: "old\n", "CHANGELOG.md": "# Log\n" },
        head: { [LIB]: "new\n", "CHANGELOG.md": "# Log\n\n" },
      }),
    ),
    passing("Adding a test file passes", {
      event: "pull_request",
      body: REFACTOR,
      head: { "crates/rules/src/encounter/tests/new_tests.rs": "fn new() {}\n" },
    }),
    failing(
      "A crate file plus README is not a refactor",
      ["spec-drift: refactor is not crates-only"],
      crateChange(REFACTOR, {
        base: { [LIB]: "old\n", "README.md": "old\n" },
        head: { [LIB]: "new\n", "README.md": "new\n" },
      }),
    ),
    failing(
      "A lowercase changelog name is not the bullet file",
      ["spec-drift: refactor is not crates-only"],
      crateChange(REFACTOR, {
        base: { [LIB]: "old\n", "changelog.md": "old\n" },
        head: { [LIB]: "new\n", "changelog.md": "new\n" },
      }),
    ),
    failing(
      "A crate file plus openspec logs both path lines",
      ["spec-drift: refactor is not crates-only", "spec-drift: refactor changes openspec"],
      {
        ...crateChange(REFACTOR, {
          base: { [LIB]: "old\n", "openspec/specs/spec-drift-check/spec.md": "old\n" },
          head: { [LIB]: "new\n", "openspec/specs/spec-drift-check/spec.md": "new\n" },
        }),
        assertOutput(output) {
          const crates = output.indexOf("spec-drift: refactor is not crates-only");
          const openspec = output.indexOf("spec-drift: refactor changes openspec");
          assert.ok(crates >= 0 && openspec > crates, output);
        },
      },
    ),
    failing("Renaming into crates counts both sides", ["spec-drift: refactor is not crates-only"], {
      event: "pull_request",
      body: REFACTOR,
      base: { "docs/note.md": "note\n" },
      rename: [{ from: "docs/note.md", to: "crates/rules/note.md" }],
    }),
    passing("A docs-only valid refactor line passes", {
      ...docsChange(REFACTOR),
      excludes: ["spec-drift: refactor is not crates-only", "spec-drift:"],
    }),
    failing(
      "A tests directory path is a test file",
      ["spec-drift: refactor changes existing tests"],
      {
        event: "pull_request",
        body: REFACTOR,
        base: { "crates/rules/src/encounter/tests/delay_ready_tests.rs": "fn old() {}\n" },
        head: { "crates/rules/src/encounter/tests/delay_ready_tests.rs": "fn new() {}\n" },
      },
    ),
    passing("A sibling tests.rs file is not a test file", {
      event: "pull_request",
      body: REFACTOR,
      base: { "crates/rules/src/tests.rs": "fn old() {}\n" },
      head: { "crates/rules/src/tests.rs": "fn new() {}\n" },
    }),
    passing("An inline test module is not classified", {
      ...crateChange(REFACTOR, {
        head: { [LIB]: "fn demo() {}\n#[cfg(test)]\nmod tests {}\n" },
      }),
      assertNoRustParse: true,
    }),
    failing("Modifying a test file fails", ["spec-drift: refactor changes existing tests"], {
      event: "pull_request",
      body: REFACTOR,
      base: { "crates/rules/tests/foo.rs": "fn old() {}\n" },
      head: { "crates/rules/tests/foo.rs": "fn new() {}\n" },
    }),
    failing("Deleting a test file fails", ["spec-drift: refactor changes existing tests"], {
      event: "pull_request",
      body: REFACTOR,
      base: { "crates/rules/tests/foo.rs": "fn old() {}\n" },
      remove: ["crates/rules/tests/foo.rs"],
    }),
    passing("Adding a test file is allowed", {
      event: "pull_request",
      body: REFACTOR,
      head: { "crates/rules/tests/foo.rs": "fn new() {}\n" },
    }),
    failing("A typechange of a test file fails", ["spec-drift: refactor changes existing tests"], {
      event: "pull_request",
      body: REFACTOR,
      base: { "crates/rules/tests/foo.rs": "fn old() {}\n" },
      symlink: { "crates/rules/tests/foo.rs": "foo.rs" },
    }),
    failing("Renaming a test file fails", ["spec-drift: refactor changes existing tests"], {
      event: "pull_request",
      body: REFACTOR,
      base: { "crates/rules/tests/foo.rs": "fn old() {}\n" },
      rename: [{ from: "crates/rules/tests/foo.rs", to: "crates/rules/tests/bar.rs" }],
    }),
    passing("Renaming a non-test file into tests passes", {
      event: "pull_request",
      body: REFACTOR,
      base: { [LIB]: "fn old() {}\n" },
      rename: [{ from: LIB, to: "crates/rules/src/encounter/tests/lib.rs" }],
    }),
    failing("Mixed body failures keep that order", [], {
      ...crateChange(
        "OpenSpec-Change: Add-Foo\nOpenSpec-Refactor: NOT-AN-ITEM\nOpenSpec-Refactor: dice-and-checks",
      ),
      assertOutput(output) {
        const lines = [
          "spec-drift: invalid change id Add-Foo",
          "spec-drift: invalid refactor item NOT-AN-ITEM",
          "spec-drift: multiple OpenSpec-Refactor lines",
          "spec-drift: refactor line conflicts with OpenSpec-Change",
        ];
        let at = -1;
        for (const line of lines) {
          const next = output.indexOf(line);
          assert.ok(next > at, output);
          at = next;
        }
      },
    }),
    failing("Token failures omit path lines", ["spec-drift: refactor line conflicts with OpenSpec-Change"], {
      ...crateChange(
        "OpenSpec-Change: Add-Foo\nOpenSpec-Refactor: NOT-AN-ITEM\nOpenSpec-Refactor: dice-and-checks",
      ),
      excludes: ["spec-drift: refactor is not crates-only", "spec-drift: refactor changes openspec"],
    }),
    passing("Spec-drift ignores a missing bullet", {
      ...crateChange(REFACTOR),
      excludes: ["implementation PR missing CHANGELOG bullet", "spec-drift:"],
    }),
  ];
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
  failing(
    "Extra words are not a reference",
    ["spec-drift: invalid change id add-dice-and-checks extra"],
    {
      event: "pull_request",
      body: "OpenSpec-Change: add-dice-and-checks extra",
      base: {
        "Cargo.toml": "old\n",
        "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
      },
      head: { "Cargo.toml": "new\n" },
      excludes: [MISSING, "spec-drift: unknown change"],
    },
  ),
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
    ["spec-drift: invalid change id\n", "spec-drift: invalid change idAdd"],
    {
      event: "pull_request",
      body: "OpenSpec-Change:\nOpenSpec-Change:Add",
      base: { "docs/architect/decisions.md": "old\n" },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
    },
  ),
  failing("A spaceless change id is invalid", ["spec-drift: invalid change idadd-foo"], {
    event: "pull_request",
    body: "OpenSpec-Change:add-foo",
    base: {
      "docs/architect/decisions.md": "old\n",
      "openspec/changes/add-foo/tasks.md": OPEN,
    },
    head: { "docs/architect/decisions.md": "new\n" },
    excludes: [MISSING],
  }),
  failing(
    "Two spaces before the id are invalid",
    ["spec-drift: invalid change id  add-foo"],
    {
      event: "pull_request",
      body: "OpenSpec-Change:  add-foo",
      base: {
        "docs/architect/decisions.md": "old\n",
        "openspec/changes/add-foo/tasks.md": OPEN,
      },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
    },
  ),
  failing("A tab after the colon is invalid", ["spec-drift: invalid change id\\u0009add-foo"], {
    event: "pull_request",
    body: "OpenSpec-Change:\tadd-foo",
    base: {
      "docs/architect/decisions.md": "old\n",
      "openspec/changes/add-foo/tasks.md": OPEN,
    },
    head: { "docs/architect/decisions.md": "new\n" },
    excludes: [MISSING],
    assertOutput(output) {
      assert.equal(output.includes('"'), false, output);
    },
  }),
  failing("A lowercase prefix is ignored", [MISSING], {
    event: "pull_request",
    body: "openspec-change: foo",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/foo/tasks.md": OPEN,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
    excludes: ["spec-drift: invalid change id", "spec-drift: unknown change foo"],
  }),
  failing("A zero-width space is visible", ["spec-drift: invalid change id \\u200badd-foo"], {
    event: "pull_request",
    body: "OpenSpec-Change: \u200badd-foo",
    base: { "docs/architect/decisions.md": "old\n" },
    head: { "docs/architect/decisions.md": "new\n" },
    excludes: [MISSING],
    assertOutput(output) {
      assert.equal(output.includes("\u200b"), false, output);
    },
  }),
  failing("An NBSP is visible", ["spec-drift: invalid change id \\u00a0add-foo"], {
    event: "pull_request",
    body: "OpenSpec-Change: \u00a0add-foo",
    base: { "docs/architect/decisions.md": "old\n" },
    head: { "docs/architect/decisions.md": "new\n" },
    excludes: [MISSING],
    assertOutput(output) {
      assert.equal(output.includes("\u00a0"), false, output);
    },
  }),
  failing(
    "An astral character is escaped per UTF-16 code unit",
    ["spec-drift: invalid change id \\ud83d\\ude00add-foo"],
    {
      event: "pull_request",
      body: "OpenSpec-Change: \u{1F600}add-foo",
      base: { "docs/architect/decisions.md": "old\n" },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
      assertOutput(output) {
        assert.equal(output.includes("\u{1F600}"), false, output);
        assert.equal(output.includes("\\ud83d\\ude00"), true, output);
      },
    },
  ),
  failing("An em space is visible", ["spec-drift: invalid change id \\u2003add-foo"], {
    event: "pull_request",
    body: "OpenSpec-Change: \u2003add-foo",
    base: { "docs/architect/decisions.md": "old\n" },
    head: { "docs/architect/decisions.md": "new\n" },
    excludes: [MISSING],
    assertOutput(output) {
      assert.equal(output.includes("\u2003"), false, output);
    },
  }),
  failing(
    "A typed backslash escape differs from a real NBSP",
    [
      "spec-drift: invalid change id \\u005cu00a0add-foo",
      "spec-drift: invalid change id \\u00a0add-foo",
    ],
    {
      event: "pull_request",
      body: "OpenSpec-Change: \\u00a0add-foo\nOpenSpec-Change: \u00a0add-foo",
      base: { "docs/architect/decisions.md": "old\n" },
      head: { "docs/architect/decisions.md": "new\n" },
      excludes: [MISSING],
      assertOutput(output) {
        const typed = "spec-drift: invalid change id \\u005cu00a0add-foo";
        const real = "spec-drift: invalid change id \\u00a0add-foo";
        assert.notEqual(typed, real);
        assert.equal(output.includes("\u00a0"), false, output);
        assert.equal(output.includes('"'), false, output);
      },
    },
  ),
  failing(
    "A tabbed extra word beside a valid line fails",
    ["spec-drift: invalid change id \\u0009Add-Foo extra"],
    {
      event: "pull_request",
      body: "OpenSpec-Change: add-dice-and-checks\nOpenSpec-Change: \tAdd-Foo extra",
      base: {
        "Cargo.toml": "old\n",
        "openspec/changes/add-dice-and-checks/tasks.md": OPEN,
      },
      head: { "Cargo.toml": "new\n" },
      excludes: [MISSING],
    },
  ),
  failing(
    "A docs-only malformed line fails",
    ["spec-drift: invalid change id add-dice-and-checks extra"],
    {
      event: "pull_request",
      body: "OpenSpec-Change: add-dice-and-checks extra",
      base: { "README.md": "old\n" },
      head: { "README.md": "new\n" },
      excludes: [MISSING],
    },
  ),
  failing("Completed changes are listed in byte order", [completed("Zed"), completed("alpha")], {
    base: {
      "openspec/changes/Zed/tasks.md": DONE,
      "openspec/changes/alpha/tasks.md": DONE,
    },
    assertOutput(output) {
      const byte = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
      assert.notEqual(byte("Zed", "alpha"), "Zed".localeCompare("alpha", "en"));
      assert.notEqual(byte("Zed", "alpha"), "Zed".localeCompare("alpha", "sv"));
      assert.notEqual(byte("Zed", "alpha"), "Zed".localeCompare("alpha", "fr"));
      const zed = output.indexOf(completed("Zed"));
      const alpha = output.indexOf(completed("alpha"));
      assert.ok(zed >= 0 && alpha > zed, output);
    },
  }),
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
  failing("A dated archive folder name is not an id", [unknown("2026-10-21-foo")], {
    event: "pull_request",
    body: "OpenSpec-Change: 2026-10-21-foo",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/archive/2026-10-21-foo/tasks.md": DONE,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
    excludes: [MISSING],
  }),
  passing("The stripped id of a dated folder exists", {
    event: "pull_request",
    body: "OpenSpec-Change: foo",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/archive/2026-10-21-foo/tasks.md": DONE,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  passing("An active dated directory name is still an id", {
    event: "pull_request",
    body: "OpenSpec-Change: 2026-10-21-foo",
    base: {
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/2026-10-21-foo/tasks.md": OPEN,
    },
    head: { "crates/rules/src/lib.rs": "new\n" },
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
  ...whitespaceReferenceCases(),
  ...refactorCases(),
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
  assert.doesNotMatch(editedWorkflow, /cargo /);
  const ciSteps = stepsBlock(job);
  const editedSteps = stepsBlock(jobBlock(editedWorkflow, "spec-drift"));
  assert.equal(editedSteps, ciSteps);
  const suite = editedSteps.indexOf("node --test scripts/spec-drift-check.test.mjs");
  const script = editedSteps.indexOf("node scripts/spec-drift-check.mjs");
  assert.ok(suite >= 0);
  assert.ok(script > suite);
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
  assert.doesNotMatch(jobBlock(workflow, "ci"), /concurrency:/);
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
  const jobs = editedWorkflow
    .slice(editedWorkflow.indexOf("jobs:"))
    .match(/^ {2}[a-z0-9-]+:$/gm);
  assert.deepEqual(jobs, ["  spec-drift:"]);
  const job = jobBlock(editedWorkflow, "spec-drift");
  const suite = job.indexOf("node --test scripts/spec-drift-check.test.mjs");
  const script = job.indexOf("node scripts/spec-drift-check.mjs");
  assert.ok(suite >= 0);
  assert.ok(script > suite);
  assert.equal(stepsBlock(job), stepsBlock(jobBlock(workflow, "spec-drift")));
  assert.ok(job.includes(`group: ${SPEC_DRIFT_GROUP}`), job);
  assert.match(job, /cancel-in-progress:\s*true/);
  const header = editedWorkflow.slice(0, editedWorkflow.indexOf("jobs:"));
  assert.equal(header.includes("concurrency:"), false);
  assert.doesNotMatch(editedWorkflow, /spec-drift-edited-/);
  assert.doesNotMatch(editedWorkflow, /pr-hygiene/);
});

test("Both spec-drift jobs share a pull request group", () => {
  for (const yaml of [workflow, editedWorkflow]) {
    const job = jobBlock(yaml, "spec-drift");
    assert.ok(job.includes(`group: ${SPEC_DRIFT_GROUP}`), job);
    assert.match(job, /cancel-in-progress:\s*true/);
    const header = yaml.slice(0, yaml.indexOf("jobs:"));
    assert.equal(header.includes("concurrency:"), false, header);
    assert.equal(header.includes(SPEC_DRIFT_GROUP), false);
  }
  assert.doesNotMatch(jobBlock(workflow, "ci"), /concurrency:/);
  assert.doesNotMatch(editedWorkflow, /spec-drift-edited-/);
});

test("A push does not join a pull request group", () => {
  const job = jobBlock(workflow, "spec-drift");
  assert.ok(job.includes(`group: ${SPEC_DRIFT_GROUP}`), job);
  const sha = "abc123def";
  const pushGroup = `spec-drift-${"" || sha}`;
  const pullGroup = `spec-drift-${"31" || sha}`;
  assert.equal(pushGroup, `spec-drift-${sha}`);
  assert.equal(pullGroup, "spec-drift-31");
  assert.notEqual(pushGroup, pullGroup);
  assert.equal(pushGroup.includes("31"), false);
  assert.match(job, /github\.sha/);
  assert.doesNotMatch(job, /group: spec-drift-\$\{\{ github\.event\.pull_request\.number \}\}/);
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

function stepsBlock(job) {
  const marker = "\n    steps:\n";
  const start = job.indexOf(marker);
  assert.ok(start >= 0, "missing steps");
  return job.slice(start);
}

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
    let baseSha = git(dir, ["rev-parse", "HEAD"]).stdout.trim();

    const renames = row.rename ?? [];
    const removals = row.remove ?? [];
    const hasHead =
      row.head !== undefined || removals.length > 0 || renames.length > 0 || row.symlink;
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
      if (row.symlink) {
        for (const [rel, target] of Object.entries(row.symlink)) {
          const full = join(dir, rel);
          await rm(full, { force: true });
          await symlink(target, full);
        }
      }
      git(dir, ["add", "-A"]);
      git(dir, ["commit", "-m", row.commitMessage ?? "head"]);
    }

    if (row.advanceBase) {
      const mergePoint = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
      git(dir, ["checkout", "-B", "pr-base", mergePoint]);
      await writeTree(dir, row.advanceBase);
      git(dir, ["add", "-A"]);
      git(dir, ["commit", "-m", "base tip"]);
      baseSha = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
      git(dir, ["checkout", "--detach", mergePoint]);
    }
    if (row.unrelatedBase) {
      const mergePoint = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
      git(dir, ["checkout", "--orphan", "unrelated"]);
      git(dir, ["rm", "-rf", "."]);
      await writeTree(dir, { "unrelated.txt": "unrelated\n" });
      git(dir, ["add", "-A"]);
      git(dir, ["commit", "-m", "unrelated"]);
      baseSha = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
      git(dir, ["checkout", "--detach", mergePoint]);
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
    const gitArgsLog = join(dir, "git-args.txt");
    if (row.recordGitLocale || row.captureDiff) {
      const gitStub = join(bin, "git");
      const body = row.captureDiff
        ? "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$GIT_ARGS_LOG\"\nexec \"$REAL_GIT\" \"$@\"\n"
        : "#!/bin/sh\nif [ \"$1\" = \"diff\" ]; then\n  printf '%s\\n' \"$LC_ALL\" > \"$GIT_LOCALE_LOG\"\nfi\nexec \"$REAL_GIT\" \"$@\"\n";
      await writeFile(gitStub, body);
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
    if (row.captureDiff) {
      env.GIT_ARGS_LOG = gitArgsLog;
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
    if (row.captureDiff) {
      const log = readFileSync(gitArgsLog, "utf8");
      const diffLine = log.split("\n").find((line) => line.startsWith("diff "));
      assert.ok(diffLine, log);
      assert.match(diffLine, /--find-renames/);
      assert.equal(diffLine.includes("--find-copies"), false, diffLine);
      const fromSha = diffLine.split(" ").at(-2);
      assert.notEqual(fromSha, baseSha, log);
    }
    if (row.assertCodePath) {
      const mod = await import("./spec-drift-check.mjs");
      assert.equal(mod.isCodePath("crates/rules/src/lib.rs"), true);
    }
    if (row.assertNoRustParse) {
      assert.equal(readFileSync(scriptPath, "utf8").includes("cfg(test)"), false);
    }
    if (row.assertOutput) {
      row.assertOutput(output);
    }
    if (row.assertRepo) {
      row.assertRepo(dir);
    }
  } finally {
    await removeFixture(dir);
  }
}

const REMOVAL_ATTEMPTS = 5;
const REMOVAL_BACKOFF_MS = 10;

function isEnoempty(error) {
  return typeof error === "object" && error !== null && error.code === "ENOTEMPTY";
}

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

// fs.rm maxRetries is not the mechanism. The loop is what a test injects
// ENOTEMPTY into, and the backoff gives a parallel rmdir time to finish.
async function removeFixture(dir, remover = rm) {
  for (let attempt = 1; attempt <= REMOVAL_ATTEMPTS; attempt += 1) {
    try {
      await remover(dir, { recursive: true, force: true });
      return;
    } catch (error) {
      if (!isEnoempty(error) || attempt === REMOVAL_ATTEMPTS) {
        throw error;
      }
      await delay(REMOVAL_BACKOFF_MS * 2 ** (attempt - 1));
    }
  }
}

function enoemptyOnObjects(dir) {
  const objects = join(dir, ".git", "objects");
  const error = new Error(`ENOTEMPTY: directory not empty, rmdir '${objects}'`);
  error.code = "ENOTEMPTY";
  error.path = objects;
  return error;
}

test("A transient ENOTEMPTY is retried", async () => {
  const dir = await mkdtemp(join(tmpdir(), "spec-drift-eno-"));
  try {
    await mkdir(join(dir, ".git", "objects"), { recursive: true });
    let calls = 0;
    const remover = async (target, options) => {
      calls += 1;
      if (calls === 1) {
        throw enoemptyOnObjects(target);
      }
      await rm(target, options);
    };
    await removeFixture(dir, remover);
    assert.equal(calls, 2);
    assert.equal(existsSync(dir), false);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("Retries stop after five", async () => {
  const dir = await mkdtemp(join(tmpdir(), "spec-drift-eno-"));
  const error = enoemptyOnObjects(dir);
  let calls = 0;
  const remover = async () => {
    calls += 1;
    throw error;
  };
  try {
    await assert.rejects(
      removeFixture(dir, remover),
      (caught) => caught === error,
    );
    assert.equal(calls, 5);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

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

test("A refactor line contributes no change id", () => {
  const parsed = changeReferences("OpenSpec-Refactor: dice-and-checks");
  assert.deepEqual(parsed.ids, []);
  assert.deepEqual(parsed.invalidTokens, []);
});

test("A refactor line is not an own-change line", () => {
  const parsed = changeReferences("OpenSpec-Refactor: dice-and-checks\n");
  assert.deepEqual(parsed.ids, []);
  assert.deepEqual(parsed.invalidTokens, []);
});

test("The bullet stays with the other specs", () => {
  const root = fileURLToPath(new URL("..", import.meta.url));
  const hygiene = readFileSync(join(root, "openspec/specs/pr-hygiene/spec.md"), "utf8");
  const changelog = readFileSync(join(root, "openspec/specs/changelog/spec.md"), "utf8");
  const script = readFileSync(scriptPath, "utf8");
  assert.match(hygiene, /implementation PR missing CHANGELOG bullet/);
  assert.match(changelog, /## \[Unreleased\]/);
  assert.equal(script.includes("implementation PR missing CHANGELOG bullet"), false);
});
