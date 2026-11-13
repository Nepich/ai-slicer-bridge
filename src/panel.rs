use crate::{session, *};
use std::{
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};
static PANEL: AtomicUsize = AtomicUsize::new(0);
static TARGET: std::sync::Mutex<Option<Window>> = std::sync::Mutex::new(None);
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn shutdown() {
    let hwnd = PANEL.load(Ordering::SeqCst) as HWND;
    if !hwnd.is_null() {
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
}
fn open(slicer: Slicer) -> Result<Window> {
    let mut desktop = crate::windows::WindowsDesktop::default();
    let mut windows = desktop.windows(slicer)?;
    if windows.is_empty() {
        let exe = slicer.adapter().executable();
        let dir = match slicer {
            Slicer::Bambu => "Bambu Studio",
            Slicer::Orca => "OrcaSlicer",
        };
        let mut roots = vec![std::path::PathBuf::from("E:/Programs")];
        for key in ["ProgramFiles", "LOCALAPPDATA"] {
            if let Some(p) = std::env::var_os(key) {
                roots.push(p.into());
            }
        }
        let path=roots.into_iter().map(|r|r.join(dir).join(exe)).find(|p|p.is_file()).ok_or("Slicer not found. Start it manually, then click Open again. No software will be installed.")?;
        std::process::Command::new(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(200));
            windows = desktop.windows(slicer)?;
            if !windows.is_empty() {
                break;
            }
        }
    }
    if windows.len() != 1 {
        return Err("More than one target window or no visible window. Close extra windows or finish startup manually.".into());
    }
    Ok(windows.remove(0))
}
fn alongside(panel: HWND, target: &Window) -> Result<()> {
    unsafe {
        let hwnd = target.handle as HWND;
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid != target.pid || IsWindow(hwnd) == 0 {
            return Err("Target changed".into());
        }
        let mut area: RECT = std::mem::zeroed();
        if SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut area as *mut _ as *mut _, 0) == 0 {
            return Err("Work area unavailable".into());
        }
        let width = area.right - area.left;
        let height = area.bottom - area.top;
        let panel_width = 380.min(width / 3);
        ShowWindow(hwnd, SW_RESTORE);
        SetWindowPos(
            panel,
            null_mut(),
            area.left,
            area.top,
            panel_width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        if SetWindowPos(
            hwnd,
            null_mut(),
            area.left + panel_width,
            area.top,
            width - panel_width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        ) == 0
        {
            return Err("Could not arrange slicer".into());
        }
        SetForegroundWindow(hwnd);
        Ok(())
    }
}
unsafe fn control(hwnd: HWND, class: &str, text: &str, id: usize, y: i32, height: i32) -> HWND {
    unsafe {
        CreateWindowExW(
            0,
            wide(class).as_ptr(),
            wide(text).as_ptr(),
            WS_CHILD | WS_VISIBLE | if class == "BUTTON" { WS_TABSTOP } else { 0 },
            16,
            y,
            320,
            height,
            hwnd,
            id as HMENU,
            GetModuleHandleW(null_mut()),
            null_mut(),
        )
    }
}
unsafe extern "system" fn procedure(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                control(
                    hwnd,
                    "STATIC",
                    "AI Slicer Bridge — live workspace",
                    0,
                    16,
                    32,
                );
                control(
                    hwnd,
                    "STATIC",
                    "Write commands in the connected AI chat. The real slicer is shown alongside. No screenshot stream.",
                    0,
                    52,
                    62,
                );
                control(hwnd, "BUTTON", "Open Bambu Studio alongside", 101, 124, 36);
                control(hwnd, "BUTTON", "Open OrcaSlicer alongside", 102, 168, 36);
                control(hwnd, "BUTTON", "Confirm each action / Resume", 103, 228, 36);
                control(
                    hwnd,
                    "BUTTON",
                    "Full trust for selected slicer...",
                    104,
                    272,
                    36,
                );
                control(hwnd, "BUTTON", "STOP — revoke trust", 105, 316, 44);
                control(
                    hwnd,
                    "STATIC",
                    "Not connected to an AI by this panel alone. Configure the MCP client with --panel.",
                    0,
                    384,
                    64,
                );
                control(
                    hwnd,
                    "STATIC",
                    "Ready. Every input needs local approval.",
                    106,
                    470,
                    180,
                );
                SetTimer(hwnd, 1, 300, None);
                0
            }
            WM_TIMER => {
                let mode = if session::PAUSED.load(Ordering::SeqCst) {
                    "STOPPED"
                } else if TARGET
                    .lock()
                    .ok()
                    .and_then(|t| t.as_ref().map(|w| session::trusted(w.pid)))
                    .unwrap_or(false)
                {
                    "FULL TRUST (includes printing)"
                } else {
                    "CONFIRM EACH ACTION"
                };
                let text = format!(
                    "{mode}\r\n\r\n{}",
                    session::STATUS
                        .lock()
                        .map(|s| s.clone())
                        .unwrap_or_default()
                );
                SetWindowTextW(GetDlgItem(hwnd, 106), wide(&text).as_ptr());
                0
            }
            WM_COMMAND => {
                match w & 0xffff {
                    101 | 102 => {
                        session::revoke();
                        let slicer = if w & 0xffff == 101 {
                            Slicer::Bambu
                        } else {
                            Slicer::Orca
                        };
                        match open(slicer).and_then(|target| {
                            alongside(hwnd, &target)?;
                            Ok(target)
                        }) {
                            Ok(target) => {
                                session::status(&format!(
                                    "Selected {} (PID {}). Ready for MCP commands.",
                                    slicer.adapter().label(),
                                    target.pid
                                ));
                                if let Ok(mut t) = TARGET.lock() {
                                    *t = Some(target);
                                }
                            }
                            Err(e) => session::status(&e),
                        }
                    }
                    103 => session::resume(),
                    105 => session::pause(),
                    104 => {
                        let target = TARGET.lock().ok().and_then(|t| t.clone());
                        if let Some(target) = target {
                            let warning = format!(
                                "Grant FULL TRUST to {} (PID {}) for this agent session?\n\nThe AI may SEND PRINT JOBS, upload files and delete models WITHOUT more approval. Check printer readiness yourself.\n\nSTOP, closing this panel or restarting the agent revokes trust. Other slicer processes remain untrusted.\n\nOnly choose Yes if you accept these actions.",
                                target.executable, target.pid
                            );
                            if MessageBoxW(
                                hwnd,
                                wide(&warning).as_ptr(),
                                wide("Full trust — includes printing").as_ptr(),
                                MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
                            ) == IDYES
                            {
                                let mut pid = 0;
                                GetWindowThreadProcessId(target.handle as HWND, &mut pid);
                                if pid == target.pid && IsWindow(target.handle as HWND) != 0 {
                                    session::grant(target.pid);
                                } else {
                                    session::status("Target exited; no trust granted.");
                                }
                            }
                        } else {
                            session::status("Select a slicer with Open first.");
                        }
                    }
                    _ => {}
                }
                0
            }
            WM_CLOSE => {
                session::pause();
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                PANEL.store(0, Ordering::SeqCst);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, w, l),
        }
    }
}
pub fn run() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(null_mut());
        let class = wide("AISlicerBridgePanel");
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(procedure);
        wc.hInstance = instance;
        wc.lpszClassName = class.as_ptr();
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
        if RegisterClassW(&wc) == 0 {
            return Err("Panel class registration failed".into());
        }
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            wide("AI Slicer Bridge — live workspace").as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            40,
            40,
            380,
            740,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        if hwnd.is_null() {
            return Err("Panel creation failed".into());
        }
        PANEL.store(hwnd as usize, Ordering::SeqCst);
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, null_mut(), 0, 0);
            if result <= 0 {
                break;
            }
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }
}
