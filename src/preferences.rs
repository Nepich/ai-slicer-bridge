use crate::*;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Connection {
    #[default]
    Ask,
    Bambu,
    Orca,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub connection: Connection,
}
impl Preferences {
    pub fn slicer(&self) -> Result<Slicer> {
        match self.connection{Connection::Bambu=>Ok(Slicer::Bambu),Connection::Orca=>Ok(Slicer::Orca),Connection::Ask=>Err("No saved slicer. Ask the user once which slicer to connect to; open it, then save their choice with slicer_preferences. Never guess.".into())}
    }
}
pub fn path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("AI_SLICER_SETTINGS_PATH") {
        return Ok(path.into());
    }
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(root).join("AISlicerBridge/preferences.json"));
    }
    if let Some(root) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(root).join("ai-slicer-bridge/preferences.json"));
    }
    Err("Set AI_SLICER_SETTINGS_PATH to a writable local preferences file".into())
}
pub fn load_at(path: &Path) -> Result<Preferences> {
    match std::fs::read(path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).map_err(|e| format!("Invalid preferences: {e}"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(e) => Err(e.to_string()),
    }
}
pub fn save_at(path: &Path, prefs: &Preferences) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn load() -> Result<Preferences> {
    load_at(&path()?)
}
pub fn save(connection: Connection) -> Result<Preferences> {
    let prefs = Preferences { connection };
    save_at(&path()?, &prefs)?;
    Ok(prefs)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_connection_requires_choice() {
        assert!(Preferences::default().slicer().is_err());
    }
    #[test]
    fn saved_choice_resolves() {
        assert_eq!(
            Preferences {
                connection: Connection::Bambu
            }
            .slicer()
            .unwrap(),
            Slicer::Bambu
        );
        assert_eq!(
            Preferences {
                connection: Connection::Orca
            }
            .slicer()
            .unwrap(),
            Slicer::Orca
        );
    }
    #[test]
    fn no_trust_can_be_persisted() {
        assert!(
            serde_json::from_str::<Preferences>(r#"{"connection":"bambu","full_trust":true}"#)
                .is_err()
        );
    }
    #[test]
    fn persists_across_reads() {
        let path = std::env::temp_dir().join(format!(
            "slicer-prefs-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert_eq!(load_at(&path).unwrap(), Preferences::default());
        let prefs = Preferences {
            connection: Connection::Bambu,
        };
        save_at(&path, &prefs).unwrap();
        assert_eq!(load_at(&path).unwrap(), prefs);
        std::fs::remove_file(path).unwrap();
    }
}
