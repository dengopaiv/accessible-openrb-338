---
id: TASK-247
title: 'Landing page: GoatCounter visitor counting'
status: Done
assignee:
  - '@agent-chore'
created_date: '2026-11-18T10:15:00Z'
updated_date: '2026-11-18T10:17:00Z'
labels:
  - tools
milestone: m-6
dependencies: []
references:
  - site/index.html
  - tools/build_site.py
  - tools/build_site_test.py
  - docs/building.md
  - task-212
priority: medium
type: enhancement
ordinal: 190600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user (2026-11-18) signed up for GoatCounter, a cookieless, free, open-source analytics service, to count the landing page's visitors. They gave this snippet:

```html
<script data-goatcounter="https://pingw33n.goatcounter.com/count"
        async src="//gc.zgo.at/count.js"></script>
```

What to do:
- Add it to the landing page's `<head>` (site/index.html, task-212), with `src="https://gc.zgo.at/count.js"`. The protocol-relative `//` fails when the built page is opened from a file. Keep `async` and the `data-goatcounter` URL exactly as given.
- If the page or build_site.py sets a Content-Security-Policy, allow gc.zgo.at for scripts and pingw33n.goatcounter.com for connect/img.
- docs/building.md (the landing page section): the page counts visitors with GoatCounter (cookieless, no consent banner needed). The dashboard is https://pingw33n.goatcounter.com. This is the page's only third-party request.
- Extend tools/build_site_test.py to assert that the built index.html contains the GoatCounter script with that data-goatcounter URL.

Do not change anything else on the page.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The built _site/index.html loads https://gc.zgo.at/count.js async with data-goatcounter="https://pingw33n.goatcounter.com/count"
- [x] #2 build_site_test.py asserts it and passes; node site/detect_test.js passes
- [x] #3 docs/building.md names GoatCounter, its dashboard URL, and that it is the page's only third-party request
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add the GoatCounter script to site/index.html's head as given in the task, with src=https://gc.zgo.at/count.js (not protocol-relative), async and the data-goatcounter URL exact; before </head>. No CSP exists in index.html or build_site.py (checked by grep), so none to change.
2. Extend tools/build_site_test.py Build.test_build to assert the built index.html contains the script tag with async, src https://gc.zgo.at/count.js and data-goatcounter https://pingw33n.goatcounter.com/count.
3. Add a visitor-counting bullet to docs/building.md's landing page section (task-212): GoatCounter, cookieless, no consent banner, dashboard https://pingw33n.goatcounter.com, the page's only third-party request when it loads.
4. Run /usr/bin/python3 -m unittest tools/build_site_test.py and node site/detect_test.js; build with --tag v2026.1118.0919 --out _site; grep goatcounter _site/index.html.
5. Check ACs, final summary, Done, one commit (tools subject, Task: task-247).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a5dc68f5c4b234988 (worktree /Users/pingw33n/dev/OpenRB-338/.claude/worktrees/agent-a5dc68f5c4b234988), rebased onto main (9dba6a2 has the task-247 file).

Done in the worktree: site/index.html head gets the GoatCounter script (before </head>, after the js class script) with src https://gc.zgo.at/count.js, async, data-goatcounter https://pingw33n.goatcounter.com/count; an HTML comment above it names the cookieless reason. No CSP in index.html or build_site.py (grep for content-security/csp: none), so none added.
Ran: python3 -m unittest tools/build_site_test.py: Ran 4 tests OK (the new assertion is in Build.test_build). A ResourceWarning from the existing Names test (open() without close) is pre-existing, not changed.
Ran: node site/detect_test.js: detect_test: ok.
Ran: python3 tools/build_site.py --tag v2026.1118.0919 --out _site: built _site for v2026.1118.0919: 15 release assets linked. grep goatcounter _site/index.html: line 47 has data-goatcounter=https://pingw33n.goatcounter.com/count; grep -c gc.zgo.at/count.js: 1. _site is git-ignored (.gitignore line 35).
docs/building.md: new bullet 'The visitor count' in the landing page section (task-212), names GoatCounter, dashboard URL, cookieless, only third-party request.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
GoatCounter is on the landing page. site/index.html carries the script in its head (src https://gc.zgo.at/count.js, async, data-goatcounter https://pingw33n.goatcounter.com/count). tools/build_site_test.py asserts the script in the built page. docs/building.md (landing page section) names GoatCounter, its dashboard URL, that it is cookieless, and that it is the page's only third-party request. No CSP exists, so none changed. Branch worktree-agent-a5dc68f5c4b234988, rebased onto main, one commit. Tests: python3 -m unittest tools/build_site_test.py Ran 4 tests OK; node site/detect_test.js ok; the build with --tag v2026.1118.0919 wrote _site/index.html with the script at line 47 (one occurrence of gc.zgo.at/count.js). Not pushed.
<!-- SECTION:FINAL_SUMMARY:END -->
