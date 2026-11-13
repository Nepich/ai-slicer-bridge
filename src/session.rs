use crate::*;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
};
pub static PAUSED: AtomicBool = AtomicBool::new(false);
pub static EPOCH: AtomicU64 = AtomicU64::new(0);
static TRUSTED_PID: AtomicU32 = AtomicU32::new(0);
pub static STATUS: Mutex<String> = Mutex::new(String::new());
pub fn status(text: &str) {
    if let Ok(mut s) = STATUS.lock() {
        *s = text.to_owned();
    }
}
pub fn revoke() {
    EPOCH.fetch_add(1, Ordering::SeqCst);
    TRUSTED_PID.store(0, Ordering::SeqCst);
}
pub fn pause() {
    PAUSED.store(true, Ordering::SeqCst);
    revoke();
    status("STOPPED. Trust revoked. No further inputs permitted.");
}
pub fn resume() {
    revoke();
    PAUSED.store(false, Ordering::SeqCst);
    status("Ready. Every input requires local approval.");
}
pub fn grant(pid: u32) {
    EPOCH.fetch_add(1, Ordering::SeqCst);
    TRUSTED_PID.store(pid, Ordering::SeqCst);
    PAUSED.store(false, Ordering::SeqCst);
    status(&format!(
        "FULL TRUST for PID {pid}: includes print, upload and delete. Stop revokes."
    ));
}
pub fn trusted(pid: u32) -> bool {
    pid != 0 && TRUSTED_PID.load(Ordering::SeqCst) == pid
}
pub fn check() -> Result<()> {
    if PAUSED.load(Ordering::SeqCst) {
        Err("Agent stopped in local panel".into())
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_trust_scope_and_revocation() {
        grant(123);
        assert!(trusted(123));
        assert!(!trusted(124));
        pause();
        assert!(!trusted(123));
        assert!(check().is_err());
        resume();
        assert!(!trusted(123));
        assert!(check().is_ok());
    }
}
