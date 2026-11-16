---
id: TASK-204
title: 'Plugin packaging: VST3/CLAP bundles in the packages and CI, with validators'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T07:47:00Z'
labels:
  - tools
  - tests
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
references:
  - backlog task-185 (final summary)
  - scripts/package.sh
  - scripts/build-all.sh
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - docs/building.md
priority: medium
type: task
ordinal: 217000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The plugin crate (task-197) is shipped as .vst3 and .clap bundles with the existing packages (task-185's final summary), and checked by the plugin validators.

- scripts/package.sh TARGET bundles the plugin (nice-plug's bundler: cargo-nice-plug or its xtask) for the six targets and adds the bundles to the package archive; scripts/build-all.sh and release.yml follow. macOS bundles are ad-hoc signed like the app (no notarization yet).
- CI runs clap-validator on the CLAP bundle and the VST3 SDK's validator (or pluginval) on the VST3 bundle, at least on one OS; document what runs where.
- docs/building.md: how to build and install the bundles locally (the user's plugin folders per OS), and the validators.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 scripts/package.sh produces .vst3 and .clap bundles in the package for this machine's target; the other targets are wired the same way
- [ ] #2 CI runs clap-validator and a VST3 validator on the bundles and fails on errors
- [ ] #3 docs/building.md describes building, installing and validating the plugin
<!-- AC:END -->
