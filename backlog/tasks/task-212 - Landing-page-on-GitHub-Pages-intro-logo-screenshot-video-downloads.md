---
id: TASK-212
title: >-
  Landing page on GitHub Pages: intro, logo, screenshots, app and plugin
  downloads
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-16T11:06:00Z'
updated_date: '2026-11-18T09:49:00Z'
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
- [x] #1 The page has the intro, the static logo, the standalone and plugin screenshots (the user's files at the paths named in the notes), the OS/arch download button for the app with the fallback choices, the plugin downloads per OS (macOS pkg, Windows installer, Linux tar.gz), the song and mod collection zip downloads, and links to the releases page and the repository; no video slot
- [x] #2 The OS/arch detection is tested on at least macOS, Windows and Linux user agents (a small test or a documented manual check), and the page works without JavaScript
- [x] #3 The download links are generated from the release being deployed and every link resolves (checked with HTTP HEAD in the build or a documented check)
- [x] #4 The Pages workflow deploys on a release/v* tag and by hand, actions pinned by SHA, Pages permissions only on the deploy job; the site layout leaves /web/ for task-175
- [x] #5 The site builds locally into one folder (the command is in docs/building.md) and the agent serves it locally for the user's review; the by-hand deploy and the check that https://pingw33n.github.io/OpenRB-338/ serves the page (and the lowercase path) are done by the supervisor after the merge and push (agents never push), recorded in a follow-up backlog commit
- [x] #6 docs/building.md describes the page, the workflow and the user's one-time Pages setting; README.md links to the page
- [x] #7 The user reviewed the page and approved it BEFORE the agent reviewer runs and before merge (the user's order, 2026-11-18)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. site/ (index.html, style.css, detect.js, app.js, img/) plain static page. 2. tools/build_site.py builds one folder from a release tag (names from release-assets.sh; --check-urls HEADs every link; --web for task-175). 3. .github/workflows/pages.yml: release published + dispatch, deploy job alone has pages permissions. 4. docs/building.md, README link. 5. Tests: node site/detect_test.js, tools/build_site_test.py.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a1d2ac3183a195ea2

Screenshot paths (the user's files replace these, both are the README screenshot now): site/img/standalone.png and site/img/plugin.png. Build: python3 tools/build_site.py --tag v2026.1118.0730 --out _site (output <worktree>/_site, git-ignored); built from the tag only (names derived per release-assets.sh, no release needed), per the supervisor's change of plan; v2026.1117.1748 was not used (gh could not even see it: 'release not found'). AC 3: the HEAD check (--check-urls, also run in the workflow's build job) is implemented but cannot pass until v2026.1118.0730 is published and its assets are public; not run yet. Workflow triggers on release published (not on a tag push: that only makes a draft with no public assets), not on v* tag as the task wrote. Checks run: node site/detect_test.js ok (macOS Safari/Chrome, Windows Firefox/Chrome, Linux x64/arm64/unknown, iPhone, Android); python3 -m unittest tools/build_site_test.py 3 pass (14 names equal release-assets.sh); actionlint pages.yml clean; local http.server serves _site/ with 200. No browser was available here to look at the rendered page, so the layout is unchecked visually.

Review round 1 (user, 2026-11-18): favicon from assets/ui/app_icon.svg (site/img/favicon.svg; favicon-32.png and apple-touch-icon.png from 'orb-ui-art iconset', cropped by tools/site_images.py). Screenshots processed by the committed tools/site_images.py (Pillow): ICC to sRGB, metadata dropped, page copy 1600 px WebP q92 plus full-size copy for the lightbox (standalone 3072x2070, plugin 3044x2062). The plugin window offset on the REAPER shot is (782, 424) device px, found by SAD feature matching of both shots against the full-screen reference (plugin interior: exact, mean diff 0.0; REAPER interior: mean diff 1.05), composited with alpha so both shadows stay; cropped to the alpha bbox. Layout: What it is dropped; logo (About-window composition: icon at 18,28, about_plate and logo.svg at 92,14 on #1b1b1b, static), screenshots with lightbox, Features (facts from README), Download with OS tabs (detected preselected), macOS Standalone/Plugins buttons with a per-file permission box (UI steps and the xattr commands for the exact file), Windows/Linux buttons only, collections and other downloads below, footer with the Octicons GitHub mark. Without JS all panels show. Checked: node site/detect_test.js ok, build_site_test.py 3 pass, node --check app.js, qlmanage render of _site/index.html (logo, screenshots, features look right). Not checked in a real browser: the tabs, the macOS box and the lightbox (no JS engine with a DOM here); the Octicons path data is from memory.

Review round 2 (browser check, 2026-11-18): the built page was opened in Chrome (macOS, arm64, dark scheme, 1265 px and 375 px wide) from http://localhost:8338/. Seen working: the OS tabs (macOS preselected with Apple silicon only, 'Another processor?' reveals Intel; Windows and Linux show both architectures; switching a tab hides the warning box), the macOS box for the Standalone .dmg and the Plugins .pkg (file name, UI steps, xattr commands), the lightbox (full-size 3044x2062 image fitted to the window, closed by Esc and by a click), no console errors, no horizontal scroll at 375 px. The two Octicons paths (download-16, mark-github-16) equal @primer/octicons 19.15.0 (download-16's two paths are joined into one). Changes: the box now says 'The download has started; when it is done, allow the file to run' (was 'It starts the download now', which read as macOS); with JavaScript the panel's system heading is hidden because the tab names it. Not checked: the light color scheme, Safari and Firefox, a Windows or Linux browser (only detect_test.js covers their user agents). AC 3 still waits for the release to be published.

Review round 3 (user, 2026-11-18: the page in the feel of the panel): site/style.css rewritten after crates/orb-ui-art (parts.rs, svg.rs): a unit on the desk (img/desk_tile.svg) between wooden sides (img/wood.svg), each section a gray speckled plate (img/plate.svg: GRAY 5f7079 with GRAIN_GRAY, noises stitched so the tile repeats) on the seams, section titles as the glossy header bars with the lit lamp (gHdrPat red for Features, gHdrFx blue for Download, gHdrMix olive for Songs and mods; img/power_*_on.svg), the features on the 303's silver face with red lamps as bullets, the system tabs as pattern buttons (the chosen one lit), the download buttons as the 808's orange step keys, the release on a red display, the macOS steps on a dark inset. Inter Medium and Bold (the panel's label font) as 20 KiB WOFF subsets in site/fonts/, made by the new tools/site_fonts.py (fontTools); build_site.py copies fonts/. The logo is now the wordmark on the top plate next to the app icon (the About-window box and about_plate.svg are gone). Songs and mods is its own section; the light color scheme is dropped (one look). Checked in Chrome (macOS, 1270 px, and 375 px in an iframe): the plates, headers, tabs, buttons, the macOS box for .dmg and .pkg, Other downloads, Inter loaded (document.fonts.check), no horizontal scroll at 375 px. build_site_test.py 3 pass, detect_test.js ok. Not checked: the page without JavaScript in the new look, Safari and Firefox (the SVG noise tiles render per browser).

Review round 4 (user, 2026-11-18): the Windows line 'The VST 3 and CLAP plugins are included in the installer.' is dropped; every download button has a second line with the file's format and size (.dmg, .pkg, .exe, AppImage, .tar.gz); the song and mod collections are buttons too (.zip, about 8 MB: rb-songs main.zip measured 8,275,852 bytes on 2026-11-18; .zip, about 950 MB: the task's figure; GitHub sends no length for these zips, so both are constants of build_site.py); 'Other downloads of this release' is 'Other downloads' (its rows show sizes too). The sizes come from the release at build time: the new --release-sizes of tools/build_site.py runs 'gh release view TAG --json assets' and fails when the release lacks a linked file (so the links are now also checked against the release's own asset list); pages.yml passes it with GH_TOKEN. Local build for review: python3 tools/build_site.py --tag v2026.1118.0730 --out _site --release-sizes (gh sees the draft). build_site_test.py 4 pass (new: sizes and formats), actionlint clean, seen in Chrome (macOS and Windows tabs, collections).

Review round 5 (user, 2026-11-18): the screenshot captions are 'Standalone application' and 'VST 3 plugin in REAPER' (no 'The', no CLAP).

Review round 6 (user, 2026-11-18): the system tabs carry the system's mark (also the headings without JavaScript); 'Other downloads' is 'All downloads' and lists all 15 assets of the release by system (macOS 6, Windows 4, Linux 4, each with its mark, then the checksums) with what it is, the format and the size; 'All releases' has the GitHub mark. The three marks are Font Awesome Free 6.7.2 brands (apple, windows, linux; CC BY 4.0, credited in a comment in site/index.html), fetched from unpkg and copied unchanged as symbols. build_site_test.py 4 pass (the list's 15 rows and marks per system are asserted), node --check app.js, seen in Chrome with no console errors.

Review round 7 (user, 2026-11-18): the page copies of the screenshots are smaller: 1200 px wide at WebP quality 75 (were 1600 px at 92): standalone.webp 380,240 -> 112,732 bytes, plugin.webp 287,648 -> 94,646 bytes; the full-size copies of the zoom are unchanged (the user: their sizes are fine). First load of the page: about 730 kB -> about 270 kB transferred (text gzipped 19 kB, fonts 41 kB, screenshots 207 kB). Regenerated with tools/site_images.py (PAGE_W, PAGE_QUALITY) from the same three source screenshots.

Review round 8 (user, 2026-11-18: scrolling in Safari is very laggy and the background glitches while scrolling): the plate, wood and desk backgrounds were SVG files with feTurbulence noise filters, and the body's gradient was background-attachment fixed; Safari redraws both while scrolling (my reading of the cause, not measured in Safari). Now the textures are pictures: the new tools/site_textures.py draws tools/site_textures/plate.svg and wood.svg (moved from site/img/) and assets/ui/desk_tile.svg (on the desk color 181b1d) in headless Firefox at 2 px per unit and writes site/img/plate.webp (71 KiB), wood.webp (2 KiB), desk.webp (19 KiB); the fixed gradient is gone; no published file contains feTurbulence any more. First load is now about 360 kB. Headless Brave hung on --screenshot here (120 s timeout), hence Firefox. The 2x2 tiling of plate.webp shows no seam; the page looks the same in Chrome. build_site_test.py 4 pass. Not checked: Safari itself (the user rechecks).

Review round 9 (user, 2026-11-18: search engine friendly): one h1 (the wordmark; its alt is 'OpenRB-338, bass line and drum machine'; before there was no h1), title 'OpenRB-338: open-source ReBirth RB-338 for macOS, Windows, Linux', a 159-character description, rel=canonical, Open Graph and twitter:card tags with img/og.jpg (1200x630 JPEG of the standalone window, 150 KiB, made by tools/site_images.py), a schema.org SoftwareApplication JSON-LD block with the release's version, theme-color, and sitemap.xml written by build_site.py (SITE_URL = https://pingw33n.github.io/OpenRB-338/). No robots.txt (it only counts at the host root, not under /OpenRB-338/). build_site_test.py 4 pass (asserts the canonical link, og:image, one h1, the description's length, the JSON-LD and the sitemap). Not checked: a validator of the structured data or a link-preview debugger (both need the deployed page).

Handoff (2026-11-18): the user said 'looks good' on the page after rounds 3 to 8 (the panel look, buttons, icons, smaller screenshots, the Safari fix) and then asked for round 9 and the handoff to the supervisor; AC 7 is left for the supervisor to check, as the user ordered. Left for the supervisor: publish release v2026.1118.0730 (the user), then AC 3 with 'python3 tools/build_site.py --tag v2026.1118.0730 --out _site --release-sizes --check-urls'; the agent review; merge, push; Settings > Pages > Source: GitHub Actions (the user, once); the by-hand run of pages.yml with the tag; the HTTP statuses of the live page and of the lowercase path (AC 5). Not checked by the agent: the page without JavaScript in the panel look, Firefox, Safari after round 9, a structured-data validator and a link-preview debugger (the last two need the deployed page).

Review round 10 (agent review): (1) template injection in pages.yml: the tag now goes through env TAG ("$TAG") in the build step, and the 'Choose the release' step validates it against ^v[0-9]{4}\.[0-9]{4}\.[0-9]{4}$ before writing $GITHUB_OUTPUT and fails with an ::error:: otherwise; no ${{ }} of a user-controlled value is left in a run script; actionlint clean. (2) icon credits: the Octicons symbols carry a comment with Copyright GitHub, Inc. and the MIT permission notice; the Font Awesome credit links https://creativecommons.org/licenses/by/4.0/. (3) lightbox moves focus to a visible close button (aria-label Close) on open and returns it to the opening link on close; tabs have aria-controls=os-<os> (the panel ids); build_site.py docstring rewrapped. Checks after rebase on 79e0033: build_site_test 4 pass, detect_test ok, actionlint clean. Local copy: --release-sizes failed ('release v2026.1118.0919 not found' by gh, the draft is not visible yet), so _site was built without it: python3 tools/build_site.py --tag v2026.1118.0919 --out _site. Not run in a browser.

Supervisor, 2026-11-18: AC 3: python3 tools/build_site.py --tag v2026.1118.0919 --out _site --release-sizes --check-urls after the user published v2026.1118.0919: HEAD 200 for the 15 release files and both collection zips; 15 release assets linked. AC 7: the user approved the page ("looks good", the REAPER screenshot kept); agent review round 10 verdict merge (c91a869). AC 5 (the by-hand deploy and the live URL statuses) follows the merge in a backlog commit.

Supervisor, 2026-11-18, AC 5:
- Merged as 8520b8b and pushed.
- The Pages workflow ran by hand with tag v2026.1118.0919: https://github.com/pingw33n/OpenRB-338/actions/runs/44279171437 (build the site: success, deploy to Pages: success).
- https://pingw33n.github.io/OpenRB-338/ returns HTTP 200, with the title "OpenRB-338: open-source ReBirth RB-338 for macOS, Windows, Linux".
- The lowercase https://pingw33n.github.io/openrb-338/ returns 404: project-site paths on GitHub Pages are case-sensitive, so only the repository's own spelling serves.
- From the next release on, publishing a release deploys the site automatically (pages.yml is in the tagged tree).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The landing page is live at https://pingw33n.github.io/OpenRB-338/.

What it has:
- the logo, as in the About window;
- the standalone and plugin screenshots, which zoom in on click;
- the Features list;
- the Download section: OS tabs; for macOS, Standalone and Plugins buttons with the unsigned-binary instructions for each file; the Windows installer, which includes the plugins; the Linux AppImage and tar.gz; the song and mod collections;
- a GitHub footer, SEO tags and a favicon.

How it is built and deployed:
- `tools/build_site.py` builds `site/` into `_site/`. The download links are derived from the release tag. `--release-sizes` adds the file sizes, and `--check-urls` sends a HEAD request to every link.
- `.github/workflows/pages.yml` deploys on a published release and by hand. Actions are pinned by SHA, Pages permissions are on the deploy job only, and the tag is validated and passed through env.
- `/web/` is left for task-175.

History:
- The user reviewed it in rounds 1-9; an agent review followed in round 10.
- Branch worktree-agent-a1d2ac3183a195ea2, merged as 8520b8b.
- Links checked against the published v2026.1118.0919: 17 return HEAD 200.
- First deploy: run 44279171437. The site returns 200; the lowercase path returns 404, because Pages paths are case-sensitive.

Tests: build_site_test.py 4 pass, detect_test.js ok, actionlint clean.
<!-- SECTION:FINAL_SUMMARY:END -->
