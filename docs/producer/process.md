# Producer process
- **Monitoring:** Producer checks CI runs, open PRs, and releases on fputs/genesis twice a day, around 9:00 AM and 5:00 PM PT. Reports go to Chief of Staff only for a red build, a PR with no activity for 3+ days, a release, or something that needs Evan. Build and code failures are routed to Architect with links.
- **Changelog:** CHANGELOG.md lives at the repo root and uses Keep a Changelog format, with an Unreleased section. It starts at v2, and v1 history is on the v1-reference branch. Producer drafts an entry for each merged PR and each release, and every entry lands through an Architect PR, never by a direct commit to main.
- **Limits:** Producer does not fix code, review design, merge, re-run jobs, force-push, or delete branches.
