import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./pr-hygiene.mjs", import.meta.url));
const workflowPath = fileURLToPath(
  new URL("../.github/workflows/pr-hygiene.yml", import.meta.url),
);
const ciPath = fileURLToPath(new URL("../.github/workflows/ci.yml", import.meta.url));
const editedWorkflowPath = fileURLToPath(
  new URL("../.github/workflows/spec-drift-edited.yml", import.meta.url),
);
const workflow = readFileSync(workflowPath, "utf8");
const ciWorkflow = readFileSync(ciPath, "utf8");
const editedWorkflow = readFileSync(editedWorkflowPath, "utf8");
const scriptSource = readFileSync(scriptPath, "utf8");

const realGit = spawnSync("which", ["git"], { encoding: "utf8" }).stdout.trim();

const MISSING = "implementation PR missing CHANGELOG bullet";
const ARCHIVE = "archive commit is not last";
const PULL_URL = "https://github.com/fputs/genesis/pull";

function linkedBullet(text, number = 7) {
  return `- ${text} ([#${number}](${PULL_URL}/${number}))`;
}

const LOG = `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n`;
const NEW_LOG = `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n${linkedBullet("new gate")}\n`;
const GATE_LOG = `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n- Record the gate\n`;
const LINKED_GATE = `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`;
const OUTSIDE_LOG = "## [Unreleased]\n\n### Added\n- old\n\n## [0.1.0]\n\n- shipped\n";
const NO_HEADING = "# Changelog\n\n- old\n";
const MOVED_BASE =
  "## [Unreleased]\n\n### Added\n- old\n\n## [0.1.0]\n\n- Record the gate\n";
const MOVED_HEAD =
  "## [Unreleased]\n\n### Added\n- old\n- Record the gate\n\n## [0.1.0]\n\n- Record the gate\n";
const EDITED_BASE = "## [Unreleased]\n\n### Added\n- Record the gate\n";
const EDITED_HEAD = "## [Unreleased]\n\n### Added\n- Record the gate today\n";
const FENCED_BULLET = "## [Unreleased]\n\n### Added\n- old\n\n```\n- brand new bullet\n```\n";
const FENCED_OPEN = "```\n## [Unreleased]\n\n- brand new bullet\n```\n";
const FENCED_CLOSE = `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n\n\`\`\`\n## [0.1.0]\n\`\`\`\n\n${linkedBullet("brand new bullet")}\n`;
const MAIN_BULLET = linkedBullet("main only bullet");
const MAIN_LOG = `${LOG}${MAIN_BULLET}\n`;

function withLog(files) {
  return { "CHANGELOG.md": LOG, ...files };
}

function archiveRename(id) {
  return {
    from: `openspec/changes/${id}/proposal.md`,
    to: `openspec/changes/archive/2026-10-21-${id}/proposal.md`,
  };
}

function passing(name, fields = {}) {
  return {
    event: "pull_request",
    action: "opened",
    body: "",
    excludes: [MISSING, ARCHIVE],
    includes: [],
    ...fields,
    name,
    status: 0,
  };
}

function failing(name, lines, fields = {}) {
  const excludes = fields.excludes ?? [
    ...(lines.includes(MISSING) ? [] : [MISSING]),
    ...(lines.includes(ARCHIVE) ? [] : [ARCHIVE]),
  ];
  return {
    event: "pull_request",
    action: "opened",
    body: "",
    ...fields,
    name,
    status: 1,
    includes: lines,
    excludes,
  };
}

const cases = [
  failing("A diff parse error fails closed", ["diff failed"], {
    gitStub: "fail-diff",
    base: { "docs/architect/decisions.md": "old\n" },
    head: { "docs/architect/decisions.md": "new\n" },
  }),
  failing(
    "A broken NUL record fails closed",
    ["pr-hygiene: NUL record could not be parsed"],
    {
      gitStub: "broken-nul",
      base: { "docs/architect/decisions.md": "old\n" },
      head: { "docs/architect/decisions.md": "new\n" },
    },
  ),
  failing(
    "An event payload that cannot be parsed fails closed",
    ["pr-hygiene: event payload could not be parsed"],
    {
      invalidPayload: true,
      base: { "docs/architect/decisions.md": "old\n" },
      head: { "docs/architect/decisions.md": "new\n" },
    },
  ),
  failing("A rename into crates is an implementation pull request", [MISSING], {
    base: withLog({ "docs/note.md": "same\n" }),
    rename: [{ from: "docs/note.md", to: "crates/rules/note.md" }],
  }),
  failing("A crate file is an implementation pull request", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("A workflow file is an implementation pull request", [MISSING], {
    base: withLog({ ".github/workflows/ci.yml": "old\n" }),
    head: { ".github/workflows/ci.yml": "new\n" },
  }),
  failing("Root Cargo.toml is an implementation pull request", [MISSING], {
    base: withLog({ "Cargo.toml": "old\n" }),
    head: { "Cargo.toml": "new\n" },
  }),
  failing("A scripts-only pull request is not an implementation pull request", [MISSING], {
    base: withLog({ "scripts/note.mjs": "old\n" }),
    head: { "scripts/note.mjs": "new\n" },
  }),
  failing("A scripts-only pull request needs a changelog bullet", [MISSING], {
    base: withLog({ "scripts/note.mjs": "old\n" }),
    head: { "scripts/note.mjs": "new\n" },
  }),
  passing("A nested Cargo.toml is not a root manifest", {
    base: withLog({ "docs/Cargo.toml": "old\n" }),
    head: { "docs/Cargo.toml": "new\n" },
  }),
  passing("An added Unreleased bullet passes", {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
  }),
  failing("A missing bullet fails", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("A bullet outside Unreleased fails", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": OUTSIDE_LOG },
  }),
  passing("An Added bullet linking pull request 7 passes", {
    base: withLog({ ".github/workflows/pr-hygiene.yml": "old\n" }),
    head: {
      ".github/workflows/pr-hygiene.yml": "new\n",
      "CHANGELOG.md": LINKED_GATE,
    },
  }),
  failing("A bullet with no link fails", [MISSING], {
    base: withLog({ ".github/workflows/pr-hygiene.yml": "old\n" }),
    head: {
      ".github/workflows/pr-hygiene.yml": "new\n",
      "CHANGELOG.md": GATE_LOG,
    },
  }),
  failing("An unchanged existing bullet fails", [MISSING], {
    base: withLog({ "Cargo.toml": "old\n" }),
    head: { "Cargo.toml": "new\n" },
  }),
  passing("Docs-only passes without a bullet", {
    base: {
      "docs/architect/decisions.md": "old\n",
      "AGENTS.md": "old\n",
      "CHANGELOG.md": LOG,
    },
    head: {
      "docs/architect/decisions.md": "new\n",
      "AGENTS.md": "new\n",
      "CHANGELOG.md": "## [Unreleased]\n\n### Added\nnote\n- old\n",
    },
  }),
  passing("Openspec-only passes without a bullet", {
    base: withLog({ "openspec/specs/changelog/spec.md": "old\n" }),
    head: { "openspec/specs/changelog/spec.md": "new\n" },
  }),
  passing("A cursor-only pull request passes without a bullet", {
    base: withLog({ ".cursor/rules/example.md": "old\n" }),
    head: { ".cursor/rules/example.md": "new\n" },
  }),
  passing("A README-only pull request passes without a bullet", {
    base: withLog({ "README.md": "old\n" }),
    head: { "README.md": "new\n" },
  }),
  passing("A Cargo.lock-only pull request passes without a bullet", {
    base: withLog({ "Cargo.lock": "old\n" }),
    head: { "Cargo.lock": "new\n" },
  }),
  passing("A rust-toolchain-only pull request passes without a bullet", {
    base: withLog({ "rust-toolchain.toml": "old\n" }),
    head: { "rust-toolchain.toml": "new\n" },
  }),
  passing("The exempt path set together passes without a bullet", {
    base: withLog({
      ".cursor/rules/example.md": "old\n",
      "README.md": "old\n",
      "Cargo.lock": "old\n",
      "rust-toolchain.toml": "old\n",
    }),
    head: {
      ".cursor/rules/example.md": "new\n",
      "README.md": "new\n",
      "Cargo.lock": "new\n",
      "rust-toolchain.toml": "new\n",
    },
  }),
  failing("Docs plus a crate still needs a bullet", [MISSING], {
    base: withLog({
      "docs/architect/decisions.md": "old\n",
      "crates/rules/src/lib.rs": "old\n",
    }),
    head: {
      "docs/architect/decisions.md": "new\n",
      "crates/rules/src/lib.rs": "new\n",
    },
  }),
  failing("Openspec plus a workflow file still needs a bullet", [MISSING], {
    base: withLog({
      "openspec/specs/changelog/spec.md": "old\n",
      ".github/workflows/ci.yml": "old\n",
    }),
    head: {
      "openspec/specs/changelog/spec.md": "new\n",
      ".github/workflows/ci.yml": "new\n",
    },
  }),
  failing("Cursor plus a crate still needs a bullet", [MISSING], {
    base: withLog({
      ".cursor/rules/example.md": "old\n",
      "crates/rules/src/lib.rs": "old\n",
    }),
    head: {
      ".cursor/rules/example.md": "new\n",
      "crates/rules/src/lib.rs": "new\n",
    },
  }),
  failing("README plus root Cargo.toml still needs a bullet", [MISSING], {
    base: withLog({ "README.md": "old\n", "Cargo.toml": "old\n" }),
    head: { "README.md": "new\n", "Cargo.toml": "new\n" },
  }),
  failing("Cargo.lock plus a crate still needs a bullet", [MISSING], {
    base: withLog({ "Cargo.lock": "old\n", "crates/rules/src/lib.rs": "old\n" }),
    head: { "Cargo.lock": "new\n", "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("rust-toolchain.toml plus a workflow file still needs a bullet", [MISSING], {
    base: withLog({
      "rust-toolchain.toml": "old\n",
      ".github/workflows/ci.yml": "old\n",
    }),
    head: {
      "rust-toolchain.toml": "new\n",
      ".github/workflows/ci.yml": "new\n",
    },
  }),
  failing("A label does not skip the bullet", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n" },
    labels: ["skip-changelog"],
    usePayloadForShas: true,
    usePayloadForBody: true,
  }),
  failing("Extra words are not a change reference", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo extra",
    base: {
      "openspec/changes/add-foo/proposal.md": "proposal\n",
      "README.md": "old\n",
    },
    commits: [
      {
        message: "remove",
        remove: ["openspec/changes/add-foo"],
        files: { "README.md": "new\n" },
      },
    ],
  }),
  failing("A fenced line does not exempt a removal", [ARCHIVE], {
    body: "```\nOpenSpec-Change: add-foo\n```",
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "update",
        remove: ["openspec/changes/add-foo"],
      },
    ],
  }),
  passing("The archive commit passes", {
    body: "",
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("A path outside archive and specs fails", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo",
    base: {
      "openspec/changes/add-foo/proposal.md": "proposal\n",
      "docs/producer/process.md": "old\n",
    },
    commits: [
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: {
          "openspec/specs/changelog/spec.md": "spec\n",
          "docs/producer/process.md": "new\n",
        },
      },
    ],
  }),
  failing("A later fix-typo commit is not last", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo",
    base: {
      "openspec/changes/add-foo/proposal.md": "proposal\n",
      "docs/producer/process.md": "old\n",
    },
    commits: [
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
      {
        message: "fix typo",
        files: { "docs/producer/process.md": "typo\n" },
      },
    ],
  }),
  failing("A missing archive phrase fails", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo",
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "archive the change",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("A longer id does not cover a shorter id", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo",
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "openspec archive add-foo-extra --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("A body edit does not excuse a bad archive", [ARCHIVE], {
    action: "edited",
    usePayloadForBody: true,
    body: "OpenSpec-Change: add-foo",
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "fix typo",
        remove: ["openspec/changes/add-foo"],
      },
    ],
  }),
  failing("An edited body drops the change reference", [ARCHIVE], {
    action: "edited",
    usePayloadForBody: true,
    body: "Edited without a change id.",
    base: {
      "openspec/changes/add-foo/proposal.md": "proposal\n",
      "README.md": "old\n",
    },
    commits: [
      {
        message: "fix typo",
        remove: ["openspec/changes/add-foo"],
        files: { "README.md": "new\n" },
      },
    ],
  }),
  passing("Two archived ids pass", {
    body: "",
    base: {
      "openspec/changes/add-foo/proposal.md": "foo\n",
      "openspec/changes/add-bar/proposal.md": "bar\n",
    },
    commits: [
      {
        message: "openspec archive add-bar --yes && openspec archive add-foo --yes",
        rename: [archiveRename("add-foo"), archiveRename("add-bar")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("Two archived ids fail", [ARCHIVE], {
    body: "",
    base: {
      "openspec/changes/add-foo/proposal.md": "foo\n",
      "openspec/changes/add-bar/proposal.md": "bar\n",
    },
    commits: [
      {
        message: "openspec archive add-foo --yes\nopenspec archive add-bar --yes",
        rename: [archiveRename("add-foo"), archiveRename("add-bar")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("One of two archive phrases is missing", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo\nOpenSpec-Change: add-bar",
    base: {
      "openspec/changes/add-foo/proposal.md": "foo\n",
      "openspec/changes/add-bar/proposal.md": "bar\n",
    },
    commits: [
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo"), archiveRename("add-bar")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  passing("A duplicate reference counts once", {
    body: "OpenSpec-Change: add-foo\nOpenSpec-Change: add-foo",
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  passing("A named change that stays active skips archive-last", {
    body: "OpenSpec-Change: add-foo",
    base: { "openspec/changes/add-foo/proposal.md": "old\n" },
    head: { "openspec/changes/add-foo/proposal.md": "new\n" },
  }),
  failing("A removal without a reference fails", [ARCHIVE], {
    draft: false,
    body: "",
    base: {
      "openspec/changes/add-foo/proposal.md": "proposal\n",
      "README.md": "old\n",
    },
    commits: [
      {
        message: "remove",
        remove: ["openspec/changes/add-foo"],
        files: { "README.md": "new\n" },
      },
    ],
  }),
  passing("The bullet is added before the archive commit", {
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    commits: [
      {
        message: "implement",
        files: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
      },
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("A bullet inside the archive commit is not last", [ARCHIVE], {
    body: "OpenSpec-Change: add-foo",
    base: withLog({ "openspec/changes/add-foo/proposal.md": "proposal\n" }),
    commits: [
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: {
          "openspec/specs/changelog/spec.md": "spec\n",
          "CHANGELOG.md": NEW_LOG,
        },
      },
    ],
  }),
  passing("A body without a head SHA passes", {
    body: "Ready.",
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
  }),
  failing("Both checks fail together", [MISSING, ARCHIVE], {
    draft: false,
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    commits: [
      {
        message: "squash",
        remove: ["openspec/changes/add-foo"],
        files: { "crates/rules/src/lib.rs": "new\n" },
      },
    ],
    assertOutput(output) {
      const bulletAt = output.indexOf(MISSING);
      const archiveAt = output.indexOf(ARCHIVE);
      assert.ok(bulletAt >= 0 && archiveAt > bulletAt, output);
    },
  }),
  failing("The hygiene job enforces the bullet", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  failing("Edited still checks the bullet", [MISSING], {
    action: "edited",
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  passing("A docs-only pull request with no changelog passes", {
    base: { "docs/architect/decisions.md": "old\n" },
    head: { "docs/architect/decisions.md": "new\n" },
  }),
  failing("An earlier archive add is not last", [ARCHIVE], {
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "copy the archive",
        files: { "openspec/changes/archive/2026-10-21-add-foo/proposal.md": "proposal\n" },
      },
      {
        message: "openspec archive add-foo --yes",
        remove: ["openspec/changes/add-foo"],
      },
    ],
  }),
  failing("An empty head commit fails", [ARCHIVE], {
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "move",
        rename: [archiveRename("add-foo")],
      },
      {
        message: "openspec archive add-foo --yes",
        empty: true,
      },
    ],
  }),
  failing("A specs-only head commit fails", [ARCHIVE], {
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "move",
        rename: [archiveRename("add-foo")],
      },
      {
        message: "openspec archive add-foo --yes",
        files: { "openspec/specs/unrelated/spec.md": "spec\n" },
      },
    ],
  }),
  failing("A phrase below the subject does not count", [ARCHIVE], {
    base: { "openspec/changes/add-foo/proposal.md": "proposal\n" },
    commits: [
      {
        message: "Record the archive\n\nopenspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  passing("A pre-existing archive allows the removal commit", {
    base: {
      "openspec/changes/add-foo/proposal.md": "proposal\n",
      "openspec/changes/archive/2026-10-21-add-foo/proposal.md": "proposal\n",
    },
    commits: [
      {
        message: "openspec archive add-foo --yes",
        remove: ["openspec/changes/add-foo"],
      },
    ],
  }),
  failing("A moved bullet is not new", [MISSING], {
    base: { "crates/rules/src/lib.rs": "old\n", "CHANGELOG.md": MOVED_BASE },
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": MOVED_HEAD },
  }),
  failing("An edited bullet is not new", [MISSING], {
    base: { "Cargo.toml": "old\n", "CHANGELOG.md": EDITED_BASE },
    head: { "Cargo.toml": "new\n", "CHANGELOG.md": EDITED_HEAD },
  }),
  failing("A fenced bullet is not new", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": FENCED_BULLET },
  }),
  failing("A fenced heading does not open the section", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": FENCED_OPEN },
  }),
  passing("A fenced heading does not close the section", {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": FENCED_CLOSE },
  }),
  passing("Exempt categories together pass without a bullet", {
    base: withLog({
      "docs/architect/decisions.md": "old\n",
      "openspec/specs/changelog/spec.md": "old\n",
      ".cursor/rules/example.md": "old\n",
      "README.md": "old\n",
      "Cargo.lock": "old\n",
      "rust-toolchain.toml": "old\n",
    }),
    head: {
      "docs/architect/decisions.md": "new\n",
      "openspec/specs/changelog/spec.md": "new\n",
      ".cursor/rules/example.md": "new\n",
      "README.md": "new\n",
      "Cargo.lock": "new\n",
      "rust-toolchain.toml": "new\n",
    },
  }),
  passing("A draft unarchived implementation pull request passes", {
    draft: true,
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  failing("A draft still runs the bullet check", [MISSING], {
    draft: true,
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
  passing("A draft removal does not apply archive-last", {
    draft: true,
    body: "",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    commits: [
      {
        message: "remove",
        remove: ["openspec/changes/add-foo"],
        files: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
      },
    ],
  }),
  passing("Converting back to draft relaxes archive-last", {
    draft: true,
    action: "converted_to_draft",
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  passing("Archive then mark ready passes", {
    draft: false,
    action: "ready_for_review",
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    commits: [
      {
        message: "implement",
        files: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
      },
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  }),
  failing("A non-draft unarchived implementation pull request fails", [ARCHIVE], {
    draft: false,
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  failing("A ```bash block followed by the id line fails", [ARCHIVE], {
    draft: false,
    body: "```bash\necho hi\n```\nOpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  failing("A ````rust block fails", [ARCHIVE], {
    draft: false,
    body: "````rust\nfn main() {}\n````\nOpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  failing("A ~~~ block fails", [ARCHIVE], {
    draft: false,
    body: "~~~\necho hi\n~~~\nOpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  passing("The id inside an info-string fence passes", {
    draft: false,
    body: "```bash\nOpenSpec-Change: add-foo\n```",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  passing("A shorter closer that doesn't close, so the id stays fenced", {
    draft: false,
    body: "````rust\n```\nOpenSpec-Change: add-foo\n```",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  passing("A non-draft implementation pull request with no change of its own passes", {
    draft: false,
    body: "Ready, with no change line.",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  }),
  failing("A push after ready re-runs the full check", [ARCHIVE], {
    draft: false,
    action: "synchronize",
    body: "OpenSpec-Change: add-foo",
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    commits: [
      {
        message: "implement",
        files: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
      },
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
      {
        message: "fix typo",
        files: { "crates/rules/src/lib.rs": "typo\n" },
      },
    ],
  }),
  failing("A bullet under an unknown heading fails", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Notes\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  failing("A bullet that links another pull request fails", [MISSING], {
    number: 80,
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n${linkedBullet("Record the gate", 10)}\n`,
    },
  }),
  passing("A Deprecated bullet linking this pull request passes", {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Deprecated\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  failing("A missing pull request number fails closed", [MISSING], {
    omitNumber: true,
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  failing("A bullet with no category fails", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  failing("Pull request 8 linking pull 80 fails", [MISSING], {
    number: 8,
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n${linkedBullet("Record the gate", 80)}\n`,
    },
  }),
  failing("A lowercase category heading fails", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### added\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  failing("A level-four heading does not set the category", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n#### Added\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  passing("A level-four heading does not reset the category", {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Added\n${linkedBullet("old")}\n#### Removed\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  failing("The category resets when Unreleased opens", [MISSING], {
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": `### Added\n\n## [Unreleased]\n\n${linkedBullet("old")}\n${linkedBullet("Record the gate")}\n`,
    },
  }),
  passing("A tests-only pull request passes without a bullet", {
    base: withLog({
      "crates/rules/src/encounter/tests/delay_ready_tests.rs": "old\n",
    }),
    head: {
      "crates/rules/src/encounter/tests/delay_ready_tests.rs": "new\n",
    },
  }),
  passing("The job ignores a tests-only bullet heading", {
    base: withLog({
      "crates/rules/src/encounter/tests/delay_ready_tests.rs": "old\n",
    }),
    head: {
      "crates/rules/src/encounter/tests/delay_ready_tests.rs": "new\n",
      "CHANGELOG.md": `## [Unreleased]\n\n### Fixed\n${linkedBullet("old")}\n- tests only\n`,
    },
  }),
  failing(
    "A ready tests-only pull request naming an active own change fails archive-last",
    [ARCHIVE],
    {
      draft: false,
      body: "OpenSpec-Change: add-foo",
      base: withLog({
        "crates/rules/src/encounter/tests/delay_ready_tests.rs": "old\n",
        "openspec/changes/add-foo/proposal.md": "proposal\n",
      }),
      head: {
        "crates/rules/src/encounter/tests/delay_ready_tests.rs": "new\n",
      },
    },
  ),
  failing("A missing path uses C-locale git text", [MISSING], {
    gitStub: "locale",
    excludes: [ARCHIVE, "chemin inexistant"],
    base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
    remove: ["CHANGELOG.md"],
    head: { "crates/rules/src/lib.rs": "new\n" },
  }),
];

for (const row of cases) {
  test(row.name, async () => {
    await runCase(row);
  });
}

test("NUL-split diff includes both rename sides", async () => {
  const built = await buildFixture({
    base: withLog({ "docs/note.md": "same\n" }),
    rename: [{ from: "docs/note.md", to: "crates/rules/note.md" }],
  });
  const spyLog = join(built.dir, "spy.log");
  try {
    const result = spawnScript(built, { spyLog, body: "" });
    const output = combined(result);
    assert.equal(result.status, 1, output);
    assert.ok(output.includes(MISSING), output);
    const log = readFileSync(spyLog, "utf8");
    assert.match(log, /^merge-base /m);
    assert.match(log, /^diff -z /m);
    const diff = git(built.dir, [
      "diff",
      "-z",
      "--name-status",
      "--find-renames",
      built.branchPoint,
      built.headSha,
    ]).stdout;
    assert.ok(diff.includes("docs/note.md\0"), diff);
    assert.ok(diff.includes("crates/rules/note.md\0"), diff);
  } finally {
    await rm(built.dir, { recursive: true, force: true });
  }

  // The reverse rename fails only when the pre-image path is kept. Dropping
  // that side would leave a docs path and the bullet check would not run.
  await runCase(
    failing("reverse rename", [MISSING], {
      base: withLog({ "crates/rules/src/lib.rs": "same\n" }),
      rename: [{ from: "crates/rules/src/lib.rs", to: "docs/lib.rs" }],
    }),
  );
});

test("A missing base fails closed", async () => {
  await runCase({
    status: 1,
    omitBaseSha: true,
    includes: ["pr-hygiene: base sha is required"],
    excludes: [MISSING, ARCHIVE],
    base: { "docs/architect/decisions.md": "old\n" },
    head: { "docs/architect/decisions.md": "new\n" },
  });
  await runCase({
    status: 1,
    kind: "unrelated",
    includes: ["pr-hygiene: no merge-base"],
    excludes: [MISSING, ARCHIVE],
  });
});

test("A missing head parent fails closed", async () => {
  await runCase({
    status: 1,
    kind: "root-head",
    includes: ["pr-hygiene: head commit has no parent"],
    excludes: [MISSING, ARCHIVE],
  });
});

test("The check uses the event head", async () => {
  const built = await buildFixture({
    base: {
      "CHANGELOG.md": LOG,
      "crates/rules/src/lib.rs": "base\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    },
    main: {
      "CHANGELOG.md": MAIN_LOG,
      "crates/rules/src/lib.rs": "main\n",
    },
    commits: [
      {
        message: "implement",
        files: {
          "crates/rules/src/lib.rs": "pr\n",
          "CHANGELOG.md": MAIN_LOG,
        },
      },
      {
        message: "openspec archive add-foo --yes",
        rename: [archiveRename("add-foo")],
        files: { "openspec/specs/changelog/spec.md": "spec\n" },
      },
    ],
  });
  const spyLog = join(built.dir, "spy.log");
  try {
    const runnerSha = makeRunnerMerge(built);
    assert.notEqual(runnerSha, built.headSha);
    assert.notEqual(rev(built.dir), built.headSha);
    const mergeBase = git(built.dir, ["merge-base", built.baseSha, built.headSha]).stdout.trim();
    assert.equal(mergeBase, built.branchPoint);
    assert.notEqual(mergeBase, built.baseSha);

    const result = spawnScript(built, {
      spyLog,
      body: "OpenSpec-Change: add-foo",
      usePayloadForShas: true,
      usePayloadForBody: true,
      useGithubEventName: true,
    });
    const output = combined(result);
    assert.equal(result.status, 0, output);
    assert.ok(!output.includes(MISSING), output);
    assert.ok(!output.includes(ARCHIVE), output);

    const log = readFileSync(spyLog, "utf8");
    assert.ok(log.includes(`merge-base ${built.baseSha} ${built.headSha}`), log);
    assert.ok(log.includes(`diff -z --name-status --find-renames ${mergeBase} ${built.headSha}`), log);
    assert.ok(log.includes(`show ${mergeBase}:CHANGELOG.md`), log);
    assert.ok(!log.includes(`show ${built.baseSha}:CHANGELOG.md`), log);
    assert.ok(log.includes(`--format=%s ${built.headSha}`), log);
    assert.ok(!log.includes(runnerSha), log);
    assert.ok(!/^HEAD\b/m.test(log) && !log.includes(" HEAD"), log);
  } finally {
    await rm(built.dir, { recursive: true, force: true });
  }
});

test("main advanced after branching", async () => {
  const docs = await buildFixture({
    base: {
      "docs/architect/decisions.md": "old\n",
      "crates/rules/src/lib.rs": "base\n",
      "CHANGELOG.md": LOG,
    },
    main: {
      "crates/rules/src/lib.rs": "from main\n",
      "CHANGELOG.md": MAIN_LOG,
    },
    commits: [
      {
        message: "docs",
        files: { "docs/architect/decisions.md": "new\n" },
      },
    ],
  });
  try {
    const mergeBase = git(docs.dir, ["merge-base", docs.baseSha, docs.headSha]).stdout.trim();
    assert.equal(mergeBase, docs.branchPoint);
    const atMerge = git(docs.dir, ["show", `${mergeBase}:CHANGELOG.md`]).stdout;
    const atMain = git(docs.dir, ["show", `${docs.baseSha}:CHANGELOG.md`]).stdout;
    assert.equal(atMerge.includes(MAIN_BULLET), false);
    assert.equal(atMain.includes(MAIN_BULLET), true);
    const result = spawnScript(docs, { body: "" });
    const output = combined(result);
    assert.equal(result.status, 0, output);
    assert.ok(!output.includes(MISSING), output);
  } finally {
    await rm(docs.dir, { recursive: true, force: true });
  }

  // The same bullet text exists on main after the branch point. It counts as
  // added only when the base set is the changelog at the merge-base.
  const impl = await buildFixture({
    base: {
      "crates/rules/src/lib.rs": "base\n",
      "CHANGELOG.md": LOG,
    },
    main: {
      "crates/rules/src/lib.rs": "from main\n",
      "CHANGELOG.md": MAIN_LOG,
    },
    commits: [
      {
        message: "implement",
        files: {
          "crates/rules/src/lib.rs": "pr\n",
          "CHANGELOG.md": MAIN_LOG,
        },
      },
    ],
  });
  try {
    const mergeBase = git(impl.dir, ["merge-base", impl.baseSha, impl.headSha]).stdout.trim();
    const atMerge = git(impl.dir, ["show", `${mergeBase}:CHANGELOG.md`]).stdout;
    assert.equal(atMerge.includes(MAIN_BULLET), false);
    const result = spawnScript(impl, { body: "" });
    const output = combined(result);
    assert.equal(
      result.status,
      0,
      `a bullet added only on main must be excluded from the base set\n${output}`,
    );
  } finally {
    await rm(impl.dir, { recursive: true, force: true });
  }
});

test("A missing head changelog fails", async () => {
  await runCase(
    failing("missing file", [MISSING], {
      base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
      remove: ["CHANGELOG.md"],
      head: { "crates/rules/src/lib.rs": "new\n" },
    }),
  );
  await runCase(
    failing("missing heading", [MISSING], {
      base: withLog({ "crates/rules/src/lib.rs": "old\n" }),
      head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NO_HEADING },
    }),
  );
});

test("A missing base changelog leaves no bullets", async () => {
  await runCase(
    passing("missing file", {
      base: { "crates/rules/src/lib.rs": "old\n" },
      head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": NEW_LOG },
    }),
  );
  await runCase(
    passing("missing heading", {
      base: { "crates/rules/src/lib.rs": "old\n", "CHANGELOG.md": NO_HEADING },
      head: { "crates/rules/src/lib.rs": "new\n", "CHANGELOG.md": LOG },
    }),
  );
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

test("Job runs on its pull request types", () => {
  assert.match(workflow, /pull_request:/);
  assert.match(
    workflow,
    /types:\s*\[opened, synchronize, reopened, edited, ready_for_review, converted_to_draft]/,
  );
  assert.match(workflow, /branches:\s*\[main]/);
  assert.doesNotMatch(workflow, /merge_group/);
  const job = jobBlock(workflow, "pr-hygiene");
  assert.match(job, /node scripts\/pr-hygiene\.mjs/);
});

test("A fenced or extra-word line is not its own change", async () => {
  const shared = {
    draft: false,
    base: withLog({
      "crates/rules/src/lib.rs": "old\n",
      "openspec/changes/add-foo/proposal.md": "proposal\n",
    }),
    head: {
      "crates/rules/src/lib.rs": "new\n",
      "CHANGELOG.md": NEW_LOG,
    },
  };
  await runCase(
    passing("fenced", {
      ...shared,
      body: "```\nOpenSpec-Change: add-foo\n```",
    }),
  );
  await runCase(
    passing("extra words", {
      ...shared,
      body: "OpenSpec-Change: add-foo extra",
    }),
  );
});

test("The job does not run on push", () => {
  assert.doesNotMatch(workflow, /^push:/m);
  assert.doesNotMatch(workflow, /\n\s*push:/);
});

test("Permissions are contents read only", () => {
  assert.match(workflow, /permissions:\n  contents: read\n/);
  assert.doesNotMatch(workflow, /pull-requests:/);
  assert.doesNotMatch(workflow, /permissions:[\s\S]*contents: write/);
});

test("The job installs no npm package", () => {
  const job = jobBlock(workflow, "pr-hygiene");
  assert.doesNotMatch(job, /npm/);
  assert.doesNotMatch(workflow, /npm/);
  const specs = [...scriptSource.matchAll(/from\s+["']([^"']+)["']/g)].map((match) => match[1]);
  assert.ok(specs.length >= 4);
  for (const specifier of specs) {
    const sharedParser = specifier === "./spec-drift-check.mjs";
    assert.ok(specifier.startsWith("node:") || sharedParser, specifier);
  }
  assert.match(
    scriptSource,
    /import\s*\{\s*changeReferences\s*\}\s*from\s*["']\.\/spec-drift-check\.mjs["']/,
  );
  assert.doesNotMatch(scriptSource, /function changeReferences|function referenceId/);
  assert.doesNotMatch(scriptSource, /\brequire\s*\(/);
  const driftSource = readFileSync(
    fileURLToPath(new URL("./spec-drift-check.mjs", import.meta.url)),
    "utf8",
  );
  const driftSpecs = [...driftSource.matchAll(/from\s+["']([^"']+)["']/g)].map((match) => match[1]);
  for (const specifier of driftSpecs) {
    assert.match(specifier, /^node:/);
  }
});

test("The job runs the suite first", () => {
  const job = jobBlock(workflow, "pr-hygiene");
  const suite = job.indexOf("node --test scripts/pr-hygiene.test.mjs");
  const script = job.indexOf("node scripts/pr-hygiene.mjs");
  assert.ok(suite >= 0);
  assert.ok(script > suite);
  assert.match(job, /node-version:\s*["']?22["']?/);
  assert.match(job, /fetch-depth:\s*0/);
  assert.doesNotMatch(job, /continue-on-error/);
});

test("Cargo and spec-drift stay", () => {
  assert.match(ciWorkflow, /push:\n\s+branches: \[main]/);
  assert.match(ciWorkflow, /pull_request:\n\s+branches: \[main]/);
  assert.doesNotMatch(ciWorkflow, /edited/);
  assert.doesNotMatch(ciWorkflow, /\bif:/);
  const cargo = jobBlock(ciWorkflow, "ci");
  assert.match(cargo, /cargo fmt --check/);
  assert.match(cargo, /cargo clippy --workspace --all-targets -- -D warnings/);
  assert.match(cargo, /cargo build --workspace/);
  assert.match(cargo, /cargo test --workspace/);
  assert.doesNotMatch(cargo, /\bif:/);
  const drift = jobBlock(ciWorkflow, "spec-drift");
  assert.match(drift, /node --test scripts\/spec-drift-check\.test\.mjs/);
  assert.match(drift, /node scripts\/spec-drift-check\.mjs/);
  assert.doesNotMatch(drift, /\bif:/);
  assert.doesNotMatch(ciWorkflow, /pr-hygiene/);
  assert.match(editedWorkflow, /types:\s*\[edited]/);
  assert.match(editedWorkflow, /node scripts\/spec-drift-check\.mjs/);
  const editedJobs = editedWorkflow
    .slice(editedWorkflow.indexOf("jobs:"))
    .match(/^ {2}[a-z0-9-]+:$/gm);
  assert.deepEqual(editedJobs, ["  spec-drift:"]);
  assert.doesNotMatch(editedWorkflow, /pr-hygiene/);
  const hygieneJobs = workflow.slice(workflow.indexOf("jobs:")).match(/^ {2}[a-z0-9-]+:$/gm);
  assert.deepEqual(hygieneJobs, ["  pr-hygiene:"]);
});

test("An in-progress run is cancelled", () => {
  assert.match(
    workflow,
    /group: pr-hygiene-\$\{\{ github\.event\.pull_request\.number \}\}/,
  );
  assert.match(workflow, /cancel-in-progress:\s*true/);
  const jobs = workflow.slice(workflow.indexOf("jobs:")).match(/^ {2}[a-z0-9-]+:$/gm);
  assert.deepEqual(jobs, ["  pr-hygiene:"]);
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
  const built = await buildFixture(row);
  try {
    const result = spawnScript(built, row);
    const output = combined(result);
    assert.equal(result.status, row.status, output);
    for (const line of row.includes ?? []) {
      assert.ok(output.includes(line), `missing ${line}\n${output}`);
    }
    for (const line of row.excludes ?? []) {
      assert.ok(!output.includes(line), `unexpected ${line}\n${output}`);
    }
    if (row.assertOutput) {
      row.assertOutput(output);
    }
    if (row.assertRepo) {
      row.assertRepo(built.dir, built);
    }
  } finally {
    await rm(built.dir, { recursive: true, force: true });
  }
}

function combined(result) {
  return `${result.stdout ?? ""}${result.stderr ?? ""}`;
}

async function buildFixture(row) {
  if (row.kind === "unrelated") {
    return buildUnrelated();
  }
  if (row.kind === "root-head") {
    return buildRootHead();
  }
  const dir = await mkdtemp(join(tmpdir(), "pr-hygiene-"));
  initRepo(dir);
  await writeTree(dir, { "README.md": "base\n", ...(row.base ?? {}) });
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-m", "base"]);
  const branchPoint = rev(dir);
  let baseSha = branchPoint;
  if (row.main) {
    await writeTree(dir, row.main);
    git(dir, ["add", "-A"]);
    git(dir, ["commit", "-m", "main advances"]);
    baseSha = rev(dir);
    git(dir, ["checkout", "-b", "pr", branchPoint]);
  }
  const commits = row.commits ?? synthesizedCommits(row);
  for (const commit of commits) {
    await applyCommit(dir, commit);
  }
  return { dir, baseSha, headSha: rev(dir), branchPoint };
}

function synthesizedCommits(row) {
  const hasRename = (row.rename ?? []).length > 0;
  const hasRemove = (row.remove ?? []).length > 0;
  if (!row.head && !hasRename && !hasRemove) {
    return [];
  }
  return [
    {
      message: row.commitMessage ?? "head",
      files: row.head,
      remove: row.remove,
      rename: row.rename,
    },
  ];
}

async function buildUnrelated() {
  const dir = await mkdtemp(join(tmpdir(), "pr-hygiene-"));
  initRepo(dir);
  await writeTree(dir, { "README.md": "base\n", "docs/a.md": "base\n" });
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-m", "base"]);
  const baseSha = rev(dir);
  git(dir, ["checkout", "--orphan", "other"]);
  git(dir, ["rm", "-rf", "."]);
  await writeTree(dir, { "README.md": "head\n", "docs/a.md": "head\n" });
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-m", "other base"]);
  await writeTree(dir, { "docs/a.md": "head2\n" });
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-m", "other head"]);
  return { dir, baseSha, headSha: rev(dir), branchPoint: baseSha };
}

async function buildRootHead() {
  const dir = await mkdtemp(join(tmpdir(), "pr-hygiene-"));
  initRepo(dir);
  await writeTree(dir, { "README.md": "root\n" });
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-m", "root"]);
  const headSha = rev(dir);
  await writeTree(dir, { "README.md": "child\n" });
  git(dir, ["add", "-A"]);
  git(dir, ["commit", "-m", "child"]);
  return { dir, baseSha: rev(dir), headSha, branchPoint: headSha };
}

async function applyCommit(dir, commit) {
  for (const rel of commit.remove ?? []) {
    await rm(join(dir, rel), { recursive: true, force: true });
  }
  for (const { from, to } of commit.rename ?? []) {
    await mkdir(dirname(join(dir, to)), { recursive: true });
    git(dir, ["mv", from, to]);
  }
  if (commit.files) {
    await writeTree(dir, commit.files);
  }
  git(dir, ["add", "-A"]);
  const args = ["commit", "-m", commit.message ?? "head"];
  if (commit.empty) {
    args.push("--allow-empty");
  }
  git(dir, args);
}

function makeRunnerMerge(built) {
  git(built.dir, ["checkout", "-B", "runner-work", built.branchPoint]);
  writeFileSync(join(built.dir, "crates/rules/src/lib.rs"), "runner\n");
  git(built.dir, ["add", "-A"]);
  git(built.dir, ["rm", "-r", "openspec/changes/add-foo"]);
  git(built.dir, ["commit", "-m", "runner work"]);
  const tree = git(built.dir, ["rev-parse", "HEAD^{tree}"]).stdout.trim();
  const merge = git(built.dir, [
    "commit-tree",
    tree,
    "-p",
    built.baseSha,
    "-p",
    built.headSha,
    "-m",
    "Merge pull request",
  ]).stdout.trim();
  git(built.dir, ["checkout", "--detach", merge]);
  return merge;
}

function spawnScript(built, row) {
  const env = {
    PATH: process.env.PATH ?? "",
    HOME: process.env.HOME ?? "",
    LANG: process.env.LANG ?? "C",
    TMPDIR: process.env.TMPDIR ?? "/tmp",
    PR_HYGIENE_ROOT: built.dir,
  };
  if (row.useGithubEventName) {
    env.GITHUB_EVENT_NAME = row.event ?? "pull_request";
  } else {
    env.PR_HYGIENE_EVENT = row.event ?? "pull_request";
  }
  if (row.action !== undefined) {
    env.PR_HYGIENE_ACTION = row.action;
  } else {
    env.PR_HYGIENE_ACTION = "opened";
  }

  const payloadPath = join(built.dir, "event.json");
  if (row.invalidPayload) {
    writeFileSync(payloadPath, "{");
  } else {
    const pullRequest = {
      body: row.body ?? "",
      ...(row.draft !== undefined ? { draft: row.draft } : {}),
      labels: (row.labels ?? []).map((name) => ({ name })),
    };
    if (!row.omitNumber) {
      pullRequest.number = row.number ?? 7;
    }
    if (row.usePayloadForShas) {
      pullRequest.base = { sha: built.baseSha };
      pullRequest.head = { sha: built.headSha };
    }
    writeFileSync(
      payloadPath,
      JSON.stringify({
        action: row.action ?? "opened",
        pull_request: pullRequest,
      }),
    );
  }
  env.GITHUB_EVENT_PATH = payloadPath;

  if (!row.usePayloadForShas && !row.omitBaseSha) {
    env.PR_HYGIENE_BASE_SHA = built.baseSha;
  }
  if (!row.usePayloadForShas && !row.omitHeadSha) {
    env.PR_HYGIENE_HEAD_SHA = built.headSha;
  }
  if (!row.usePayloadForBody) {
    env.PR_HYGIENE_BODY = row.body ?? "";
  }

  if (row.gitStub) {
    const bin = join(built.dir, ".stub-bin");
    mkdirSync(bin);
    let body =
      "#!/bin/sh\nif [ \"$1\" = \"diff\" ]; then\n  echo \"diff failed\" >&2\n  exit 1\nfi\nexec \"$REAL_GIT\" \"$@\"\n";
    if (row.gitStub === "broken-nul") {
      body =
        "#!/bin/sh\nif [ \"$1\" = \"diff\" ]; then\n  printf 'M\\0'\n  exit 0\nfi\nexec \"$REAL_GIT\" \"$@\"\n";
    } else if (row.gitStub === "locale") {
      env.LC_ALL = "fr_FR.UTF-8";
      body =
        "#!/bin/sh\nif [ \"$LC_ALL\" != \"C\" ]; then\n  echo \"chemin inexistant\" >&2\n  exit 1\nfi\nexec \"$REAL_GIT\" \"$@\"\n";
    }
    writeFileSync(join(bin, "git"), body);
    chmodSync(join(bin, "git"), 0o755);
    env.REAL_GIT = realGit;
    env.PATH = `${bin}${delimiter}${env.PATH}`;
  } else if (row.spyLog) {
    const bin = join(built.dir, ".spy-bin");
    mkdirSync(bin);
    writeFileSync(
      join(bin, "git"),
      "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$GIT_SPY_LOG\"\nexec \"$REAL_GIT\" \"$@\"\n",
    );
    chmodSync(join(bin, "git"), 0o755);
    env.GIT_SPY_LOG = row.spyLog;
    env.REAL_GIT = realGit;
    env.PATH = `${bin}${delimiter}${env.PATH}`;
  }

  return spawnSync(process.execPath, [scriptPath], {
    cwd: built.dir,
    env,
    encoding: "utf8",
  });
}

function initRepo(dir) {
  git(dir, ["init", "-b", "main"]);
  git(dir, ["config", "user.email", "pr-hygiene@example.com"]);
  git(dir, ["config", "user.name", "PR Hygiene"]);
  git(dir, ["config", "commit.gpgsign", "false"]);
  git(dir, ["config", "core.hooksPath", "/dev/null"]);
}

function rev(dir) {
  return git(dir, ["rev-parse", "HEAD"]).stdout.trim();
}

function git(cwd, args) {
  const result = spawnSync(realGit, args, {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: "PR Hygiene",
      GIT_AUTHOR_EMAIL: "pr-hygiene@example.com",
      GIT_COMMITTER_NAME: "PR Hygiene",
      GIT_COMMITTER_EMAIL: "pr-hygiene@example.com",
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
