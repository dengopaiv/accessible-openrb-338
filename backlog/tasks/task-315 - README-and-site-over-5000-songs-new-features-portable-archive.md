---
id: TASK-315
title: 'README and site: over 5,000 songs, new features, portable archive'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-26T08:30:00Z'
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
- [ ] #1 README says the engine is tested on over 5,000 songs
- [ ] #2 The site says tested on over 5,000 songs and does not mention a corpus
- [ ] #3 The site's features list has mod skin support (stated as fully functional), the pattern editors, and the plugin's per-device and per-drum outputs
- [ ] #4 The site's ALL DOWNLOADS entry for the Windows .zip reads portable archive
- [ ] #5 The user reviewed the site served locally and approved it
<!-- AC:END -->
