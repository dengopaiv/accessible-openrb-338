---
id: TASK-315
title: 'README and site: over 5,000 songs, new features, portable archive'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-26T08:30:00Z'
updated_date: '2026-11-26T08:44:00Z'
labels:
  - docs
milestone: m-5
dependencies: []
references:
  - README.md
  - site/index.html
  - site/app.js
priority: medium
type: docs
ordinal: 286700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked (2026-11-26) to bring the README and the landing page (site/, deployed with the next release, .github/workflows/pages.yml) up to date for the release.

**README.md** (Features, "Sounds exactly like the original", lines ~25-28): today it says "tested on 4,834 songs of the song collection, 2,693 of them on 135 third-party mods, and on a 175-render reference corpus". Use the user's wording "over 5,000 songs" for the song count (4,834 third-party songs plus the 175-render corpus). Keeping the mods clause is fine; keep the sentence short.

**site/index.html:**
1. The features list (line ~89) says "tested on 1,268 songs and a 175-render reference corpus". Make it say "tested on over 5,000 songs", with no mention of a corpus.
2. Add the new major features since the last release to the site's features list, in the page's existing style and length:
   - **Mod skin support:** a mod's original panel art can be used as the panel (View > Classic / Modern; see README "A classic theme"), and it is fully functional (fully playable, not a picture). The user's words: "mod skinning support (mention it's fully functional)".
   - **Pattern editors:** floating editors for the 303, 808 and 909 patterns (docs/ui_spec.md sections 23-25).
   - **Multiple output channels in the plugin:** per-device and per-drum outputs, set up in the plugin's Output Routing window (docs/plugin.md; docs/rewire.md).
   Check the README's features and docs for the exact facts; do not invent capabilities.
3. In the ALL DOWNLOADS section (it may be filled by site/app.js from release asset names), the Windows `.zip` entry must be described as "portable archive". Find where the label text comes from and change it there. Check site/detect_test.js still passes if it covers the labels (run it with node).

**Review:** the user reviews the site before it is committed. Serve site/ locally (e.g. `python3 -m http.server` from a scratch copy or the worktree's site/; note any {{PLACEHOLDER}} the deploy fills, and show it with plausible values if the page is unreadable otherwise, without committing that) and give the URL. Do not mark Done until the user approves.

User-visible text says 303/808/909, never TB-/TR-.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 README says the engine is tested on over 5,000 songs
- [x] #2 The site says tested on over 5,000 songs and does not mention a corpus
- [x] #3 The site's features list has mod skin support (stated as fully functional), the pattern editors, and the plugin's per-device and per-drum outputs
- [x] #4 The site's ALL DOWNLOADS entry for the Windows .zip reads portable archive
- [x] #5 The user reviewed the site served locally and approved it
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-af5db2bcc82799587 (uncommitted, awaiting the user's review of the site served locally). Label source: tools/build_site.py asset note 'archive, .zip' -> 'portable archive, .zip' (test updated). build_site_test.py OK, detect_test ok.

User review round 1 (2026-11-26): (1) plugin bullet text replaced with the user's exact wording, separate Multiple outputs bullet removed: done. (2) collection sizes (build_site.py SONGS_DETAIL 8 MB, MODS_DETAIL 950 MB): blocked, the sandbox refuses repository commands on the two collection checkouts from this isolated worktree, so cleanliness and commits were not confirmed; stopped as instructed.

Review round 1, item 2 (checks run by the supervisor): rb-songs clean at 40a6f4d (= origin/main), 4,834 .rbs, zip 30.1 MB; rb-mods clean at 0e465e8 (= origin/main), 324 .rbm, zip 1,019.7 MB. build_site.py SONGS_DETAIL 'about 8 MB' -> 'about 30 MB', MODS_DETAIL 'about 950 MB' -> 'about 1 GB'; build_site_test.py updated, passes. README plugin bullet left as is.

Review round 2 (2026-11-26): removed 'It is not a picture: the skin is fully functional and fully playable, with every control working.' from the mod-skin bullet (the bullet title still says fully functional). The user approved the site with that change and skipped another review. Tests: build_site_test.py OK, detect_test.js ok.

Review round 3 (reviewer send-back, 2026-11-26): DESCRIPTION in tools/build_site.py said two TB-303s, a TR-808, a TR-909 (meta, og, JSON-LD): now 'two 303s, an 808 and a 909'; no other TB-/TR- in site/ or tools/build_site*.py. Removed a trailing space before </li> at site/index.html:92. The earlier notes 'uncommitted, awaiting the user's review' and 'do not mark Done' are superseded: the user approved on 2026-11-26.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
README and site say tested on over 5,000 songs (site: no corpus mention). Site features: Mod skins, fully functional; Pattern editors; the plugin bullet now states per-device and per-drum outputs (22) and Options > Output Routing. Windows .zip note reads 'portable archive, .zip' (tools/build_site.py, test updated). Collection captions: about 30 MB songs, about 1 GB mods (checked on clean rb-songs 40a6f4d, rb-mods 0e465e8). User reviewed the served site and approved it (round 2). Branch worktree-agent-af5db2bcc82799587. Tests: build_site_test.py OK, node site/detect_test.js ok.
<!-- SECTION:FINAL_SUMMARY:END -->
