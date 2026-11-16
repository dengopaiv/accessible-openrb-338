---
id: TASK-192
title: 'Decide: may orb build with and ship Steinberg''s ASIO SDK, and how'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-15T12:44:00Z'
updated_date: '2026-11-16T07:55:00Z'
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
- [x] #1 The ASIO SDK version and its license terms are recorded in the final summary with links
- [x] #2 Each of the four questions has an outcome with the date in the final summary, and task-193 is updated or unblocked accordingly
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Checked 2026-11-16 in the SDK itself (Steinberg's page https://www.steinberg.net/developers/asiosdk-open/ renders its text by JavaScript only). https://www.steinberg.net/asiosdk redirects to https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip, SHA-256 d5ebf0c20dd2c5f43771fd0c1418f4b361bf52434ee670097cfa6b3a335e2eca. changes.txt: 'Changes in ASIO 2.3.4 - new DUAL licensing: Proprietary and GPVv3'. LICENSE.txt (top level and common/): 'licensed under the terms of the Steinberg ASIO License, or alternatively under the terms of the General Public License (GPL) Version 3' (no 'or later'). README FAQ: switching to GPLv3 is allowed 'if your product is released under GPLv3 License terms (including source code disclosure)'. Per-file licenses: common/asio.h, asio.cpp, asiosys.h, iasiodrv.h refer to LICENSE.txt (dual); host/asiodrivers.cpp/.h, host/pc/asiolist.cpp/.h, host/ginclude.h carry a BSD-3-clause text ((c) 2025 Steinberg; keep the notice in source, reproduce it in binary documentation); common/combase.* is Microsoft's (driver side, not needed by a host). Trademark (README): the 'ASIO' name or logo is optional under GPLv3; if used, Steinberg's ASIO Usage Guidelines apply (logo unaltered, product contexts only, never 'ASIO' in a product name). Press coverage: Steinberg announced the GPLv3 option in October 2025 (heise.de/-10963451, librearts.org/2025/11/steinberg-relicenses-vst3-and-asio/).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user, 2026-11-16.

ASIO SDK 2.3.4 (ASIO-SDK_2.3.4_2025-10-15.zip, https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip, SHA-256 d5ebf0c20dd2c5f43771fd0c1418f4b361bf52434ee670097cfa6b3a335e2eca) is dual-licensed: Steinberg's proprietary ASIO License or GPL Version 3; the host helpers (host/asiodrivers.*, host/pc/asiolist.*, host/ginclude.h) are BSD-3-clause. Details and quotes in the notes.

1. **License:** compatible. orb (GPL-3.0-only) uses the SDK under GPLv3.
2. **Where the build gets it:** vendored minimally in the repository: only the files a host build needs (cpal's asio-sys: the common headers asio.h, asiosys.h, iasiodrv.h, common/asio.cpp, host/asiodrivers.*, host/ginclude.h, host/pc/asiolist.*, and whatever else asio-sys's build actually compiles or includes), unmodified, with the SDK's LICENSE.txt, the version and the zip's SHA-256 in a README next to them; CPAL_ASIO_DIR points there. No driver-side files (combase, dllentry, register, the sample driver), no PDFs, no logo artwork. No download at build time.
3. **Windows packages:** ship with ASIO by default.
4. **Notices:** the package's third-party notices carry the SDK's LICENSE.txt and the BSD copyright and disclaimer of the host helpers; wherever the name ASIO appears (README, docs, the Audio Output window) a line 'ASIO is a trademark and software of Steinberg Media Technologies GmbH'. No ASIO logo.

task-193 is updated accordingly and no longer blocked by this task.
<!-- SECTION:FINAL_SUMMARY:END -->
