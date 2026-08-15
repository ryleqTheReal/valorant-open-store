// ======= START UNINSTALL =======

use std::os::windows::process::CommandExt;

use winreg::{RegKey, enums::HKEY_CURRENT_USER};

#[derive(thiserror::Error, Debug)]
pub enum UninstallError {
    #[error("uninstall registry base key open failed: {0}")]
    RegistryOpenFailed(std::io::Error),
    #[error("no uninstall registry entry found for 'open-store'")]
    RegistryKeyNotFound,
    #[error("uninstall string spawn failed: {0}")]
    SpawnFailed(std::io::Error),
}

const UNINSTALL_BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const PRODUCT_NAME: &str = "VALORANT Open Store";
const BUNDLE_ID: &str = "site.valstats.open-store";

const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub fn run() -> Result<(), UninstallError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let uninstall_key = hkcu
        .open_subkey(UNINSTALL_BASE)
        .map_err(UninstallError::RegistryOpenFailed)?;

    let uninstall_cmd = find_uninstall_string(&uninstall_key)?;

    let (program, args) = split_command(&uninstall_cmd);

    std::process::Command::new(program)
        .args(args)
        .creation_flags(CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map_err(UninstallError::SpawnFailed)?;

    Ok(())
}

fn find_uninstall_string(uninstall_key: &RegKey) -> Result<String, UninstallError> {
    for name in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
        let Ok(subkey) = uninstall_key.open_subkey(&name) else {
            continue;
        };
        let display_name: String = subkey.get_value("DisplayName").unwrap_or_default();
        if display_name == PRODUCT_NAME || display_name == BUNDLE_ID || name == BUNDLE_ID {
            let quiet: String = subkey.get_value("QuietUninstallString").unwrap_or_default();
            if !quiet.is_empty() {
                return Ok(quiet);
            }
            let normal: String = subkey.get_value("UninstallString").unwrap_or_default();
            if !normal.is_empty() {
                return Ok(normal);
            }
        }
    }
    Err(UninstallError::RegistryKeyNotFound)
}

fn split_command(cmd: &str) -> (&str, Vec<&str>) {
    let cmd = cmd.trim();
    if let Some(after_quote) = cmd.strip_prefix('"') {
        if let Some(end) = after_quote.find('"') {
            let program = &after_quote[..end];
            let rest = after_quote[end + 1..].trim();
            let args = if rest.is_empty() {
                vec![]
            } else {
                rest.split_whitespace().collect()
            };
            return (program, args);
        }
    }
    if let Some(idx) = cmd.find(' ') {
        let args: Vec<&str> = cmd[idx + 1..].split_whitespace().collect();
        return (&cmd[..idx], args);
    }
    (cmd, vec![])
}

// ======= END UNINSTALL =======

// ======= START TESTS =======

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use winreg::enums::KEY_ALL_ACCESS;

    static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_uninstall_entry(name: &str, f: impl FnOnce(&RegKey)) {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let uninstall_key = hkcu
            .open_subkey_with_flags(UNINSTALL_BASE, KEY_ALL_ACCESS)
            .expect("uninstall base key must exist and be writable under HKCU");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&uninstall_key)));
        let _ = uninstall_key.delete_subkey_all(name);
        result.unwrap();
    }

    #[test]
    fn find_uninstall_string_matches_by_display_name() {
        let name = format!("open-store-test-display-{}", std::process::id());
        with_temp_uninstall_entry(&name, |uninstall_key| {
            let (subkey, _) = uninstall_key.create_subkey(&name).unwrap();
            subkey.set_value("DisplayName", &PRODUCT_NAME).unwrap();
            subkey
                .set_value("QuietUninstallString", &r#""C:\fake\uninstall.exe" /S"#)
                .unwrap();

            let found = find_uninstall_string(uninstall_key).unwrap();
            assert_eq!(found, r#""C:\fake\uninstall.exe" /S"#);
        });
    }

    #[test]
    fn find_uninstall_string_matches_by_key_name_when_display_name_differs() {
        with_temp_uninstall_entry(BUNDLE_ID, |uninstall_key| {
            let (subkey, _) = uninstall_key.create_subkey(BUNDLE_ID).unwrap();
            // real NSIS DisplayName is the product name, not the bundle id
            subkey.set_value("DisplayName", &PRODUCT_NAME).unwrap();
            subkey
                .set_value("UninstallString", &r"C:\fake\uninstall.exe")
                .unwrap();

            let found = find_uninstall_string(uninstall_key).unwrap();
            assert_eq!(found, r"C:\fake\uninstall.exe");
        });
    }

    #[test]
    fn find_uninstall_string_prefers_quiet_over_normal() {
        let name = format!("open-store-test-quiet-{}", std::process::id());
        with_temp_uninstall_entry(&name, |uninstall_key| {
            let (subkey, _) = uninstall_key.create_subkey(&name).unwrap();
            subkey.set_value("DisplayName", &PRODUCT_NAME).unwrap();
            subkey
                .set_value("UninstallString", &r"C:\fake\normal.exe")
                .unwrap();
            subkey
                .set_value("QuietUninstallString", &r"C:\fake\quiet.exe")
                .unwrap();

            let found = find_uninstall_string(uninstall_key).unwrap();
            assert_eq!(found, r"C:\fake\quiet.exe");
        });
    }

    #[test]
    fn split_command_quoted() {
        let (prog, args) = split_command(r#""C:\Program Files\app\uninstall.exe" /S"#);
        assert_eq!(prog, r"C:\Program Files\app\uninstall.exe");
        assert_eq!(args, vec!["/S"]);
    }

    #[test]
    fn split_command_unquoted_no_args() {
        let (prog, args) = split_command(r"C:\app\uninstall.exe");
        assert_eq!(prog, r"C:\app\uninstall.exe");
        assert!(args.is_empty());
    }

    #[test]
    fn split_command_unquoted_with_args() {
        let (prog, args) = split_command(r"C:\app\uninstall.exe /S /quiet");
        assert_eq!(prog, r"C:\app\uninstall.exe");
        assert_eq!(args, vec!["/S", "/quiet"]);
    }

    #[test]
    fn split_command_quoted_no_args() {
        let (prog, args) = split_command(r#""C:\app\uninstall.exe""#);
        assert_eq!(prog, r"C:\app\uninstall.exe");
        assert!(args.is_empty());
    }
}

// ======= END TESTS =======
