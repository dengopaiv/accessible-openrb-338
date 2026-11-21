---
id: TASK-268
title: 'Themes 3/3: a crossfade between the modern and classic skins'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T12:54:00Z'
labels:
  - ui
milestone: m-8
dependencies:
  - TASK-267
references:
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** Task 3 of the user's themes request (2026-11-21, milestone m-8), after task-267 (View > Classic / Modern, two radio items directly in View).

**The user's decision (2026-11-21).** The user first asked for a pixelating transition. They saw preview videos of three variants: pixelate-and-fade, pixelate-first-then-fade, and fade only. They chose the **plain crossfade**. The pixelation, the panel capture and the scaling are dropped.

**The animation.**
- Switching skins crossfades the panel over about 500 ms: a constant in the code, not a setting. The user may tune it at review.
- The ease curve is smoothstep, `t*t*(3-2t)`, the one the user approved in the preview.
- Both directions: Modern → Classic fades the classic panel in, and Classic → Modern fades it out.
- The desk, the shadows and the menu bar don't animate.

**How (cheap, no capture).** The classic skin is drawn as one opaque texture (skin/classic.rs). So:
1. Draw the modern skin normally.
2. Draw the classic texture over it with alpha going 0 → 1 (to Classic) or 1 → 0 (to Modern).

That's an exact crossfade with no offscreen render. Both panels stay live during the fade: meters, lights and running steps. Don't fade the modern skin's many SVG parts one by one; their overlap would show lower parts through upper ones.

**Input during the fade.** It goes to the target skin's layout as soon as the switch is made. Document that, or choose another rule and say why.

**Standalone only.** The plugin has no classic skin.

**Testing.**
- The alpha schedule is unit tested: start, end, monotonic, and the ease.
- A test switches both ways and checks that the end state draws only the target skin, and that a switch during a fade reverses smoothly from the current alpha.

The user is the first reviewer: give them a dev app (`scripts/macos-dev-app.sh`) and say where the duration constant is, before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Switching View > Classic / Modern crossfades the panel over about 500 ms (a code constant) with smoothstep easing, both directions; the desk, shadows and menu bar stay still
- [ ] #2 The fade draws the modern skin, then the classic texture over it with the fade's alpha (no capture, no per-part fading); both skins stay live during the fade
- [ ] #3 Input during the fade is defined and documented, and a switch during a fade reverses from the current alpha
- [ ] #4 The alpha schedule is unit tested and a both-way switch test passes; docs/ui_spec.md describes the transition
- [ ] #5 The user reviewed the build (and tuned the duration if wanted) before the agent review
<!-- AC:END -->
