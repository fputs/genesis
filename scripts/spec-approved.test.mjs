import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { mkdtemp, mkdir, rm, writeFile, chmod } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./spec-approved.mjs", import.meta.url));
const hygienePath = fileURLToPath(new URL("./pr-hygiene.mjs", import.meta.url));
const workflowPath = fileURLToPath(new URL("../.github/workflows/spec-approved.yml", import.meta.url));
const ownersPath = fileURLToPath(new URL("../.github/CODEOWNERS", import.meta.url));
const authorsPath = fileURLToPath(new URL("../openspec/spec-approved-authors.txt", import.meta.url));
const realGit = spawnSync("which", ["git"], { encoding: "utf8" }).stdout.trim();

const AUTHOR = 239839957;
const LIB = "crates/rules/src/lib.rs";
const SCRIPT_PATH = "scripts/spec-approved.mjs";
const WORKFLOW_PATH = ".github/workflows/spec-approved.yml";
const AUTHORS_PATH = "openspec/spec-approved-authors.txt";
const PROPOSAL = "# Proposal\n";
const SPEC = "# Spec\n";
const TASKS = "- [ ] 1.1 Open\n";
const HEX40 = "0123456789abcdef0123456789abcdef01234567";
const UPPER40 = "0123456789ABCDEF0123456789ABCDEF01234567";
const APPROVAL_PATHS = [
  "scripts/spec-approved.mjs",
  "scripts/spec-approved.test.mjs",
  "scripts/spec-drift-check.mjs",
  "scripts/pr-hygiene.mjs",
  "openspec/spec-approved-authors.txt",
];

function protectedTree(extra = {}) {
  return {
    "README.md": "base\n",
    [AUTHORS_PATH]: "239839957\n",
    [SCRIPT_PATH]: "base script\n",
    [WORKFLOW_PATH]: "jobs:\n  spec-approved:\n",
    ...extra,
  };
}

function changeTree(id, options = {}) {
  const root = options.archived
    ? `openspec/changes/archive/2026-10-23-${id}`
    : `openspec/changes/${id}`;
  const tree = {
    [`${root}/proposal.md`]: options.proposal ?? PROPOSAL,
    [`${root}/specs/aid/spec.md`]: options.spec ?? SPEC,
    [`${root}/tasks.md`]: options.tasks ?? TASKS,
  };
  if (options.design !== undefined) {
    tree[`${root}/design.md`] = options.design;
  }
  return tree;
}

function review(partial) {
  const submitted = partial.submitted_at ?? "2026-10-23T00:00:00Z";
  return {
    id: partial.id,
    user: { id: partial.userId ?? AUTHOR, login: partial.login ?? "fputs" },
    body: partial.body ?? "",
    state: partial.state ?? "APPROVED",
    submitted_at: submitted,
    updated_at: partial.updated_at ?? submitted,
    comments: partial.comments,
  };
}

function codePull(overrides = {}) {
  return {
    base: protectedTree({ [LIB]: "v1\n", ...changeTree("add-foo") }),
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [review({ id: 1, body: "spec-approved: {{head}}" })],
    status: 0,
    includes: [],
    excludes: [],
    ...overrides,
  };
}

const cases = [
  {
    name: "A head edit to the authors file is ignored",
    ...codePull({
      steps: [
        {
          id: "head",
          files: {
            [LIB]: "v2\n",
            [AUTHORS_PATH]: "239839957\n2\n",
          },
        },
      ],
      reviews: [review({ id: 2, userId: 2, body: "spec-approved: {{head}}" })],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "The head script is not executed",
    ...codePull({
      steps: [
        {
          id: "head",
          files: { [LIB]: "v2\n", [SCRIPT_PATH]: "process.exit(0)\n" },
        },
      ],
      reviews: [],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "This pull request bootstraps",
    base: { "README.md": "base\n", [LIB]: "v1\n" },
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", [SCRIPT_PATH]: "throw new Error('head script ran')\n" },
      },
    ],
    body: "",
    reviews: [],
    status: 0,
    includes: ["bootstrap: no base script"],
    excludes: ["spec-approved: partial base", "spec-approved: missing OpenSpec-Change"],
  },
  {
    name: "Exactly one base blob fails closed",
    base: {
      "README.md": "base\n",
      [AUTHORS_PATH]: "239839957\n",
      [SCRIPT_PATH]: "base script\n",
      [LIB]: "v1\n",
    },
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "",
    status: 1,
    includes: ["spec-approved: partial base"],
    excludes: ["bootstrap: no base script"],
  },
  {
    name: "A base with only the workflow fails closed",
    base: {
      "README.md": "base\n",
      [WORKFLOW_PATH]: "jobs:\n  spec-approved:\n",
      [LIB]: "v1\n",
    },
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "",
    status: 1,
    includes: ["spec-approved: partial base"],
    excludes: ["bootstrap: no base script"],
  },
  {
    name: "A docs-only pull request passes without a review",
    base: protectedTree({ "docs/architect/decisions.md": "old\n" }),
    steps: [{ id: "head", files: { "docs/architect/decisions.md": "new\n" } }],
    body: "",
    reviews: [],
    status: 0,
    includes: [],
    excludes: ["spec-approved:"],
  },
  {
    name: "A qualifying refactor skips spec-approved",
    base: protectedTree({ [LIB]: "v1\n", "CHANGELOG.md": "old\n" }),
    steps: [{ id: "head", files: { [LIB]: "v2\n", "CHANGELOG.md": "new\n" } }],
    body: "OpenSpec-Refactor: dice-and-checks\n",
    reviews: [],
    status: 0,
    includes: [],
    excludes: ["spec-approved: missing OpenSpec-Change"],
  },
  {
    name: "A conflicting refactor line does not skip spec-approved",
    ...codePull({
      body: "OpenSpec-Refactor: dice-and-checks\nOpenSpec-Change: add-foo\n",
      reviews: [],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "A code path with no change line fails",
    ...codePull({
      body: "",
      reviews: [],
      status: 1,
      includes: ["spec-approved: missing OpenSpec-Change"],
    }),
  },
  {
    name: "A protected path with no change line fails",
    base: protectedTree({ "scripts/pr-hygiene.mjs": "old\n" }),
    steps: [{ id: "head", files: { "scripts/pr-hygiene.mjs": "new\n" } }],
    body: "",
    reviews: [],
    status: 1,
    includes: ["spec-approved: missing OpenSpec-Change"],
    excludes: ["spec-approved: approval missing"],
  },
  {
    name: "No merge-base fails closed",
    base: protectedTree(),
    unrelatedBase: true,
    body: "",
    status: 1,
    includes: ["spec-approved: no merge-base"],
  },
  {
    name: "Deleting the approval files fails closed",
    base: protectedTree({
      "scripts/spec-approved.test.mjs": "test\n",
      "scripts/spec-drift-check.mjs": "drift\n",
      "scripts/pr-hygiene.mjs": "hygiene\n",
      ...changeTree("add-foo"),
    }),
    steps: [{ id: "head", remove: APPROVAL_PATHS }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [],
    status: 1,
    includes: ["spec-approved: approval missing"],
    excludes: ["bootstrap: no base script"],
  },
  {
    name: "Checking tasks does not change the digest",
    ...codePull({
      steps: [
        {
          id: "head",
          files: { [LIB]: "v2\n", "openspec/changes/add-foo/tasks.md": "- [x] 1.1 Open\n" },
        },
      ],
      reviews: [review({ id: 1, body: "spec-approved: {{base}}" })],
      compareDigest: ["base", "head", "add-foo"],
    }),
  },
  {
    name: "Archiving alone stays neutral",
    ...codePull({
      steps: [
        {
          id: "head",
          remove: ["openspec/changes/add-foo"],
          files: { [LIB]: "v2\n", ...changeTree("add-foo", { archived: true }) },
        },
      ],
      reviews: [review({ id: 1, body: "spec-approved: {{base}}" })],
      compareDigest: ["base", "head", "add-foo"],
    }),
  },
  {
    name: "A dated archive prefix does not match the id",
    base: protectedTree({
      [LIB]: "v1\n",
      "openspec/changes/archive/2026-10-23-xadd-foo/proposal.md": PROPOSAL,
      "openspec/changes/archive/2026-10-23-xadd-foo/specs/aid/spec.md": SPEC,
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [review({ id: 1, body: "spec-approved: {{head}}" })],
    status: 1,
    includes: ["spec-approved: unknown change add-foo"],
    excludes: ["spec-approved: ambiguous change path"],
  },
  {
    name: "A refactor line does not cover pr-hygiene.mjs",
    base: protectedTree({
      [LIB]: "v1\n",
      "scripts/pr-hygiene.mjs": "old\n",
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n", "scripts/pr-hygiene.mjs": "new\n" } }],
    body: "OpenSpec-Refactor: dice-and-checks\n",
    reviews: [],
    status: 1,
    includes: ["spec-approved: missing OpenSpec-Change"],
  },
  {
    name: "A refactor line does not cover Cargo.lock",
    base: protectedTree({
      [LIB]: "v1\n",
      "Cargo.lock": "old\n",
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n", "Cargo.lock": "new\n" } }],
    body: "OpenSpec-Refactor: dice-and-checks\n",
    reviews: [],
    status: 1,
    includes: ["spec-approved: missing OpenSpec-Change"],
  },
  {
    name: "The authors file stays on the approval path",
    base: protectedTree({ ...changeTree("add-foo") }),
    steps: [{ id: "head", files: { [AUTHORS_PATH]: "239839957\n2\n" } }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [],
    status: 1,
    includes: ["spec-approved: approval missing"],
    excludes: ["spec-approved: missing OpenSpec-Change"],
  },
  {
    name: "The spec-approved test stays on the approval path",
    base: protectedTree({
      "scripts/spec-approved.test.mjs": "old\n",
      ...changeTree("add-foo"),
    }),
    steps: [{ id: "head", files: { "scripts/spec-approved.test.mjs": "new\n" } }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [],
    status: 1,
    includes: ["spec-approved: approval missing"],
    excludes: ["spec-approved: missing OpenSpec-Change"],
  },
  {
    name: "Two copies of the change fail closed",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo"),
      ...changeTree("add-foo", { archived: true }),
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [review({ id: 1, body: "spec-approved: {{head}}" })],
    status: 1,
    includes: ["spec-approved: ambiguous change path"],
  },
  {
    name: "One sha covers two change ids",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo"),
      ...changeTree("add-bar"),
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "OpenSpec-Change: add-foo\nOpenSpec-Change: add-bar\n",
    reviews: [review({ id: 1, body: "spec-approved: {{base}}" })],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "Uppercase hex is invalid",
    ...codePull({
      reviews: [review({ id: 1, body: `spec-approved: ${UPPER40}` })],
      status: 1,
      includes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "A short sha is invalid",
    ...codePull({
      reviews: [review({ id: 1, body: "spec-approved: abcdef" })],
      status: 1,
      includes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "Wrong spacing is invalid",
    ...codePull({
      reviews: [review({ id: 1, body: `spec-approved:  ${HEX40}` })],
      status: 1,
      includes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "Trailing text is invalid",
    ...codePull({
      reviews: [review({ id: 1, body: `spec-approved: ${HEX40} thanks` })],
      status: 1,
      includes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "A malformed line does not fall back",
    ...codePull({
      reviews: [
        review({
          id: 1,
          body: "spec-approved: {{head}}",
          submitted_at: "2026-10-23T00:00:01Z",
        }),
        review({
          id: 2,
          body: "spec-approved: abcdef",
          submitted_at: "2026-10-23T00:00:02Z",
        }),
      ],
      status: 1,
      includes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "A tilde fence hides the line",
    ...codePull({
      reviews: [review({ id: 1, body: "~~~\nspec-approved: {{head}}\n~~~\n" })],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "Trimmed whitespace around an exact line is valid",
    ...codePull({
      reviews: [review({ id: 1, body: "  spec-approved: {{head}}" })],
    }),
  },
  {
    name: "A line that is not an attempt is missing",
    ...codePull({
      reviews: [review({ id: 1, body: "LGTM", state: "COMMENTED" })],
      status: 1,
      includes: ["spec-approved: approval missing"],
      excludes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "Two approval lines in one review are malformed",
    ...codePull({
      reviews: [
        review({ id: 1, body: "spec-approved: {{head}}\nspec-approved: {{head}}\n" }),
      ],
      status: 1,
      includes: ["spec-approved: approval malformed"],
    }),
  },
  {
    name: "A newer issue comment is ignored",
    ...codePull({
      reviews: [],
      pages: [
        {
          status: 200,
          reviews: [],
          issue_comment: "spec-approved: {{head}}",
          next: null,
        },
      ],
      issueComment: "spec-approved: {{head}}",
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "The pull request body does not approve",
    ...codePull({
      body: "OpenSpec-Change: add-foo\nspec-approved: {{head}}\n",
      reviews: [],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "An inline review comment does not approve",
    ...codePull({
      reviews: [
        review({
          id: 1,
          body: "LGTM",
          state: "COMMENTED",
          comments: [{ body: "spec-approved: {{head}}" }],
        }),
      ],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "The later submitted review wins",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo", { proposal: "v1\n" }),
    }),
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", "openspec/changes/add-foo/proposal.md": "v2\n" },
      },
    ],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [
      review({
        id: 1,
        body: "spec-approved: {{base}}",
        submitted_at: "2026-10-23T00:00:01Z",
      }),
      review({
        id: 2,
        body: "spec-approved: {{head}}",
        submitted_at: "2026-10-23T00:00:02Z",
      }),
    ],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "The review id breaks a submitted_at tie",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo", { proposal: "v1\n" }),
    }),
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", "openspec/changes/add-foo/proposal.md": "v2\n" },
      },
    ],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [
      review({
        id: 1,
        body: "spec-approved: {{base}}",
        submitted_at: "2026-10-23T00:00:01Z",
      }),
      review({
        id: 2,
        body: "spec-approved: {{head}}",
        submitted_at: "2026-10-23T00:00:01Z",
      }),
    ],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "Editing a review does not make it newer",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo", { proposal: "v1\n" }),
    }),
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", "openspec/changes/add-foo/proposal.md": "v2\n" },
      },
    ],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [
      review({
        id: 1,
        body: "spec-approved: {{base}}",
        submitted_at: "2026-10-23T00:00:01Z",
        updated_at: "2026-10-23T00:00:09Z",
      }),
      review({
        id: 2,
        body: "spec-approved: {{head}}",
        submitted_at: "2026-10-23T00:00:02Z",
        updated_at: "2026-10-23T00:00:02Z",
      }),
    ],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "A newer disallowed review does not displace",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo", { proposal: "v1\n" }),
    }),
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", "openspec/changes/add-foo/proposal.md": "v2\n" },
      },
    ],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [
      review({
        id: 1,
        body: "spec-approved: {{head}}",
        submitted_at: "2026-10-23T00:00:01Z",
      }),
      review({
        id: 2,
        userId: 2,
        body: "spec-approved: {{base}}",
        submitted_at: "2026-10-23T00:00:02Z",
      }),
    ],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "A later review with no approval line cancels",
    ...codePull({
      reviews: [
        review({
          id: 1,
          body: "spec-approved: {{head}}",
          submitted_at: "2026-10-23T00:00:01Z",
        }),
        review({
          id: 2,
          body: "LGTM",
          state: "CHANGES_REQUESTED",
          submitted_at: "2026-10-23T00:00:02Z",
        }),
      ],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "A truncated review page fails closed",
    ...codePull({
      pages: [
        {
          status: 200,
          reviews: [review({ id: 1, body: "spec-approved: {{head}}" })],
          next: 1,
        },
      ],
      status: 1,
      includes: ["spec-approved: reviews unavailable"],
    }),
  },
  {
    name: "A dismissed approval is missing",
    ...codePull({
      reviews: [review({ id: 1, body: "spec-approved: {{head}}", state: "DISMISSED" })],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "An older undismissed approval stays latest",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo", { proposal: "v1\n" }),
    }),
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", "openspec/changes/add-foo/proposal.md": "v2\n" },
      },
    ],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [
      review({
        id: 1,
        body: "spec-approved: {{head}}",
        submitted_at: "2026-10-23T00:00:01Z",
      }),
      review({
        id: 2,
        body: "spec-approved: {{base}}",
        state: "DISMISSED",
        submitted_at: "2026-10-23T00:00:02Z",
      }),
    ],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "A pending review is not the latest",
    base: protectedTree({
      [LIB]: "v1\n",
      ...changeTree("add-foo", { proposal: "v1\n" }),
    }),
    steps: [
      {
        id: "head",
        files: { [LIB]: "v2\n", "openspec/changes/add-foo/proposal.md": "v2\n" },
      },
    ],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [
      review({
        id: 1,
        body: "spec-approved: {{head}}",
        submitted_at: "2026-10-23T00:00:01Z",
      }),
      review({
        id: 2,
        body: "spec-approved: {{base}}",
        state: "PENDING",
        submitted_at: "2026-10-23T00:00:02Z",
      }),
    ],
    status: 0,
    includes: [],
    excludes: [],
  },
  {
    name: "An empty commented review leaves the approval",
    ...codePull({
      reviews: [
        review({
          id: 1,
          body: "spec-approved: {{head}}",
          submitted_at: "2026-10-23T00:00:01Z",
        }),
        review({
          id: 2,
          body: "",
          state: "COMMENTED",
          submitted_at: "2026-10-23T00:00:02Z",
        }),
      ],
    }),
  },
  {
    name: "A whitespace-only commented review leaves the approval",
    ...codePull({
      reviews: [
        review({
          id: 1,
          body: "spec-approved: {{head}}",
          submitted_at: "2026-10-23T00:00:01Z",
        }),
        review({
          id: 2,
          body: " \n\t \n",
          state: "COMMENTED",
          submitted_at: "2026-10-23T00:00:02Z",
        }),
      ],
    }),
  },
  {
    name: "The approved sha need not be the head",
    ...codePull({
      steps: [
        {
          id: "head",
          files: {
            [LIB]: "v2\n",
            "crates/rules/src/encounter/tests/new_tests.rs": "fn fresh() {}\n",
          },
        },
      ],
      reviews: [review({ id: 1, body: "spec-approved: {{base}}" })],
    }),
  },
  {
    name: "A rebase that drops the sha fails closed",
    ...codePull({
      reviews: [
        review({
          id: 1,
          body: "spec-approved: deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        }),
      ],
      status: 1,
      includes: ["spec-approved: approved sha unavailable"],
    }),
  },
  {
    name: "A login match does not approve",
    ...codePull({
      reviews: [
        review({ id: 1, userId: 42, login: "fputs", body: "spec-approved: {{head}}" }),
      ],
      status: 1,
      includes: ["spec-approved: approval missing"],
    }),
  },
  {
    name: "An empty authors file fails closed",
    base: protectedTree({
      [AUTHORS_PATH]: "# nobody\n\n",
      [LIB]: "v1\n",
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "",
    status: 1,
    includes: ["spec-approved: authors unavailable"],
  },
  {
    name: "An unknown change id fails closed",
    ...codePull({
      body: "OpenSpec-Change: missing-change\n",
      reviews: [],
      status: 1,
      includes: ["spec-approved: unknown change missing-change"],
    }),
  },
  {
    name: "A missing proposal fails closed",
    base: protectedTree({
      [LIB]: "v1\n",
      "openspec/changes/add-foo/tasks.md": TASKS,
    }),
    steps: [{ id: "head", files: { [LIB]: "v2\n" } }],
    body: "OpenSpec-Change: add-foo\n",
    reviews: [review({ id: 1, body: "spec-approved: {{head}}" })],
    status: 1,
    includes: ["spec-approved: proposal missing"],
  },
  {
    name: "An invalid change id fails closed",
    ...codePull({
      body: "OpenSpec-Change: Add-Foo\n",
      reviews: [],
      status: 1,
      includes: ["spec-approved: invalid change id Add-Foo"],
      excludes: ["spec-approved: missing OpenSpec-Change"],
    }),
  },
  {
    name: "Another scripts path does not require approval",
    base: protectedTree({ "scripts/other.mjs": "old\n" }),
    steps: [{ id: "head", files: { "scripts/other.mjs": "new\n" } }],
    body: "",
    reviews: [],
    status: 0,
    includes: [],
    excludes: ["spec-approved: missing OpenSpec-Change", "spec-approved: approval missing"],
  },
  {
    name: "Git children use LC_ALL=C",
    ...codePull({
      body: "",
      reviews: [],
      captureLocale: true,
      status: 0,
    }),
    // docs-only would also work; this code path has no change line and fails.
    // Override below so the locale probe is a passing docs diff.
  },
];

// The locale probe must be a passing run, or a git failure could skip the diff.
cases[cases.length - 1] = {
  name: "Git children use LC_ALL=C",
  base: protectedTree({ "docs/architect/decisions.md": "old\n" }),
  steps: [{ id: "head", files: { "docs/architect/decisions.md": "new\n" } }],
  body: "",
  reviews: [],
  captureLocale: true,
  status: 0,
  includes: [],
  excludes: [],
};

for (const row of cases) {
  test(row.name, async () => {
    await runJob(row);
  });
}

test("The base script step receives the GitHub token", () => {
  const job = jobBlock(readFileSync(workflowPath, "utf8"), "spec-approved");
  const marker = "name: Run the base script";
  const start = job.indexOf(marker);
  assert.ok(start >= 0, job);
  const after = job.slice(start);
  const next = after.slice(marker.length).search(/\n {6}- /);
  const step = next === -1 ? after : after.slice(0, marker.length + next);
  assert.match(step, /GITHUB_TOKEN:\s*\$\{\{\s*github\.token\s*\}\}/);
  assert.ok(step.includes('node "$script"'), step);
  assert.equal(step.includes("secrets."), false, step);
});

test("The job tests the base script first", () => {
  const workflow = readFileSync(workflowPath, "utf8");
  const job = jobBlock(workflow, "spec-approved");
  assert.match(job, /node-version:\s*["']?22["']?/);
  assert.equal(jobKeys(job).includes("name"), false, job);
  assert.equal(job.includes("matrix:"), false);
  const suite = job.indexOf('node --test "$RUNNER_TEMP/base/scripts/spec-approved.test.mjs"');
  const script = job.indexOf('node "$script"');
  assert.ok(suite >= 0, job);
  assert.ok(script > suite, job);
  assert.match(workflow, /permissions:\n  contents: read\n  pull-requests: read\n/);
  assert.equal(workflow.includes("secrets:"), false);
  const source = readFileSync(scriptPath, "utf8");
  assert.match(source, /changeReferences/);
  assert.match(source, /isCodePath/);
  assert.match(source, /unfencedBody/);
  assert.equal(source.includes("function unfencedBody"), false);
  assert.equal(source.includes("function isCodePath"), false);
  assert.equal(source.includes("from \"npm"), false);
});

test("A review re-runs the job", () => {
  const workflow = readFileSync(workflowPath, "utf8");
  assert.match(workflow, /pull_request_review:/);
  assert.match(workflow, /submitted/);
  assert.match(workflow, /edited/);
  assert.match(workflow, /dismissed/);
  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /opened/);
  assert.match(workflow, /synchronize/);
  assert.match(workflow, /reopened/);
  assert.match(workflow, /ready_for_review/);
  assert.match(workflow, /branches:\s*\[main]/);
});

test("An issue comment does not run the job", () => {
  const workflow = readFileSync(workflowPath, "utf8");
  assert.equal(workflow.includes("issue_comment"), false);
});

test("A push to main does not run the job", () => {
  const workflow = readFileSync(workflowPath, "utf8");
  assert.equal(/^push:/m.test(workflow), false);
  assert.equal(workflow.includes("\n  push:"), false);
});

test("A review checks out the pull request head", () => {
  const workflow = readFileSync(workflowPath, "utf8");
  assert.match(workflow, /ref:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/);
  assert.match(workflow, /fetch-depth:\s*0/);
  assert.match(workflow, /refs\/pull\/\$\{PR_NUMBER\}\/head/);
  assert.equal(workflow.includes("ref: main"), false);
});

test("A fork uses the same checkout", () => {
  const workflow = readFileSync(workflowPath, "utf8");
  assert.equal(workflow.includes("pull_request_target"), false);
  assert.equal(workflow.includes("secrets:"), false);
  assert.match(workflow, /contents: read/);
  assert.match(workflow, /pull-requests: read/);
  assert.match(workflow, /github\.event\.pull_request\.head\.sha/);
});

test("A failed pull request lookup fails closed", async () => {
  const workflow = readFileSync(workflowPath, "utf8");
  assert.match(workflow, /spec-approved: pull request unavailable/);
  await runJob({
    base: protectedTree(),
    blankHead: true,
    body: "",
    status: 1,
    includes: ["spec-approved: pull request unavailable"],
    excludes: [],
  });
});

test("The spec-approved job declares no concurrency", () => {
  const yaml = readFileSync(workflowPath, "utf8");
  const job = jobBlock(yaml, "spec-approved");
  const header = yaml.slice(0, yaml.indexOf("jobs:"));
  assert.equal(job.includes("concurrency:"), false);
  assert.equal(header.includes("concurrency:"), false);
  assert.equal(yaml.includes("cancel-in-progress"), false);
  assert.equal(yaml.includes("queue:"), false);
});

test("The digest is sha-256 of the canonical bytes", async () => {
  const { specDigest } = await import("./spec-approved.mjs");
  const proposal = Buffer.from("proposal-bytes");
  const spec = Buffer.from("spec-bytes");
  const expected = canonicalDigest([
    ["proposal.md", proposal],
    ["specs/aid/spec.md", spec],
  ]);
  const actual = specDigest([
    { path: "specs/aid/spec.md", bytes: spec },
    { path: "proposal.md", bytes: proposal },
  ]);
  assert.equal(actual, expected);
  assert.match(actual, /^[0-9a-f]{64}$/);
});

test("Editing a spec file mismatches", async () => {
  const edits = [
    { file: "openspec/changes/add-foo/proposal.md", next: "edited proposal\n" },
    { file: "openspec/changes/add-foo/design.md", next: "edited design\n", design: "old design\n" },
    { file: "openspec/changes/add-foo/specs/aid/spec.md", next: "edited spec\n" },
  ];
  for (const edit of edits) {
    await runJob({
      base: protectedTree({
        [LIB]: "v1\n",
        ...changeTree("add-foo", { design: edit.design }),
      }),
      steps: [{ id: "head", files: { [LIB]: "v2\n", [edit.file]: edit.next } }],
      body: "OpenSpec-Change: add-foo\n",
      reviews: [review({ id: 1, body: "spec-approved: {{base}}" })],
      status: 1,
      includes: ["spec-approved: spec hash mismatch"],
      excludes: [],
    });
  }
});

test("Importing pr-hygiene has no side effects", () => {
  const source = readFileSync(hygienePath, "utf8");
  assert.match(source, /runningAsScript/);
  assert.match(source, /export function unfencedBody/);
  const result = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `
        import { unfencedBody } from ${JSON.stringify(hygienePath)};
        if (typeof unfencedBody !== "function") process.exit(2);
        const hidden = unfencedBody("~~~\\nspec-approved: abc\\n~~~\\nvisible\\n");
        if (hidden.includes("spec-approved")) process.exit(3);
        if (!hidden.includes("visible")) process.exit(4);
        if (process.exitCode) process.exit(5);
      `,
    ],
    {
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_EVENT_PATH: "/no/such/spec-approved-event.json",
        PR_HYGIENE_EVENT: "push",
        PR_HYGIENE_BASE_SHA: "",
        PR_HYGIENE_HEAD_SHA: "",
      },
    },
  );
  assert.equal(result.status, 0, `${result.stdout ?? ""}\n${result.stderr ?? ""}`);
});

test("The shipped file allows one user id", () => {
  const ids = [];
  for (const raw of readFileSync(authorsPath, "utf8").split("\n")) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    if (line.trim() === "" || /^\s*#/.test(line)) {
      continue;
    }
    ids.push(line.trim());
  }
  assert.deepEqual(ids, ["239839957"]);
});

test("A missing token fails closed", async () => {
  await runJob({
    ...codePull({ reviews: [], status: 1, includes: ["spec-approved: reviews unavailable"] }),
    omitPages: true,
  });
});

test("The check asks for per_page=100", () => {
  assert.match(readFileSync(scriptPath, "utf8"), /per_page=100/);
  assert.match(readFileSync(scriptPath, "utf8"), /merge-base", "--is-ancestor"/);
});

test("CODEOWNERS gives the spec paths to fputs", () => {
  const lines = ownerLines();
  for (const path of ["/openspec/", "/AGENTS.md"]) {
    const matches = lines.filter((line) => line.startsWith(`${path} `) || line.startsWith(`${path}\t`));
    assert.equal(matches.length, 1, matches.join("\n"));
    assert.deepEqual(matches[0].trim().split(/\s+/).slice(1), ["@fputs"]);
  }
});

test("CODEOWNERS gives workflow and script paths to fputs", () => {
  const lines = ownerLines();
  for (const path of ["/.github/", "/scripts/"]) {
    const matches = lines.filter((line) => line.startsWith(`${path} `) || line.startsWith(`${path}\t`));
    assert.equal(matches.length, 1, matches.join("\n"));
    assert.deepEqual(matches[0].trim().split(/\s+/).slice(1), ["@fputs"]);
  }
});

test("Each case is a fixture repository", async () => {
  assert.ok(cases.length > 0);
  const names = cases.map((row) => row.name);
  assert.equal(new Set(names).size, names.length);
  for (const row of cases) {
    assert.equal(typeof row.status, "number");
    assert.ok(Array.isArray(row.includes));
    await runJob(row);
  }
});

function ownerLines() {
  return readFileSync(ownersPath, "utf8")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));
}

function canonicalDigest(pairs) {
  const chunks = [];
  for (const [path, bytes] of pairs) {
    const length = Buffer.alloc(8);
    length.writeBigUInt64BE(BigInt(bytes.length));
    chunks.push(Buffer.from(path), Buffer.from([0]), length, bytes);
  }
  return createHash("sha256").update(Buffer.concat(chunks)).digest("hex");
}

function jobKeys(job) {
  return job
    .split("\n")
    .filter((line) => /^ {4}[A-Za-z0-9_-]+:/.test(line))
    .map((line) => line.trim().replace(/:.*/, ""));
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

function fillValue(value, shas) {
  if (typeof value === "string") {
    return value.replace(/\{\{(\w+)\}\}/g, (_, key) => {
      if (!shas[key]) {
        throw new Error(`missing sha ${key}`);
      }
      return shas[key];
    });
  }
  if (Array.isArray(value)) {
    return value.map((item) => fillValue(item, shas));
  }
  if (value && typeof value === "object") {
    const filled = {};
    for (const [key, item] of Object.entries(value)) {
      filled[key] = fillValue(item, shas);
    }
    return filled;
  }
  return value;
}

async function runJob(row) {
  const dir = await mkdtemp(join(tmpdir(), "spec-approved-"));
  try {
    git(dir, ["init", "-b", "main"]);
    git(dir, ["config", "user.email", "spec-approved@example.com"]);
    git(dir, ["config", "user.name", "Spec Approved"]);
    git(dir, ["config", "commit.gpgsign", "false"]);
    git(dir, ["config", "core.hooksPath", "/dev/null"]);
    const shas = {};
    await writeTree(dir, row.base ?? { "README.md": "base\n" });
    git(dir, ["add", "-A"]);
    git(dir, ["commit", "-m", "base"]);
    shas.base = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
    for (const step of row.steps ?? []) {
      for (const rel of step.remove ?? []) {
        await rm(join(dir, rel), { recursive: true, force: true });
      }
      if (step.files) {
        await writeTree(dir, step.files);
      }
      git(dir, ["add", "-A"]);
      git(dir, ["commit", "-m", step.message ?? step.id]);
      shas[step.id] = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
    }
    let headSha = shas[row.head ?? "head"] ?? shas.base;
    if (row.unrelatedBase) {
      git(dir, ["checkout", "--orphan", "unrelated"]);
      git(dir, ["rm", "-rf", "."]);
      await writeTree(dir, { "unrelated.txt": "unrelated\n" });
      git(dir, ["add", "-A"]);
      git(dir, ["commit", "-m", "unrelated"]);
      headSha = git(dir, ["rev-parse", "HEAD"]).stdout.trim();
    } else {
      git(dir, ["checkout", "--detach", headSha]);
    }

    if (row.compareDigest) {
      const { digestChange } = await import("./spec-approved.mjs");
      const [left, right, id] = row.compareDigest;
      assert.equal(digestChange(dir, shas[left], id), digestChange(dir, shas[right], id));
    }

    const pages = row.pages
      ? fillValue(row.pages, shas)
      : [{ status: 200, reviews: fillValue(row.reviews ?? [], shas), next: null }];
    const bin = join(dir, ".stub-bin");
    const localeLog = join(dir, "git-locale.txt");
    if (row.captureLocale) {
      await mkdir(bin);
      const gitStub = join(bin, "git");
      await writeFile(
        gitStub,
        "#!/bin/sh\nprintf '%s\\n' \"$LC_ALL\" >> \"$GIT_LOCALE_LOG\"\nexec \"$REAL_GIT\" \"$@\"\n",
      );
      await chmod(gitStub, 0o755);
    }
    const env = { ...process.env };
    delete env.GITHUB_TOKEN;
    delete env.GH_TOKEN;
    delete env.GITHUB_EVENT_PATH;
    env.PATH = row.captureLocale ? `${bin}${delimiter}${env.PATH ?? ""}` : env.PATH;
    env.SPEC_APPROVED_ROOT = dir;
    env.SPEC_APPROVED_BASE_SHA = shas.base;
    env.SPEC_APPROVED_HEAD_SHA = row.blankHead ? "" : headSha;
    env.SPEC_APPROVED_BODY = fillValue(row.body ?? "", shas);
    if (!row.omitPages) {
      env.SPEC_APPROVED_REVIEW_PAGES = JSON.stringify(pages);
    } else {
      delete env.SPEC_APPROVED_REVIEW_PAGES;
    }
    if (row.issueComment) {
      env.SPEC_APPROVED_ISSUE_COMMENT = fillValue(row.issueComment, shas);
    }
    if (row.captureLocale) {
      env.LC_ALL = "fr_FR.UTF-8";
      env.GIT_LOCALE_LOG = localeLog;
      env.REAL_GIT = realGit;
    }
    const result = spawnSync(process.execPath, [scriptPath], {
      cwd: dir,
      env,
      encoding: "utf8",
    });
    const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
    assert.equal(result.status, row.status, output);
    for (const line of row.includes ?? []) {
      assert.ok(output.includes(line), `missing ${line}\n${output}`);
    }
    for (const line of row.excludes ?? []) {
      assert.ok(!output.includes(line), `unexpected ${line}\n${output}`);
    }
    if (row.captureLocale) {
      const logged = readFileSync(localeLog, "utf8").trim().split("\n").filter(Boolean);
      assert.ok(logged.length > 0, output);
      for (const line of logged) {
        assert.equal(line, "C");
      }
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
      GIT_AUTHOR_NAME: "Spec Approved",
      GIT_AUTHOR_EMAIL: "spec-approved@example.com",
      GIT_COMMITTER_NAME: "Spec Approved",
      GIT_COMMITTER_EMAIL: "spec-approved@example.com",
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
