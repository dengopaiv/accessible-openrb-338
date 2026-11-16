---
id: TASK-212
title: 'Landing page on GitHub Pages: intro, logo, screenshot, video, downloads'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T11:06:00Z'
labels:
  - docs
  - tools
milestone: m-6
dependencies: []
references:
  - README.md
  - assets/readme/screenshot.png
  - assets/ui/logo_open.svg
  - assets/ui/logo_rb338.svg
  - .github/workflows/release.yml
  - scripts/package.sh
  - docs/building.md
  - backlog task-175
  - backlog task-211
priority: medium
type: feature
ordinal: 190500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user wants a landing page for OpenRB-338 published with GitHub Pages at https://pingw33n.github.io/OpenRB-338/ (2026-11-16). The web version of the app (m-6, task-175) will live under https://pingw33n.github.io/OpenRB-338/web/ on the same Pages site. The user reviews the result before merge.

Content:
- **A short intro** to OpenRB-338 (what it is: an open reimplementation of ReBirth RB-338 2.01's engine, sample-exact with the original, plus a UI; standalone app, CLI; mods and songs), taken from README.md's facts, with no claim the project has not checked.
- **The logo without animation** (a static picture of the panel's wordmark: assets/ui/logo_*.svg or the README's logo frame, not the animated WebP).
- **A screenshot** (assets/readme/screenshot.png or a fresh one from the app's `--screenshot`).
- **A video** slot: task-211 (the user) makes the video and chooses its hosting; the page shows it when a URL is set in the page's config and shows nothing in its place until then.
- **Download buttons:**
  - one for the visitor's OS and architecture, from the browser (navigator.userAgentData where available, else the user agent), pointing at the matching asset of the latest GitHub release (the six targets of scripts/package.sh and the DMG of scripts/package-dmg.sh: macOS aarch64/x86_64, Windows, Linux AppImage/tarball). Browsers cannot always tell Apple Silicon from Intel or the Linux architecture: when unsure, show the two choices instead of guessing. Without JavaScript, the page still lists every download;
  - the song collection (github.com/pingw33n/rb-songs) and the mod collection (github.com/pingw33n/rb-mods) as zip downloads (GitHub's archive zip of each repository's default branch, e.g. https://github.com/pingw33n/rb-songs/archive/refs/heads/main.zip; state the mods' size, about 950 MB);
  - a link to all other downloads (the GitHub releases page) and to the GitHub repository.
- Release asset URLs: generate them when the site is built from the release being deployed (asset names from release.yml), or use the stable `releases/latest/download/NAME` form if the asset names carry no version; do not call the GitHub API from the visitor's browser unless needed (rate limits). Say which in docs/building.md.

Build and deploy:
- A static page (plain HTML/CSS, a little JavaScript for the OS detection; no framework needed), its source in the repository (e.g. `web/landing/` or `site/`).
- A GitHub Actions workflow that deploys the Pages site on each release tag v* and by hand (workflow_dispatch), with actions/upload-pages-artifact and actions/deploy-pages pinned by SHA, Pages permissions only on the deploy job. GitHub Pages deploys one artifact for the whole site, so the same workflow must later also put the web app under /web/ (task-175): build the site into one folder with the landing page at its root and leave a clear place for /web/.
- The user's one-time step (Settings > Pages > Source: GitHub Actions) goes into docs/building.md; until it is done the deploy fails with a clear message.
- README.md links to the landing page.
- Check after the first deploy that https://pingw33n.github.io/OpenRB-338/ serves the page (and whether the lowercase path does), with the HTTP statuses in the notes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The page has the intro, the static logo, the screenshot, the video slot (empty until task-211's URL is set), the OS/arch download button with the fallback choices, the song and mod collection zip downloads, and links to the releases page and the repository
- [ ] #2 The OS/arch detection is tested on at least macOS, Windows and Linux user agents (a small test or a documented manual check), and the page works without JavaScript
- [ ] #3 The Pages workflow deploys on a v* tag and by hand, actions pinned by SHA, Pages permissions only on the deploy job; the site layout leaves /web/ for task-175
- [ ] #4 A by-hand deploy ran (link in the notes) and https://pingw33n.github.io/OpenRB-338/ serves the page; the lowercase path checked
- [ ] #5 docs/building.md describes the page, the workflow and the user's one-time Pages setting; README.md links to the page
- [ ] #6 The user reviewed the page and approved it before merge
<!-- AC:END -->
