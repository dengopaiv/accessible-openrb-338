---
id: TASK-175
title: Deploy the web version to GitHub Pages under /web/ from CI on each release tag
status: To Do
assignee:
  - '@agent-worker'
created_date: 2026-11-03 09:17
updated_date: '2026-11-16T11:07:00Z'
labels:
  - tools
  - docs
milestone: m-6
dependencies:
  - TASK-172
  - TASK-173
  - TASK-174
  - TASK-212
references:
  - .github/workflows/release.yml
  - .github/workflows/ci.yml
  - scripts/build-web.sh
  - docs/building.md
  - README.md
  - backlog task-212
priority: medium
type: feature
ordinal: 191000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the m-6 goal (the user, 2026-11-03) is the web version hosted on GitHub. Its address is https://pingw33n.github.io/OpenRB-338/web/ (the user, 2026-11-16): the root https://pingw33n.github.io/OpenRB-338/ is the landing page (task-212). The repository is public; it was renamed from openrb-338 to OpenRB-338 on 2026-11-12. task-172 builds the static site with scripts/build-web.sh; task-173 builds index.json of the two collections; this task publishes both under /web/.

What to build:
- **The same Pages workflow as the landing page.** GitHub Pages deploys one artifact for the whole site, so extend task-212's workflow (it runs on a release tag v* and by hand, workflow_dispatch) rather than adding a second deploying workflow: build dist/web/ with the tag's version (SOURCE_DATE_EPOCH from scripts/version.sh --epoch, as release.yml does) into the site folder's `web/` directory, next to the landing page at the root.
- The web app must work from the `/OpenRB-338/web/` base path: relative asset URLs (or a configured base) for the wasm, JS and index.json; check that deep links and reloads work there.
- Shallow-clone pingw33n/rb-songs and pingw33n/rb-mods at their main HEAD (the mods are about 950 MB; use a blobless or sparse clone if it is faster), run `orb web-index` with the two commits, and put index.json in `web/` next to the app.
- Actions pinned by SHA; permissions pages: write, id-token: write only on the deploy job.
- A `collections` input on workflow_dispatch is not needed; a redeploy by hand picks up new collection commits.
- The user's one-time Pages setting (Settings > Pages > Source: GitHub Actions) is documented by task-212.
- README.md and the landing page link to the web version at https://pingw33n.github.io/OpenRB-338/web/; the About box's links stay as they are.
- docs/building.md describes the web part of the workflow, how to redeploy after a collection changes, and the size of the site.

- The Pages address follows the repository name's case. task-212 checks the root's exact-case and lowercase forms after its first deploy; here check https://pingw33n.github.io/OpenRB-338/web/ and https://pingw33n.github.io/openrb-338/web/ (and a deep link such as web/index.json under each), record the HTTP status of each in the notes, and link only the form that works (the exact case) everywhere.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The Pages workflow (task-212's) also builds and deploys the web version under /web/ on a v* tag and by hand; a by-hand run (or a dry run with the deploy step skipped) is linked in the notes, and the landing page at the root is unaffected
- [ ] #2 The deployed web/index.json pins the rb-songs and rb-mods commits the run cloned, and the About box shows the tag's version
- [ ] #3 Actions are pinned by SHA and the Pages permissions are limited to the deploy job
- [ ] #4 README.md and the landing page link to https://pingw33n.github.io/OpenRB-338/web/ (the repository name's case); the exact-case and lowercase /web/ addresses were checked after the first deploy, with the results in the notes; docs/building.md covers the web part of the workflow and the redeploy
<!-- AC:END -->
