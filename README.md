# AI Slicer Bridge

Local Rust agent for controlling the **real GUI** of Bambu Studio and OrcaSlicer from an MCP-compatible AI chat.

**Status: experimental Windows implementation, v0.2.0.** Native macOS and Linux agents are planned; they are not implemented or released yet. No official general-purpose Bambu Studio plugin API is assumed. This project is independent of Bambu Lab and OrcaSlicer.

## Live workspace

Start the MCP server with `--panel`. In the local panel choose **Open Bambu Studio alongside** or **Open OrcaSlicer alongside**. The existing slicer window is arranged beside the control panel. If no window is running, the panel searches common installation folders and starts the installed application. It never installs the slicer.

You write instructions in the connected AI chat and watch changes in the **actual slicer window**, without a stream of screenshots. The panel displays the latest requested input and has a **STOP** button. It is not an embedded AI chat and does not establish an MCP connection by itself. The MCP client must start this same process; a second independent panel cannot control the first process's trust state.

Windows UI Automation reads accessible controls, names and bounds. It does not capture pixels. Weak or absent accessibility in a custom 3D canvas is a real limitation: the agent must report missing controls instead of guessing. No claim of fully autonomous model preparation is made.

Screenshots are **disabled by default**. The original image-based tools are exposed only with explicit `--allow-screenshots`; the no-image path never calls the capture backend.

## Safety and full trust

- Default: **every input requires local confirmation**. An arbitrary click or Enter can submit a print, so the AI cannot label its own input as harmless to bypass approval.
- Optional: choose **Full trust for selected slicer** in the local panel and accept its explicit warning. Trust applies to that slicer process for the current agent session and **includes print submission, upload and deletion without further confirmation**.
- **STOP**, closing the panel, selecting a different slicer, restarting the agent, or choosing **Confirm each action / Resume** revokes full trust. STOP blocks subsequent inputs; it cannot undo an input already sent or cancel a physical print already started.
- There is no MCP tool that can grant trust. The dialog defaults to No. Trust is never persisted.
- Every action consumes an observed state and rechecks controls before input. State expires after 60 seconds. Changes, focus problems or missing fields stop the operation.

Full trust is optional and broad, not a guarantee of correct settings or safe hardware. The human remains responsible for selecting the right printer, material, nozzle, plate and checking printer readiness. Do not use this experimental agent unattended.

## Windows build

Install stable Rust and the usual Windows Rust linker prerequisites (MSVC recommended). From the repository:

```powershell
cargo test --locked
cargo clippy --locked -- -D warnings
cargo build --release --locked
```

This development installation uses `E:\codex\AISlicerBridge`, with portable Rust and LLVM-MinGW under `E:\codex\tools`; `dev.ps1` configures those local tools. The absolute-path `.cargo/config.toml` is local and excluded from version control. It is not needed on a normal MSVC installation.

## Connect an MCP client

Point a local stdio MCP client at the compiled executable:

```json
{
  "mcpServers": {
    "ai-slicer-bridge": {
      "command": "C:\\path\\to\\ai-slicer-bridge.exe",
      "args": ["--panel"]
    }
  }
}
```

The bundled `mcp.example.json` uses this installation's E: path. An ordinary cloud chat cannot launch an exe on your computer. The client must support local MCP; this project does not automatically connect your ChatGPT account. No API keys, cloud server or printer credentials are required by the bridge.

## Tools

| Tool | Purpose |
| --- | --- |
| `slicer_windows` | List visible non-minimized windows for `bambu` or `orca` |
| `slicer_inspect` | Read accessible elements and return a one-use state digest, without images |
| `slicer_ui_action` | Click an observed element, press an allowed key, or type literal text into a focused accessible edit field |
| `slicer_observe` | Optional screenshot tool; only with `--allow-screenshots` |
| `slicer_action` | Optional image-bound input; only with `--allow-screenshots` |

To import/select a model, rotate it, choose a compatible PLA preset, configure supports, slice and check Preview, the AI must inspect the actual accessible controls and verify each result. Numeric temperatures are not hardcoded. Custom canvas painting and a reliable universal semantic workflow across slicer versions are not implemented. A dispatched input is not proof that slicing or print submission succeeded.

If a dialog opens or input succeeds but the subsequent read fails, **do not repeat blindly**: list windows and inspect the new state first. Password controls are excluded. Windows may block focus activation; click the slicer manually if requested. Do not move the mouse during an approved input.

## Architecture and testing

Shared Rust policy + separate Bambu/Orca adapters + platform desktop backend. Windows: UI Automation for observation, Win32 mouse/keyboard input, native panel and approval dialogs. Legacy GDI capture is optional. No HTTP listener, arbitrary shell execution tool or direct printer API.

`cargo test` checks approval refusal, stale/single-use state, UI changes during approval, inaccessible/password controls and a no-screenshot path whose mock capture function panics if called. `tests-mcp.ps1` checks stdio handshake, tool schemas and refusal without observation, with no GUI input.

See [validation](docs/validation.md), [research](docs/research.md), [acceptance checklist](docs/acceptance.md) and [cross-platform roadmap](docs/roadmap.md). Automated tests do not replace real GUI acceptance on each supported OS and slicer version.

## Releases

GitHub Actions builds/tests Windows and uploads a Windows archive. Tags `v*` publish that archive and SHA-256 checksum to a GitHub Release. Linux/macOS CI runs shared-core tests; unsupported-agent binaries are deliberately not advertised as working releases.

Docker is not used as a substitute for native desktop access. Native macOS Accessibility and Linux AT-SPI backends must run in the logged-in desktop session, with the OS permissions required by those platforms. See the roadmap before relying on non-Windows support.

