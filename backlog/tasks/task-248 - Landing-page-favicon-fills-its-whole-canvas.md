---
id: TASK-248
title: 'Landing page: favicon fills its whole canvas'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-18T13:15:00Z'
updated_date: '2026-11-18T13:17:00Z'
labels:
  - tools
milestone: m-6
dependencies: []
references:
  - site/img/favicon.svg
  - site/img/favicon-32.png
  - site/index.html
  - tools/site_images.py
  - docs/building.md
  - assets/ui/app_icon.svg
  - task-212
priority: medium
type: enhancement
ordinal: 190700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user (2026-11-18) asked that the landing page's favicon be as large as possible. Today `site/img/favicon.svg` is a copy of the app icon `assets/ui/app_icon.svg`: a 64x64 viewBox whose rounded tile is only 52x52 at (6,6), so the browser tab shows a 6 px transparent margin (about 19% of the width lost). `site/img/favicon-32.png` is the iconset's `icon_32x32.png`, with the same margin. `apple-touch-icon.png` already crops the tile full bleed (`tools/site_images.py`, `big.crop((96, 96, 928, 928))`).

The same `favicon.svg` is also the logo's icon in the header (`site/index.html`, `<img class="b-icon" src="img/favicon.svg">`); the logo must look unchanged.

What to do:
- The favicon SVG crops to the tile: the same drawing with `viewBox="6 6 52 52"` (and width/height 52), so the tile fills the favicon edge to edge. Keep the tile's rounded corners (the corners outside them stay transparent).
- The 32 px PNG favicon is made from the tile too: `tools/site_images.py` crops the tile from the iconset's 1024 px PNG (as it does for the apple-touch icon) and resizes it to 32x32 with LANCZOS, keeping alpha.
- The header logo keeps the full 64x64 app icon, e.g. as a separate `site/img/icon.svg` (a copy of `assets/ui/app_icon.svg`), so its size and look do not change.
- `docs/building.md` (the landing page section) says which file is which.

The user reviews the result in the browser before the agent review; the supervisor serves the build locally.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 site/img/favicon.svg is the app icon's tile with viewBox 6 6 52 52: no transparent margin around the tile
- [x] #2 site/img/favicon-32.png is 32x32 RGBA made by tools/site_images.py from the tile of the 1024 px iconset PNG, with no margin
- [x] #3 The header logo shows the full app icon at the same size as before (a separate SVG equal to assets/ui/app_icon.svg)
- [x] #4 docs/building.md names the favicon and logo files and how they are made
- [x] #5 python3 tools/build_site_test.py passes and tools/build_site.py builds the site
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. favicon.svg = app_icon.svg with viewBox 6 6 52 52, 52x52. 2. site/img/icon.svg copy of app_icon.svg for the header logo. 3. site_images.py: favicon-32 from the 1024 px tile crop, --iconset usable alone. 4. regenerate via ui-art iconset. 5. docs/building.md, tests, build.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a20cc8a542a53e9af

favicon.svg = app_icon.svg with width/height 52, viewBox 6 6 52 52; icon.svg = copy of app_icon.svg for b-icon. site_images.py: --iconset usable alone; favicon-32 now the tile crop (96..928 of icon_512x512@2x) resized 32x32 LANCZOS RGBA (corner pixel transparent, edge pixel (66,75,86,255)). Regenerated with 'cargo run --release -p orb-ui-art --bin ui-art -- iconset DIR' then 'tools/site_images.py --iconset DIR'; git status shows apple-touch-icon and screenshots unchanged. build_site_test.py OK; build_site.py --tag v2026.1118.0919 --out _site built.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Favicon fills its canvas (viewBox 6 6 52 52, 32 px PNG from the tile crop); header logo uses the new site/img/icon.svg, unchanged. tools/site_images.py --iconset works alone; docs/building.md updated. Tests: build_site_test.py passes, site builds to _site.
<!-- SECTION:FINAL_SUMMARY:END -->
