---
id: TASK-211
title: Make the OpenRB-338 demo video for the landing page
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-16T11:06:00Z'
updated_date: '2026-11-16T11:07:00Z'
labels:
  - docs
milestone: m-6
dependencies: []
references:
  - backlog task-212
priority: medium
type: task
ordinal: 192500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The landing page (task-212, GitHub Pages at https://pingw33n.github.io/OpenRB-338/) has a video of OpenRB-338: the user records it (2026-11-16).

To settle while making it:
- what it shows (e.g. a song playing with the panel, a few edits, a mod switch) and its length;
- where it is hosted: a video site embed (YouTube/Vimeo: no size limit, but a third-party player and its tracking on the page) or a file served by Pages itself (MP4/WebM in the site; Pages sites should stay under about 1 GB and files under 100 MB, and git history keeps every version, so a release asset or a separate repository may be better for the file);
- a poster image for the page before playing.

When the video exists, put its URL (and the hosting choice) in this task's final summary; the landing page shows it (task-212 leaves a slot that a URL fills).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The video is recorded and hosted; its URL, hosting choice and poster image are in the final summary
- [ ] #2 The landing page shows it (the URL set in the page's config by the supervisor or an agent)
<!-- AC:END -->
