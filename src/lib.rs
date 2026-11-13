use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Slicer {
    Bambu,
    Orca,
}
pub trait Adapter {
    fn executable(&self) -> &'static str;
    fn label(&self) -> &'static str;
}
pub struct BambuAdapter;
pub struct OrcaAdapter;
impl Adapter for BambuAdapter {
    fn executable(&self) -> &'static str {
        "bambu-studio.exe"
    }
    fn label(&self) -> &'static str {
        "Bambu Studio"
    }
}
impl Adapter for OrcaAdapter {
    fn executable(&self) -> &'static str {
        "orca-slicer.exe"
    }
    fn label(&self) -> &'static str {
        "OrcaSlicer"
    }
}
impl Slicer {
    pub fn adapter(self) -> Box<dyn Adapter> {
        match self {
            Self::Bambu => Box::new(BambuAdapter),
            Self::Orca => Box::new(OrcaAdapter),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Window {
    pub handle: usize,
    pub pid: u32,
    pub title: String,
    pub executable: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub window: Window,
    pub width: u32,
    pub height: u32,
    pub digest: String,
    pub png_base64: String,
}
pub fn fingerprint(bytes: &[u8], window: &Window, width: u32, height: u32) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.update(window.handle.to_le_bytes());
    h.update(window.pid.to_le_bytes());
    h.update(window.title.as_bytes());
    h.update(width.to_le_bytes());
    h.update(height.to_le_bytes());
    format!("{:x}", h.finalize())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Click { x: u32, y: u32 },
    Key { key: String },
    Text { text: String },
}
impl Action {
    pub fn validate(&self, s: &Snapshot) -> Result<()> {
        match self {
            Self::Click { x, y } if *x >= s.width || *y >= s.height => {
                Err("Point outside captured window".into())
            }
            Self::Key { key }
                if ![
                    "ctrl+i",
                    "ctrl+a",
                    "escape",
                    "enter",
                    "tab",
                    "r",
                    "a",
                    "f",
                    "delete",
                    "backspace",
                ]
                .contains(&key.as_str()) =>
            {
                Err("Key not allowed".into())
            }
            Self::Text { text }
                if text.is_empty() || text.len() > 4096 || text.chars().any(char::is_control) =>
            {
                Err("Invalid literal text".into())
            }
            _ => Ok(()),
        }
    }
}
pub trait Desktop {
    fn windows(&mut self, slicer: Slicer) -> Result<Vec<Window>>;
    fn capture(&mut self, w: &Window) -> Result<Snapshot>;
    fn confirm(&mut self, summary: &str) -> Result<bool>;
    fn input(&mut self, w: &Window, action: &Action) -> Result<()>;
    fn authorize(&mut self, _w: &Window, summary: &str) -> Result<bool> {
        self.confirm(summary)
    }
}
pub struct Bridge<D: Desktop> {
    pub desktop: D,
    observed: Option<Snapshot>,
}
impl<D: Desktop> Bridge<D> {
    pub fn new(desktop: D) -> Self {
        Self {
            desktop,
            observed: None,
        }
    }
    pub fn observe(&mut self, slicer: Slicer, handle: Option<usize>) -> Result<Snapshot> {
        self.observed = None;
        let ws = self.desktop.windows(slicer)?;
        let selected: Vec<_> = ws
            .into_iter()
            .filter(|w| handle.is_none_or(|h| w.handle == h))
            .collect();
        if selected.len() != 1 {
            return Err(format!(
                "Choose exactly one visible target window; candidates: {}",
                serde_json::to_string(&selected).unwrap()
            ));
        }
        let s = self.desktop.capture(&selected[0])?;
        self.observed = Some(s.clone());
        Ok(s)
    }
    pub fn act(&mut self, digest: &str, action: Action, purpose: &str) -> Result<Snapshot> {
        let s = self
            .observed
            .take()
            .ok_or("Observe first; observations are single-use")?;
        if digest != s.digest {
            return Err("Stale observation".into());
        }
        if purpose.trim().is_empty() || purpose.len() > 2000 {
            return Err("Describe intended operation".into());
        }
        action.validate(&s)?;
        let now = self.desktop.capture(&s.window)?;
        if now.digest != s.digest {
            return Err("Window changed; observe again".into());
        }
        let message = format!(
            "Target: {} (PID {})\nIntent (untrusted AI text): {}\nExact input: {:?}\n\nThis input may send a print, upload data, or delete a model. Verify the visible slicer before allowing ONE action. Cancel if uncertain.",
            s.window.executable, s.window.pid, purpose, action
        );
        if !self.desktop.authorize(&s.window, &message)? {
            return Err("Cancelled by local user".into());
        }
        let now = self.desktop.capture(&s.window)?;
        if now.digest != s.digest {
            return Err("Window changed during approval; observe again".into());
        }
        self.desktop.input(&s.window, &action)?;
        // Result is deliberately not reusable: observe again after asynchronous UI settles.
        self.desktop.capture(&s.window)
    }
}

#[cfg(windows)]
pub mod windows;

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock {
        snapshot: Snapshot,
        consent: bool,
        inputs: usize,
        captures: usize,
        change_at: usize,
    }
    impl Desktop for Mock {
        fn windows(&mut self, _: Slicer) -> Result<Vec<Window>> {
            Ok(vec![self.snapshot.window.clone()])
        }
        fn capture(&mut self, _: &Window) -> Result<Snapshot> {
            self.captures += 1;
            let mut s = self.snapshot.clone();
            if self.captures >= self.change_at {
                s.digest = "changed".into();
            }
            Ok(s)
        }
        fn confirm(&mut self, _: &str) -> Result<bool> {
            Ok(self.consent)
        }
        fn input(&mut self, _: &Window, _: &Action) -> Result<()> {
            self.inputs += 1;
            Ok(())
        }
    }
    fn bridge(consent: bool) -> Bridge<Mock> {
        Bridge::new(Mock {
            snapshot: Snapshot {
                window: Window {
                    handle: 1,
                    pid: 2,
                    title: "test".into(),
                    executable: "bambu-studio.exe".into(),
                },
                width: 100,
                height: 100,
                digest: "fresh".into(),
                png_base64: String::new(),
            },
            consent,
            inputs: 0,
            captures: 0,
            change_at: usize::MAX,
        })
    }
    fn click() -> Action {
        Action::Click { x: 10, y: 10 }
    }
    #[test]
    fn approval_required() {
        let mut b = bridge(false);
        b.observe(Slicer::Bambu, None).unwrap();
        assert!(b.act("fresh", click(), "slice").is_err());
        assert_eq!(b.desktop.inputs, 0);
    }
    #[test]
    fn single_use() {
        let mut b = bridge(true);
        b.observe(Slicer::Bambu, None).unwrap();
        b.act("fresh", click(), "slice").unwrap();
        assert!(b.act("fresh", click(), "slice").is_err());
        assert_eq!(b.desktop.inputs, 1);
    }
    #[test]
    fn stale_rejected() {
        let mut b = bridge(true);
        b.observe(Slicer::Orca, None).unwrap();
        assert!(b.act("old", click(), "slice").is_err());
        assert_eq!(b.desktop.inputs, 0);
    }
    #[test]
    fn change_during_confirmation() {
        let mut b = bridge(true);
        b.observe(Slicer::Bambu, None).unwrap();
        b.desktop.change_at = 3;
        assert!(b.act("fresh", click(), "print").is_err());
        assert_eq!(b.desktop.inputs, 0);
    }
    #[test]
    fn bounds() {
        let mut b = bridge(true);
        b.observe(Slicer::Bambu, None).unwrap();
        assert!(
            b.act("fresh", Action::Click { x: 100, y: 1 }, "slice")
                .is_err()
        );
    }
    #[test]
    fn escape_injection() {
        let mut b = bridge(true);
        b.observe(Slicer::Bambu, None).unwrap();
        assert!(
            b.act(
                "fresh",
                Action::Text {
                    text: "file\ncommand".into()
                },
                "import"
            )
            .is_err()
        );
    }
    #[test]
    fn adapters() {
        assert_eq!(Slicer::Bambu.adapter().executable(), "bambu-studio.exe");
        assert_eq!(Slicer::Orca.adapter().executable(), "orca-slicer.exe");
    }
    #[test]
    fn reject_unknown_action() {
        assert!(serde_json::from_str::<Action>(r#"{"kind":"shell","command":"calc"}"#).is_err());
    }
}

#[cfg(windows)]
pub mod accessibility;
#[cfg(windows)]
pub mod panel;
pub mod preferences;
#[cfg(windows)]
pub mod session;
pub mod ui;
