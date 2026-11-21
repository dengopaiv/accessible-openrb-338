---
id: TASK-268
title: 'Themes 3/3: a crossfade between the modern and classic skins'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T13:27:00Z'
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
- [x] #1 Switching View > Classic / Modern crossfades the panel over about 500 ms (a code constant) with smoothstep easing, both directions; the desk, shadows and menu bar stay still
- [x] #2 The fade draws the modern skin, then the classic texture over it with the fade's alpha (no capture, no per-part fading); both skins stay live during the fade
- [x] #3 Input during the fade is defined and documented, and a switch during a fade reverses from the current alpha
- [x] #4 The alpha schedule is unit tested and a both-way switch test passes; docs/ui_spec.md describes the transition
- [x] #5 The user reviewed the build (and tuned the duration if wanted) before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. skin/fade.rs: FADE_SECONDS, smoothstep, Fade (from/to, clock from first frame, reversal scaled by distance). 2. Skin::paint_over(alpha), classic tints its opaque texture. 3. App: set_theme starts the fade, target skin takes input at once, classic kept until a fade to modern ends; show draws modern then classic over it. 4. Tests + docs/ui_spec.md section 22.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a2bebe63d47dba0cd

Implemented. Input goes to the target skin from the switch (documented in ui_spec.md section 22). Reversal: new fade from the last drawn alpha, duration scaled by distance. Tests: cargo test all pass, clippy and nightly fmt clean; new tests skin/fade.rs (2) and tests/classic.rs the_switch_crossfades_both_ways_and_reverses_from_the_current_alpha. Dev app: target/OpenRB-338.app in the worktree. Awaiting the user's review (AC 5 unchecked, task left To Do).

User review round 1 (2026-11-21): the modern mod window's splash disappeared when fading to Classic. Cause: app.rs show() filled Frame.splash only when the TARGET skin's layout had a modview control; classic.tsv has none, so the modern skin drawn under the fade got no picture. Fix: the splash is also given while a fade runs. Test: the_switch_crossfades_... now asserts App::splash_in_frame() in every fade frame, both directions. AC 5 stays unchecked.

User review (2026-11-21): 'good'; the splash fix accepted, duration stays 0.5 s (FADE_SECONDS in skin/fade.rs).

User request (2026-11-21): remove the environment variable that overrides the fade duration; FADE_SECONDS stays a const. Checked: this branch never had such a variable (grep of crates/ and docs/ for ORB_*FADE and env reads: none; docs/building.md lists none). An inlining of the constant I had started was undone; no code change.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Switching View > Classic / Modern crossfades the panel over 0.5 s (FADE_SECONDS, skin/fade.rs) with smoothstep: the modern skin is drawn, then the classic texture over it with the fade's alpha. Both skins stay live, input goes to the target skin at once, a switch during a fade reverses from the current alpha, the splash stays under the fade (review round fix). Branch worktree-agent-a2bebe63d47dba0cd. Tests: full cargo test, clippy and nightly fmt clean; unit tests of the alpha schedule and a both-way switch test with the splash check. User reviewed the build: good.
<!-- SECTION:FINAL_SUMMARY:END -->
