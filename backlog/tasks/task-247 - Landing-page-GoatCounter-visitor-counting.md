---
id: TASK-247
title: 'Landing page: GoatCounter visitor counting'
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-18T10:15:00Z'
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
- [ ] #1 The built _site/index.html loads https://gc.zgo.at/count.js async with data-goatcounter="https://pingw33n.goatcounter.com/count"
- [ ] #2 build_site_test.py asserts it and passes; node site/detect_test.js passes
- [ ] #3 docs/building.md names GoatCounter, its dashboard URL, and that it is the page's only third-party request
<!-- AC:END -->
