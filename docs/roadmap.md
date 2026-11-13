# Native platform roadmap

User decision: native Windows, macOS and Linux agents, public GitHub repository. A Docker-only gateway does not meet that requirement. Do not publish a binary that immediately exits as if it were a working platform agent.

## Current: Windows v0.2

Shared policy, Bambu/Orca executable adapters, UIA state/element input without screenshots, optional image path, side-by-side live slicer and native session controls. Manual GUI acceptance is required; no actual print is part of the test suite.

## macOS implementation milestone

- Rust backend using Accessibility (AXUIElement) for per-app trees, focused fields, actions and confirmation.
- App discovery by verified bundle IDs; Bambu and Orca adapters with version information.
- Native local trust/stop panel, app activation and window arrangement; no persistent grant.
- User grants Accessibility/Automation permissions manually. No automated permission changes.
- Build and test both Apple Silicon and Intel, sign/notarize before claiming consumer distribution readiness.
- End-to-end import, orientation, PLA/supports, slicing/Preview checks on real macOS.

## Linux implementation milestone

- AT-SPI accessibility over the session bus, PID/window association and compositor-aware focus.
- X11 input backend; Wayland requires compositor/portal-supported input and explicit user consent, not unrestricted synthetic input assumptions.
- Native local trust/stop panel, correct display/session ownership and fail-closed behavior when access is absent.
- Validate AppImage/native slicer accessibility bridges, desktop environment, scaling, locales and modal dialogs.
- Build/test Linux x86_64; add arm64 only after checking slicer availability and actual operation.

## Release criteria

Shared tests on all runner OSes are useful but do not validate platform UI control. Each native agent must have a real backend, native local approvals/trust, stopped-state tests, signed/verified artifacts as appropriate and a recorded GUI acceptance result. Add the OS to release assets only after these conditions are met.

