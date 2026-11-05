---
id: TASK-206
title: A system file dialog that cannot open reads as a cancel (Linux)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-06T05:52:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-194
references:
  - crates/orb-ui/src/system_dialog.rs
  - crates/orb-ui/src/app.rs
priority: low
type: bug
ordinal: 225000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: in screen reader mode the file dialogs are rfd::FileDialog (TASK-194). On Linux rfd uses the XDG desktop portal over D-Bus (feature xdg-portal, no GTK). When libdbus or the portal is missing, or the call fails, rfd returns None, which `App::system_pick` and `App::choose_folder` read as the user cancelling: Open does nothing and says nothing, and a screen reader user cannot tell why.

What: find out what rfd 0.17.2 exposes of a failure (its log output, or a way to tell a failed portal call from a cancel; the async API `AsyncFileDialog` may report it) and, when a dialog cannot open, tell the user once (the error box: "The system's file dialog could not be opened") and fall back to the in-app browser for that request. If rfd cannot tell the two apart, say so in the task notes and propose the smallest fallback (for example, a timeout: a dialog that returns None within a few milliseconds of the call).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On Linux with no portal or no D-Bus session (test it in a container or with DBUS_SESSION_BUS_ADDRESS unset), choosing File > Open in screen reader mode shows the error box or the in-app browser instead of doing nothing
- [ ] #2 A cancelled dialog still changes nothing and shows nothing
<!-- AC:END -->
