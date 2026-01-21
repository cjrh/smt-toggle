use std::fs;
use std::io;
use std::process::Command;

const SMT_CONTROL_PATH: &str = "/sys/devices/system/cpu/smt/control";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtStatus {
    On,
    Off,
    ForceOff,
    NotSupported,
    Unknown,
}

impl SmtStatus {
    pub fn is_enabled(&self) -> bool {
        matches!(self, SmtStatus::On)
    }

    pub fn is_controllable(&self) -> bool {
        matches!(self, SmtStatus::On | SmtStatus::Off)
    }
}

impl From<&str> for SmtStatus {
    fn from(s: &str) -> Self {
        match s.trim() {
            "on" => SmtStatus::On,
            "off" => SmtStatus::Off,
            "forceoff" => SmtStatus::ForceOff,
            "notsupported" => SmtStatus::NotSupported,
            _ => SmtStatus::Unknown,
        }
    }
}

pub fn read_smt_status() -> io::Result<SmtStatus> {
    let content = fs::read_to_string(SMT_CONTROL_PATH)?;
    Ok(SmtStatus::from(content.as_str()))
}

pub fn set_smt_enabled(enabled: bool) -> io::Result<()> {
    let value = if enabled { "on" } else { "off" };

    // Try direct write first
    if fs::write(SMT_CONTROL_PATH, value).is_ok() {
        return Ok(());
    }

    // Fall back to pkexec
    let mut child = Command::new("pkexec")
        .args(["tee", SMT_CONTROL_PATH])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(value.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Failed to set SMT status (pkexec failed)",
        ))
    }
}
