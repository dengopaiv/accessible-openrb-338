---
id: TASK-273
title: >-
  Site and README: the .dmg warning blames the image, not the app; Features gain
  the plugin's outputs and the classic theme
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-21T13:03:00Z'
updated_date: '2026-11-21T13:34:00Z'
labels:
  - docs
  - tools
milestone: m-5
dependencies: []
references:
  - site/app.js
  - docs/building.md
  - README.md
priority: low
type: bug
ordinal: 258500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21. After a click on a macOS **.dmg** download button, the site's warning box (site/app.js, the "macOS warning" click handler, around lines 72-108) says:

> **macOS will not open OpenRB-338-…-aarch64-apple-darwin.dmg at first**
> OpenRB-338 is not signed or notarized with an Apple developer certificate, so macOS refuses to open it the first time. The download has started; when it is done, allow the file to run, either in the interface or in Terminal.

That's wrong for the .dmg: the disk image opens fine. What macOS refuses is the **app inside it**, OpenRB-338.app, the first time it's started after being copied to Applications. The steps below the text are already right: they open the image, copy the app, refuse, Open Anyway, and `xattr` on the app; the `xattr` on the .dmg was removed on 2026-11-20. Only the heading and the paragraph are wrong.

**Fix (the .dmg case only; the .pkg's text is right, since macOS does refuse to open an unsigned .pkg):**
- **Heading:** e.g. "macOS will not start OpenRB-338.app at first".
- **Paragraph:** the image opens normally. macOS refuses the app copied from it the first time, because it is not signed or notarized. The download has started; when it's done, open the image, copy the app, and allow it to run, either in the interface or in Terminal.
- Keep the .pkg branch as it is. Check the wording in docs/building.md (the site section, around "Clicking a macOS button starts the download and shows the box") and README.md's Quick start for the same mistake, and fix it there too.
- Run `node site/detect_test.js` and `node --check site/app.js`.

The supervisor deploys the site (pages.yml, by hand) after the merge, at the user's word.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 For a .dmg the warning's heading and paragraph say the image opens and the app inside is what macOS refuses at first; the .pkg text is unchanged
- [x] #2 docs/building.md and README.md say the same where they describe it
- [x] #3 node site/detect_test.js passes and node --check site/app.js is clean
- [x] #4 The Features sections of site/index.html (section#features) and README.md (## Features) mention the plugin's per-device outputs (303-1, 303-2, 808, 909 beside the Mix, tapped like ReBirth's ReWire channels after each channel's inserts and before its level and pan; docs/plugin.md 13, docs/rewire.md) and the classic theme (View > Classic / Modern in the standalone: the loaded mod's original panel art, fully playable; docs/ui_spec.md 21), in the sections' existing style and length
- [x] #5 The user reviewed the changed site (a local preview: tools/build_site.py into a scratch folder, opened in a browser) and README before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. site/app.js: .dmg heading and paragraph name the app, not the image; .pkg branch unchanged.
2. README Quick start and docs/building.md: check/fix the same wording.
3. Features of site/index.html and README.md: plugin per-device outputs, classic theme; README Not implemented: Mod themes line becomes partial.
4. Preview with build_site.py into the scratchpad; run node/python tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
User, 2026-11-21: also update the Features sections (site and README) for the plugin's multi-channel output (task-256) and the classic theme support (tasks 266-268). The user reviews the result first.

Branch: worktree-agent-a7d4a4d157385c3d8

Done in the working tree: app.js .dmg branch (heading "macOS will not start OpenRB-338.app at first", paragraph says the image opens, the app is refused); README Quick start macOS line; docs/building.md already said it right (xattr on the app, image needs none), unchanged. Features: site (the 303s/808/909 item gets the classic theme; the plugin item gets the outputs) and README (new classic-theme and plugin items); README Not implemented "Mod themes" reworded, since the classic panel now exists. Tests: node site/detect_test.js ok, node --check ok, build_site_test.py OK. Preview built with tag v2026.1118.0919 (latest published release) into the scratchpad site folder. AC 5 waits for the user.

User review, 2026-11-21: 'good' on the site preview (tag v2026.1118.0919) and the README, the Not implemented wording kept as written.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The .dmg warning now says the image opens and macOS refuses the app copied from it (the .pkg text unchanged); README Quick start says the same; docs/building.md was already right. Features of the site and README gained the plugin's per-device outputs and the classic theme; README's Not implemented line on mod themes was reworded. Branch worktree-agent-a7d4a4d157385c3d8, one commit. Tests: node site/detect_test.js, node --check site/app.js, build_site_test.py pass; the user reviewed the local preview and the README.
<!-- SECTION:FINAL_SUMMARY:END -->
