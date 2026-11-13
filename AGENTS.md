# Working on this project

Keep project and installed development tools under E:\codex. Rust project, external Windows UI agent; no claim of official Bambu plugin API. Preserve separate slicer adapters and shared safety layer.

Generic input requires per-action local approval, except when a human explicitly grants FULL TRUST including print/upload/delete in the local panel for the target process and current session. This user-authorized mode overrides the earlier always-confirm rule. Approval cannot be granted by an MCP tool. Consume observations before validation; fail closed on focus/window/state ambiguity. Do not run actual print, upload, delete or modify a user's open project during tests without explicit authorization for that concrete operation.

Use dev.ps1 test/build/fmt/clippy and tests-mcp.ps1. Unit tests are not evidence of real slicer workflow success. Keep docs/validation.md honest. Avoid adding cloud endpoints or collecting credentials.


