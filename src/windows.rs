use crate::*;
use base64::Engine;
use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
    path::Path,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::Threading::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
#[derive(Default)]
pub struct WindowsDesktop {
    permit: Option<(u32, u64)>,
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
unsafe extern "system" fn collect(hwnd: HWND, param: LPARAM) -> i32 {
    unsafe {
        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return 1;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let p = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if p.is_null() {
            return 1;
        }
        let mut buf = vec![0u16; 32768];
        let mut n = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(p, 0, buf.as_mut_ptr(), &mut n);
        CloseHandle(p);
        if ok == 0 {
            return 1;
        }
        let exe = String::from_utf16_lossy(&buf[..n as usize]);
        let mut title = vec![0u16; GetWindowTextLengthW(hwnd) as usize + 1];
        let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
        let ws = &mut *(param as *mut Vec<Window>);
        ws.push(Window {
            handle: hwnd as usize,
            pid,
            title: String::from_utf16_lossy(&title[..len as usize]),
            executable: exe,
        });
        1
    }
}
fn verify(w: &Window) -> Result<HWND> {
    unsafe {
        let h = w.handle as HWND;
        let mut pid = 0;
        GetWindowThreadProcessId(h, &mut pid);
        if pid != w.pid || IsWindow(h) == 0 || IsIconic(h) != 0 || GetForegroundWindow() != h {
            return Err("Target must be the visible foreground window; focus it manually".into());
        }
        Ok(h)
    }
}
impl Desktop for WindowsDesktop {
    fn authorize(&mut self, w: &Window, summary: &str) -> Result<bool> {
        use std::sync::atomic::Ordering;
        self.permit = None;
        crate::session::check()?;
        let epoch = crate::session::EPOCH.load(Ordering::SeqCst);
        crate::session::status(summary);
        let approved = if crate::session::trusted(w.pid) {
            true
        } else {
            self.confirm(summary)?
        };
        if approved && epoch == crate::session::EPOCH.load(Ordering::SeqCst) {
            self.permit = Some((w.pid, epoch));
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn windows(&mut self, s: Slicer) -> Result<Vec<Window>> {
        unsafe {
            let mut ws = Vec::<Window>::new();
            if EnumWindows(Some(collect), &mut ws as *mut _ as LPARAM) == 0 {
                return Err("EnumWindows failed".into());
            }
            Ok(ws
                .into_iter()
                .filter(|w| {
                    Path::new(&w.executable).file_name().is_some_and(|n| {
                        n.to_string_lossy()
                            .eq_ignore_ascii_case(s.adapter().executable())
                    })
                })
                .collect())
        }
    }
    fn capture(&mut self, w: &Window) -> Result<Snapshot> {
        unsafe {
            let h = verify(w)?;
            let mut rect: RECT = zeroed();
            if GetClientRect(h, &mut rect) == 0 {
                return Err("GetClientRect failed".into());
            }
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width <= 0 || height <= 0 || width > 12000 || height > 12000 {
                return Err("Invalid window dimensions".into());
            }
            let dc = GetDC(h);
            if dc.is_null() {
                return Err("GetDC failed".into());
            }
            let mem = CreateCompatibleDC(dc);
            let bmp = CreateCompatibleBitmap(dc, width, height);
            if mem.is_null() || bmp.is_null() {
                if !mem.is_null() {
                    DeleteDC(mem);
                }
                if !bmp.is_null() {
                    DeleteObject(bmp);
                }
                ReleaseDC(h, dc);
                return Err("Capture allocation failed".into());
            }
            let old = SelectObject(mem, bmp);
            let copied = BitBlt(mem, 0, 0, width, height, dc, 0, 0, SRCCOPY | CAPTUREBLT);
            SelectObject(mem, old);
            let mut info: BITMAPINFO = zeroed();
            info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            info.bmiHeader.biWidth = width;
            info.bmiHeader.biHeight = -height;
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB;
            let mut rgba = vec![0u8; width as usize * height as usize * 4];
            let rows = GetDIBits(
                mem,
                bmp,
                0,
                height as u32,
                rgba.as_mut_ptr() as *mut c_void,
                &mut info,
                DIB_RGB_COLORS,
            );
            DeleteObject(bmp);
            DeleteDC(mem);
            ReleaseDC(h, dc);
            if copied == 0 || rows != height {
                return Err("Screenshot capture failed".into());
            }
            for p in rgba.as_chunks_mut::<4>().0 {
                p.swap(0, 2);
                p[3] = 255;
            }
            verify(w)?;
            let digest = fingerprint(&rgba, w, width as u32, height as u32);
            let mut bytes = Vec::new();
            {
                let mut enc = png::Encoder::new(&mut bytes, width as u32, height as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                let mut writer = enc.write_header().map_err(|e| e.to_string())?;
                writer.write_image_data(&rgba).map_err(|e| e.to_string())?;
            }
            Ok(Snapshot {
                window: w.clone(),
                width: width as u32,
                height: height as u32,
                digest,
                png_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            })
        }
    }
    fn confirm(&mut self, summary: &str) -> Result<bool> {
        unsafe {
            let owner = GetForegroundWindow();
            let yes = MessageBoxW(
                owner,
                wide(summary).as_ptr(),
                wide("AI Slicer Bridge — local approval").as_ptr(),
                MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2 | MB_TOPMOST,
            ) == IDYES;
            Ok(yes)
        }
    }
    fn input(&mut self, w: &Window, a: &Action) -> Result<()> {
        crate::session::check()?;
        let permit = self.permit.take().ok_or("Input has no authorization")?;
        if permit
            != (
                w.pid,
                crate::session::EPOCH.load(std::sync::atomic::Ordering::SeqCst),
            )
        {
            return Err("Authorization revoked before input".into());
        }
        unsafe {
            let h = verify(w)?;
            for key in [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN] {
                if GetAsyncKeyState(key as i32) < 0 {
                    return Err("Release modifier keys before input".into());
                }
            }
            match a {
                Action::Click { x, y } => {
                    let mut p = POINT {
                        x: *x as i32,
                        y: *y as i32,
                    };
                    if ClientToScreen(h, &mut p) == 0 {
                        return Err("ClientToScreen failed".into());
                    }
                    if WindowFromPoint(p) != h && GetAncestor(WindowFromPoint(p), GA_ROOT) != h {
                        return Err("Point belongs to another window".into());
                    }
                    if SetCursorPos(p.x, p.y) == 0 {
                        return Err("Cursor move failed".into());
                    }
                    mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                }
                Action::Key { key } => {
                    let (ctrl, k) = match key.as_str() {
                        "ctrl+i" => (true, b'I'),
                        "ctrl+a" => (true, b'A'),
                        "escape" => (false, 27),
                        "enter" => (false, 13),
                        "tab" => (false, 9),
                        "delete" => (false, 46),
                        "backspace" => (false, 8),
                        "r" => (false, b'R'),
                        "a" => (false, b'A'),
                        "f" => (false, b'F'),
                        _ => return Err("Unsupported key".into()),
                    };
                    if ctrl {
                        keybd_event(VK_CONTROL as u8, 0, 0, 0);
                    }
                    keybd_event(k, 0, 0, 0);
                    keybd_event(k, 0, KEYEVENTF_KEYUP, 0);
                    if ctrl {
                        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
                    }
                }
                Action::Text { text } => {
                    let mut inputs = Vec::new();
                    for ch in text.encode_utf16() {
                        for flags in [KEYEVENTF_UNICODE, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP] {
                            let mut input: INPUT = zeroed();
                            input.r#type = INPUT_KEYBOARD;
                            input.Anonymous.ki = KEYBDINPUT {
                                wVk: 0,
                                wScan: ch,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: 0,
                            };
                            inputs.push(input);
                        }
                    }
                    if SendInput(
                        inputs.len() as u32,
                        inputs.as_ptr(),
                        size_of::<INPUT>() as i32,
                    ) != inputs.len() as u32
                    {
                        return Err("Partial input; observe before retrying".into());
                    }
                }
            }
            Ok(())
        }
    }
}
