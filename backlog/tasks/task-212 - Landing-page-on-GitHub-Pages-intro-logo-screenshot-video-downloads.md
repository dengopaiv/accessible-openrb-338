---
id: TASK-212
title: >-
  Landing page on GitHub Pages: intro, logo, screenshots, app and plugin
  downloads
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T11:06:00Z'
updated_date: '2026-11-18T07:28:00Z'
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
  - docs/plugin_usage.md
  - scripts/package-pkg.sh
  - scripts/release-assets.sh
priority: medium
type: feature
ordinal: 190500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user wants a landing page for OpenRB-338, published with GitHub Pages at https://pingw33n.github.io/OpenRB-338/ (2026-11-16). The web version of the app (m-6, task-175) will later live under https://pingw33n.github.io/OpenRB-338/web/ on the same Pages site. The user reviews the result before merge.

**Changed by the user on 2026-11-18:**
- No video for now: the page has no video slot.
- The page also offers the plugin downloads.
- The user supplies two screenshots, one of the standalone app and one of the plugin. Until they arrive, use the README screenshot (assets/readme/screenshot.png) as a placeholder for both, at the paths the page reads (for example `site/img/standalone.png` and `site/img/plugin.png`). Name those two paths in the task notes so the user's files can replace them before merge.

Content:
- **A short intro** to OpenRB-338: what it is, namely an open reimplementation of ReBirth RB-338 2.01's engine, sample-exact with the original, plus a UI; the standalone app, the VST3/CLAP instrument plugin and the CLI; mods and songs. Take the facts from README.md and make no claim the project has not checked.
- **The logo without animation:** a static picture of the panel's wordmark (assets/ui/logo_*.svg, or the README's logo frame), not the animated WebP.
- **Two screenshots:** the standalone app and the plugin (see above).
- **Download buttons for the app:**
  - One button for the visitor's OS and architecture, detected in the browser: navigator.userAgentData where available, else the user agent.
  - It points at the matching asset of the latest GitHub release: macOS DMG aarch64/x86_64, Windows setup.exe x64/arm64, Linux AppImage aarch64/x86_64.
  - Browsers cannot always tell Apple Silicon from Intel, or the Linux architecture. When unsure, show both choices instead of guessing.
  - Without JavaScript, the page still lists every download.
- **Download buttons for the plugin (VST3/CLAP)**, by OS and architecture in the same way:
  - **macOS:** `OpenRB-338-Plugins-<version>-<target>.pkg`, a separate installer.
  - **Windows:** the same setup.exe as the app. Its VST3 and CLAP components are selected by default, so say "included in the installer".
  - **Linux:** the `.tar.gz` package, whose `plugins/` folder holds the bundles and whose README.txt lists the plugin folders.
  - A short line on where the bundles go, from README.md's plugin section, and a link to docs/plugin_usage.md.
  - The plugin assets exist from release v2026.1117.1748 on. That release is a draft until the user publishes it; the latest published release, v2026.1113.1528, has no plugin assets. The page must not link to assets the deployed release lacks: generate the links from the release being deployed.
- **Other downloads:**
  - The song collection (github.com/pingw33n/rb-songs) and the mod collection (github.com/pingw33n/rb-mods) as zip downloads: GitHub's archive zip of each repository's default branch, for example https://github.com/pingw33n/rb-songs/archive/refs/heads/main.zip. State the mods' size, about 950 MB.
  - A link to all other downloads (the GitHub releases page) and one to the GitHub repository.
- **Release asset URLs:** the asset names carry the version (`OpenRB-338-<YYYY.MMDD.HHMM>-<target>...`). Generate the URLs when the site is built, from the release being deployed: take the asset names from the release, via gh or the API in the workflow. Do not call the GitHub API from the visitor's browser. Say which method is used in docs/building.md.

Build and deploy:
- A static page: plain HTML and CSS, with a little JavaScript for the OS detection and no framework. Its source lives in the repository (for example `site/`).
- A GitHub Actions workflow that deploys the Pages site on each published release (or `v*` tag) and by hand (workflow_dispatch):
  - actions/upload-pages-artifact and actions/deploy-pages, pinned by SHA;
  - Pages permissions on the deploy job only.
- GitHub Pages deploys one artifact for the whole site, so the same workflow must later also put the web app under /web/ (task-175). Build the site into one folder, with the landing page at its root and a clear place left for /web/.
- The user's one-time step (Settings > Pages > Source: GitHub Actions) goes into docs/building.md. Until it is done, the deploy fails with a clear message.
- README.md links to the landing page.
- After the first deploy, check that https://pingw33n.github.io/OpenRB-338/ serves the page, and whether the lowercase path does. Put the HTTP statuses in the notes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The page has the intro, the static logo, the standalone and plugin screenshots (the user's files at the paths named in the notes), the OS/arch download button for the app with the fallback choices, the plugin downloads per OS (macOS pkg, Windows installer, Linux tar.gz), the song and mod collection zip downloads, and links to the releases page and the repository; no video slot
- [ ] #2 The OS/arch detection is tested on at least macOS, Windows and Linux user agents (a small test or a documented manual check), and the page works without JavaScript
- [ ] #3 The download links are generated from the release being deployed and every link resolves (checked with HTTP HEAD in the build or a documented check)
- [ ] #4 The Pages workflow deploys on a release/v* tag and by hand, actions pinned by SHA, Pages permissions only on the deploy job; the site layout leaves /web/ for task-175
- [ ] #5 The site builds locally into one folder (the command is in docs/building.md) and the agent serves it locally for the user's review; the by-hand deploy and the check that https://pingw33n.github.io/OpenRB-338/ serves the page (and the lowercase path) are done by the supervisor after the merge and push (agents never push), recorded in a follow-up backlog commit
- [ ] #6 docs/building.md describes the page, the workflow and the user's one-time Pages setting; README.md links to the page
- [ ] #7 The user reviewed the page and approved it BEFORE the agent reviewer runs and before merge (the user's order, 2026-11-18)
<!-- AC:END -->
