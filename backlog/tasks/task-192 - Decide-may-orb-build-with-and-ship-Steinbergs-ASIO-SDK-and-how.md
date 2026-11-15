---
id: TASK-192
title: 'Decide: may orb build with and ship Steinberg''s ASIO SDK, and how'
status: To Do
assignee:
  - '@pingw33n'
created_date: '2026-11-15T12:44:00Z'
updated_date: '2026-11-15T12:45:00Z'
labels:
  - decision
  - player
milestone: m-5
dependencies: []
references:
  - LICENSE
  - .github/workflows/release.yml
  - docs/building.md
  - task-193
  - https://github.com/pingw33n/OpenRB-338/issues/12
priority: low
type: spike
ordinal: 205500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
task-193 adds ASIO output on Windows through cpal's `asio` feature (GitHub issue #12). It needs Steinberg's ASIO SDK at build time (`asio-sys` generates bindings from its headers and compiles parts of it into the binary). orb is `GPL-3.0-only` (LICENSE) and its release packages are public.

The ASIO SDK was long under a proprietary Steinberg license, which made shipping it in GPL binaries doubtful. As recalled in the discussion of 2026-11-15 (to verify on Steinberg's site and in the SDK's license files), Steinberg added a GPLv3 option in late 2025; if so, GPL-3.0-only orb may use it under GPLv3.

To decide:
1. Is the current ASIO SDK's license compatible with shipping GPL-3.0-only binaries (verify the version and its license text)?
2. Where the build gets it: downloaded in CI from Steinberg at a pinned version with a checksum (recommended), or vendored in the repository (only if the license allows redistribution of the sources, and it adds the SDK to git).
3. Whether the Windows packages ship with ASIO by default (recommended, if 1 allows), or as a separate build.
4. Any notice the license requires in the package or README (the ASIO trademark: Steinberg's "ASIO is a trademark and software of Steinberg Media Technologies GmbH" line).

Recommendation: if the GPLv3 option is confirmed, download the pinned SDK in CI with a checksum, ship ASIO in the Windows packages, and add the required notice; otherwise keep ASIO out of release builds and document a local build with the SDK.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The ASIO SDK version and its license terms are recorded in the final summary with links
- [ ] #2 Each of the four questions has an outcome with the date in the final summary, and task-193 is updated or unblocked accordingly
<!-- AC:END -->
