# Local application connection

This is an external slicer agent connected to the AI client over local MCP, not a plugin installed inside the slicer.

The installed client supports `codex mcp add`. For this installation:

```powershell
codex mcp add ai_slicer_bridge -- E:\codex\AISlicerBridge\target\release\ai-slicer-bridge.exe
codex mcp get ai_slicer_bridge
```

Run `install-local.ps1` after building to register this checkout's release executable. It modifies only the named MCP server entry using the client's CLI. No tokens or network service are required.

Restart the desktop application after registration if its existing chat has not loaded the new tools, then return to the same conversation. Ask the AI to verify that `slicer_preferences`, `slicer_open`, `slicer_windows`, `slicer_inspect` and `slicer_ui_action` are available before claiming a live connection. Configuration output alone is not that verification.

The first request can be: "Open Bambu Studio through AI Slicer Bridge". The AI calls `slicer_open`, which opens/activates the installed program and arranges its actual window beside the local panel. `slicer_inspect` reads accessible controls without screenshots. Subsequent edits need local approval or full trust explicitly granted by the human in the panel. Full trust is off by default.

Ordinary ChatGPT web does not read local Codex configuration. Web access would require a supported remote plugin/tunnel with a local agent and explicit setup. No cloud connection is configured by this installer. This integration does not embed the native slicer window inside ChatGPT; the window is displayed alongside it.

Official setup references:
- https://learn.chatgpt.com/docs/extend/mcp?surface=cli
- https://developers.openai.com/plugins/build/plugins

As of v0.2.3 the server is a background GUI-subsystem executable, so startup creates no panel or console. AI_SLICER_SETTINGS_PATH points at local/preferences.json on E: in this installation; local/ is excluded from Git. The user selected Bambu Studio. First connection opened it and saved connection=bambu. Subsequent connections must reuse the saved choice without another connection question unless the user asks to change or reset it. If no choice exists, ask once. Approvals/trust are not part of this file.

