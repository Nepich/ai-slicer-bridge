use crate::*;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiNode {
    pub index: usize,
    pub name: String,
    pub automation_id: String,
    pub control_type: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub enabled: bool,
    pub offscreen: bool,
    pub focused: bool,
    pub password: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    pub window: Window,
    pub width: u32,
    pub height: u32,
    pub nodes: Vec<UiNode>,
    pub digest: String,
    pub truncated: bool,
}
impl UiState {
    pub fn seal(&mut self) {
        let mut nodes = self.nodes.clone();
        for node in &mut nodes {
            node.focused = false;
        }
        self.digest = fingerprint(
            &serde_json::to_vec(&(nodes, self.truncated)).unwrap(),
            &self.window,
            self.width,
            self.height,
        );
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UiAction {
    Click { element: usize },
    Key { key: String },
    Text { text: String },
}
pub trait UiDesktop: Desktop {
    fn inspect_ui(&mut self, w: &Window) -> Result<UiState>;
    fn activate(&mut self, w: &Window) -> Result<()>;
}
pub struct UiBridge<D: UiDesktop> {
    pub desktop: D,
    observed: Option<(UiState, Instant)>,
}
impl<D: UiDesktop> UiBridge<D> {
    pub fn new(desktop: D) -> Self {
        Self {
            desktop,
            observed: None,
        }
    }
    pub fn inspect(&mut self, slicer: Slicer, handle: Option<usize>) -> Result<UiState> {
        self.observed = None;
        let ws: Vec<_> = self
            .desktop
            .windows(slicer)?
            .into_iter()
            .filter(|w| handle.is_none_or(|h| w.handle == h))
            .collect();
        if ws.len() != 1 {
            return Err(format!(
                "Select one window: {}",
                serde_json::to_string(&ws).unwrap()
            ));
        }
        let state = self.desktop.inspect_ui(&ws[0])?;
        self.observed = Some((state.clone(), Instant::now()));
        Ok(state)
    }
    pub fn action(&mut self, digest: &str, action: UiAction, purpose: &str) -> Result<UiState> {
        let (state, time) = self
            .observed
            .take()
            .ok_or("Inspect first; UI state is single-use")?;
        if time.elapsed() > Duration::from_secs(60) || digest != state.digest {
            return Err("Stale UI state".into());
        }
        if purpose.trim().is_empty() || purpose.len() > 2000 {
            return Err("Describe intended operation".into());
        }
        let input = match action {
            UiAction::Click { element } => {
                let n = state
                    .nodes
                    .iter()
                    .find(|n| n.index == element)
                    .ok_or("Unknown element")?;
                if !n.enabled || n.offscreen || n.password || n.width <= 0 || n.height <= 0 {
                    return Err("Element not actionable".into());
                }
                let x = n.x + i32::min(n.width / 2, 100);
                let y = n.y + n.height / 2;
                if x < 0 || y < 0 {
                    return Err("Element outside client".into());
                }
                Action::Click {
                    x: x as u32,
                    y: y as u32,
                }
            }
            UiAction::Key { key } => Action::Key { key },
            UiAction::Text { text } => {
                if !state.nodes.iter().any(|n| {
                    n.focused
                        && !n.password
                        && n.enabled
                        && [50003, 50004].contains(&n.control_type)
                }) {
                    return Err("Focus an accessible non-password edit field first".into());
                }
                Action::Text { text }
            }
        };
        input.validate(&Snapshot {
            window: state.window.clone(),
            width: state.width,
            height: state.height,
            digest: String::new(),
            png_base64: String::new(),
        })?;
        self.desktop.activate(&state.window)?;
        let now = self.desktop.inspect_ui(&state.window)?;
        if now.digest != state.digest {
            return Err("UI changed; inspect again".into());
        }
        let message = format!(
            "Target: {} (PID {})\nIntent (untrusted AI text): {}\nExact input: {:?}\n\nAllow ONE action? This may print, upload or delete. Full-session trust can only be granted/revoked in the local panel.",
            state.window.executable, state.window.pid, purpose, input
        );
        if !self.desktop.authorize(&state.window, &message)? {
            return Err("Cancelled by local user".into());
        }
        let now = self.desktop.inspect_ui(&state.window)?;
        if now.digest != state.digest {
            return Err("UI changed during approval".into());
        }
        if matches!(input, Action::Text { .. })
            && state.nodes.iter().find(|n| n.focused).map(|n| n.index)
                != now.nodes.iter().find(|n| n.focused).map(|n| n.index)
        {
            return Err("Text focus changed".into());
        }
        self.desktop.input(&state.window, &input)?;
        self.desktop.inspect_ui(&state.window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock {
        state: UiState,
        inputs: usize,
        allow: bool,
        reads: usize,
        change_at: usize,
    }
    impl Desktop for Mock {
        fn windows(&mut self, _: Slicer) -> Result<Vec<Window>> {
            Ok(vec![self.state.window.clone()])
        }
        fn capture(&mut self, _: &Window) -> Result<Snapshot> {
            panic!("Screenshot path must never run")
        }
        fn confirm(&mut self, _: &str) -> Result<bool> {
            Ok(self.allow)
        }
        fn input(&mut self, _: &Window, _: &Action) -> Result<()> {
            self.inputs += 1;
            Ok(())
        }
    }
    impl UiDesktop for Mock {
        fn inspect_ui(&mut self, _: &Window) -> Result<UiState> {
            self.reads += 1;
            let mut s = self.state.clone();
            if self.reads >= self.change_at {
                s.digest = "changed".into();
            }
            Ok(s)
        }
        fn activate(&mut self, _: &Window) -> Result<()> {
            Ok(())
        }
    }
    fn bridge(allow: bool) -> UiBridge<Mock> {
        let mut state = UiState {
            window: Window {
                handle: 1,
                pid: 2,
                title: "test".into(),
                executable: "bambu-studio.exe".into(),
            },
            width: 100,
            height: 100,
            nodes: vec![UiNode {
                index: 0,
                name: "field".into(),
                automation_id: "1".into(),
                control_type: 50004,
                x: 0,
                y: 0,
                width: 20,
                height: 20,
                enabled: true,
                offscreen: false,
                focused: true,
                password: false,
            }],
            digest: String::new(),
            truncated: false,
        };
        state.seal();
        UiBridge::new(Mock {
            state,
            inputs: 0,
            allow,
            reads: 0,
            change_at: usize::MAX,
        })
    }
    #[test]
    fn no_screenshot_and_single_use() {
        let mut b = bridge(true);
        let s = b.inspect(Slicer::Bambu, None).unwrap();
        b.action(&s.digest, UiAction::Click { element: 0 }, "select")
            .unwrap();
        assert!(
            b.action(&s.digest, UiAction::Click { element: 0 }, "again")
                .is_err()
        );
        assert_eq!(b.desktop.inputs, 1);
    }
    #[test]
    fn denied() {
        let mut b = bridge(false);
        let s = b.inspect(Slicer::Orca, None).unwrap();
        assert!(
            b.action(&s.digest, UiAction::Click { element: 0 }, "print")
                .is_err()
        );
        assert_eq!(b.desktop.inputs, 0);
    }
    #[test]
    fn inaccessible_no_guessing() {
        let mut b = bridge(true);
        let s = b.inspect(Slicer::Bambu, None).unwrap();
        assert!(
            b.action(&s.digest, UiAction::Click { element: 999 }, "select")
                .is_err()
        );
        assert_eq!(b.desktop.inputs, 0);
    }
    #[test]
    fn change_after_approval() {
        let mut b = bridge(true);
        let s = b.inspect(Slicer::Bambu, None).unwrap();
        b.desktop.change_at = 3;
        assert!(
            b.action(&s.digest, UiAction::Click { element: 0 }, "print")
                .is_err()
        );
        assert_eq!(b.desktop.inputs, 0);
    }
    #[test]
    fn password_denied() {
        let mut b = bridge(true);
        b.desktop.state.nodes[0].password = true;
        b.desktop.state.seal();
        let s = b.inspect(Slicer::Bambu, None).unwrap();
        assert!(
            b.action(
                &s.digest,
                UiAction::Text {
                    text: "secret".into()
                },
                "type"
            )
            .is_err()
        );
    }
}
