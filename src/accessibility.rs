use crate::{ui::*, *};
use ::windows::Win32::{Foundation::HWND, System::Com::*, UI::Accessibility::*};
use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    UI::WindowsAndMessaging::*,
};

pub fn inspect(w: &Window) -> Result<UiState> {
    unsafe {
        let hwnd = w.handle as windows_sys::Win32::Foundation::HWND;
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if IsWindow(hwnd) == 0 || pid != w.pid || IsIconic(hwnd) != 0 {
            return Err("Target window changed or minimized".into());
        }
        // Each call owns its COM apartment. No COM pointers cross threads.
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe { CoUninitialize() }
            }
        }
        let _apartment = Apartment;
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| e.to_string())?;
        let root = automation
            .ElementFromHandle(HWND(hwnd))
            .map_err(|e| e.to_string())?;
        let condition = automation
            .CreateTrueCondition()
            .map_err(|e| e.to_string())?;
        let elements = root
            .FindAll(TreeScope_Subtree, &condition)
            .map_err(|e| e.to_string())?;
        let count = elements.Length().map_err(|e| e.to_string())?;
        let mut rect: RECT = std::mem::zeroed();
        if GetClientRect(hwnd, &mut rect) == 0 {
            return Err("Client bounds unavailable".into());
        }
        let mut origin = POINT { x: 0, y: 0 };
        if windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut origin) == 0 {
            return Err("Client origin unavailable".into());
        }
        let mut nodes = Vec::new();
        for i in 0..count.min(1000) {
            let el = elements.GetElement(i).map_err(|e| e.to_string())?;
            // Skip password controls entirely; do not retrieve their names or values.
            if el.CurrentIsPassword().map_err(|e| e.to_string())?.as_bool() {
                continue;
            }
            let r = el.CurrentBoundingRectangle().map_err(|e| e.to_string())?;
            nodes.push(UiNode {
                index: i as usize,
                name: el.CurrentName().map_err(|e| e.to_string())?.to_string(),
                automation_id: el
                    .CurrentAutomationId()
                    .map_err(|e| e.to_string())?
                    .to_string(),
                control_type: el.CurrentControlType().map_err(|e| e.to_string())?.0,
                x: r.left - origin.x,
                y: r.top - origin.y,
                width: r.right - r.left,
                height: r.bottom - r.top,
                enabled: el.CurrentIsEnabled().map_err(|e| e.to_string())?.as_bool(),
                offscreen: el
                    .CurrentIsOffscreen()
                    .map_err(|e| e.to_string())?
                    .as_bool(),
                focused: el
                    .CurrentHasKeyboardFocus()
                    .map_err(|e| e.to_string())?
                    .as_bool(),
                password: false,
            });
        }
        let mut state = UiState {
            window: w.clone(),
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
            nodes,
            digest: String::new(),
            truncated: count > 1000,
        };
        state.seal();
        Ok(state)
    }
}
impl UiDesktop for crate::windows::WindowsDesktop {
    fn inspect_ui(&mut self, w: &Window) -> Result<UiState> {
        inspect(w)
    }
    fn activate(&mut self, w: &Window) -> Result<()> {
        unsafe {
            let hwnd = w.handle as windows_sys::Win32::Foundation::HWND;
            if GetForegroundWindow() != hwnd {
                SetForegroundWindow(hwnd);
            }
            if GetForegroundWindow() != hwnd {
                return Err("Windows blocked focus activation; click the slicer manually".into());
            }
            Ok(())
        }
    }
}
