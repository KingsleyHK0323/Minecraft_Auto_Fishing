//! Where the app keeps its own data: folders named after the bundle identifier, apart from the
//! install folder (`%LOCALAPPDATA%\Minecraft Auto Fishing`). They are the folders the Windows
//! uninstaller's "Delete the application data" option removes.
use std::fs::{self, OpenOptions};
use std::io::{self, copy};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_MIGRATION_TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Must match `identifier` in tauri.conf.json (checked by a test).
// Updated by: OpenAI-Codex
pub const IDENTIFIER: &str = "io.github.kingsleyhk0323.minecraftautofishing";

/// Previous identifier; retained for the one-time, non-destructive data migration.
const LEGACY_IDENTIFIER: &str = "io.github.spacesquare640.minecraftautofishing";
const MIGRATION_MARKER: &str = ".legacy-identifier-migration-complete";

/// Folder name used before 0.1.0-alpha.2. Only the settings file is carried over from it.
pub const OLD_NAME: &str = "Minecraft Auto Fishing";

/// Settings: `%APPDATA%\<identifier>`.
pub fn config_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join(IDENTIFIER))
}

/// Local data such as log files: `%LOCALAPPDATA%\<identifier>`.
pub fn local_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(IDENTIFIER))
}

fn legacy_config_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join(LEGACY_IDENTIFIER))
}

fn legacy_local_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(LEGACY_IDENTIFIER))
}

/// Copy settings and log files from the previous identifier without removing or overwriting
/// existing files. Errors are returned so the app can report them and retry on its next start.
pub fn migrate_legacy_data() -> Vec<String> {
    let (Some(old_config), Some(new_config), Some(old_local), Some(new_local)) =
        (legacy_config_dir(), config_dir(), legacy_local_dir(), local_dir())
    else {
        return vec![
            "Legacy data migration skipped because an app-data path is unavailable.".into(),
        ];
    };
    migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local)
}

fn migrate_legacy_data_at(
    old_config: &Path,
    new_config: &Path,
    old_local: &Path,
    new_local: &Path,
) -> Vec<String> {
    let marker = new_config.join(MIGRATION_MARKER);
    if marker.exists() {
        return Vec::new();
    }

    let mut errors = Vec::new();
    if let Err(error) = copy_file_if_missing(
        &old_config.join("settings.json"),
        &new_config.join("settings.json"),
    ) {
        errors.push(format!("Could not migrate settings: {error}"));
    }
    if let Err(error) = copy_log_files(&old_local.join("logs"), &new_local.join("logs")) {
        errors.push(format!("Could not migrate log files: {error}"));
    }

    if errors.is_empty() {
        if let Err(error) =
            fs::create_dir_all(new_config).and_then(|()| fs::write(marker, b"done\n"))
        {
            errors.push(format!("Could not mark data migration complete: {error}"));
        }
    }
    errors
}

fn copy_log_files(source_dir: &Path, destination_dir: &Path) -> io::Result<()> {
    let source_type = match fs::symlink_metadata(source_dir) {
        Ok(metadata) => metadata.file_type(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if !source_type.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "log source is not a directory",
        ));
    }

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().is_some_and(|extension| extension == "log")
        {
            copy_file_if_missing(&entry.path(), &destination_dir.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub(crate) fn copy_file_if_missing(source: &Path, destination: &Path) -> io::Result<()> {
    let source_type = match fs::symlink_metadata(source) {
        Ok(metadata) => metadata.file_type(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if !source_type.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "copy source is not a regular file",
        ));
    }
    match fs::symlink_metadata(destination) {
        Ok(metadata) if metadata.file_type().is_file() => return Ok(()),
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "copy destination exists and is not a regular file",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut input = fs::File::open(source)?;
    let source_len = input.metadata()?.len();
    let (staging_path, mut output) = create_staging_file(destination)?;
    let result = (|| {
        let copied = copy(&mut input, &mut output)?;
        if copied != source_len || input.metadata()?.len() != source_len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "source changed while copying",
            ));
        }
        output.sync_all()
    })();
    drop(output);
    if let Err(error) = result {
        let _ = fs::remove_file(&staging_path);
        return Err(error);
    }

    match publish_staging_file(&staging_path, destination) {
        Ok(()) => {
            #[cfg(not(windows))]
            let _ = fs::remove_file(staging_path);
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(staging_path);
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(staging_path);
            Err(error)
        }
    }
}

fn create_staging_file(destination: &Path) -> io::Result<(PathBuf, fs::File)> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destination has no filename"))?
        .to_string_lossy();

    for _ in 0..128 {
        let id = NEXT_MIGRATION_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let staging_path = parent.join(format!(
            ".{name}.legacy-migration-{}-{id}.tmp",
            std::process::id()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging_path)
        {
            Ok(file) => return Ok((staging_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique migration staging file",
    ))
}

#[cfg(windows)]
fn publish_staging_file(staging_path: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileW(existing_file_name: *const u16, new_file_name: *const u16) -> i32;
    }

    let staging_path = staging_path.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>();
    let destination = destination.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>();
    // SAFETY: Both pointers reference NUL-terminated UTF-16 paths for the duration of the call.
    let moved = unsafe { MoveFileW(staging_path.as_ptr(), destination.as_ptr()) };
    if moved == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn publish_staging_file(staging_path: &Path, destination: &Path) -> io::Result<()> {
    fs::hard_link(staging_path, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-tmp")
            .join(format!("identifier_migration_{unique}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn identifier_matches_tauri_conf() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(conf["identifier"], IDENTIFIER);
    }

    #[test]
    fn migration_copies_settings_and_logs_and_keeps_legacy_files() {
        let root = scratch();
        let old_config = root.join("old-config");
        let new_config = root.join("new-config");
        let old_local = root.join("old-local");
        let new_local = root.join("new-local");
        fs::create_dir_all(&old_config).unwrap();
        fs::create_dir_all(old_local.join("logs")).unwrap();
        fs::write(old_config.join("settings.json"), b"{\"saveLogs\":true}").unwrap();
        fs::write(old_local.join("logs/2026-11-16.log"), b"old log\n").unwrap();

        assert!(
            migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );

        assert_eq!(fs::read(new_config.join("settings.json")).unwrap(), b"{\"saveLogs\":true}");
        assert_eq!(fs::read(new_local.join("logs/2026-11-16.log")).unwrap(), b"old log\n");
        assert!(old_config.join("settings.json").exists());
        assert!(old_local.join("logs/2026-11-16.log").exists());
        assert!(new_config.join(MIGRATION_MARKER).exists());
    }

    #[test]
    fn migration_does_not_overwrite_new_data_or_repeat_after_completion() {
        let root = scratch();
        let old_config = root.join("old-config");
        let new_config = root.join("new-config");
        let old_local = root.join("old-local");
        let new_local = root.join("new-local");
        fs::create_dir_all(&old_config).unwrap();
        fs::create_dir_all(&new_config).unwrap();
        fs::create_dir_all(old_local.join("logs")).unwrap();
        fs::create_dir_all(new_local.join("logs")).unwrap();
        fs::write(old_config.join("settings.json"), b"old settings").unwrap();
        fs::write(new_config.join("settings.json"), b"new settings").unwrap();
        fs::write(old_local.join("logs/2026-11-16.log"), b"old log\n").unwrap();
        fs::write(new_local.join("logs/2026-11-16.log"), b"new log\n").unwrap();

        assert!(
            migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );

        assert_eq!(fs::read(new_config.join("settings.json")).unwrap(), b"new settings");
        assert_eq!(fs::read(new_local.join("logs/2026-11-16.log")).unwrap(), b"new log\n");
        fs::write(old_local.join("logs/2026-11-17.log"), b"later old log\n").unwrap();
        assert!(
            migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );
        assert!(!new_local.join("logs/2026-11-17.log").exists());
        assert!(old_local.join("logs/2026-11-17.log").exists());
    }

    #[test]
    fn migration_retries_after_copy_error_without_marking_complete() {
        let root = scratch();
        let old_config = root.join("old-config");
        let new_config = root.join("new-config");
        let old_local = root.join("old-local");
        let new_local = root.join("new-local");
        fs::create_dir_all(&old_config).unwrap();
        fs::write(old_config.join("settings.json"), b"old settings").unwrap();
        fs::write(&new_config, b"blocks directory creation").unwrap();

        assert!(
            !migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );
        assert!(old_config.join("settings.json").exists());
        assert!(!new_config.join(MIGRATION_MARKER).exists());

        let obstruction = root.join("new-config-obstruction");
        fs::rename(&new_config, &obstruction).unwrap();
        fs::create_dir(&new_config).unwrap();
        assert!(
            migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );

        assert_eq!(fs::read(new_config.join("settings.json")).unwrap(), b"old settings");
        assert!(old_config.join("settings.json").exists());
        assert!(obstruction.exists());
        assert!(new_config.join(MIGRATION_MARKER).exists());
    }

    #[test]
    fn incomplete_staging_file_is_not_exposed_as_the_destination() {
        let root = scratch();
        let destination = root.join("settings.json");
        let (staging_path, mut staging_file) = create_staging_file(&destination).unwrap();
        staging_file.write_all(b"partial settings").unwrap();
        drop(staging_file);

        assert!(staging_path.exists());
        assert!(!destination.exists());
    }

    #[test]
    fn migration_retries_when_settings_destination_is_not_a_file() {
        let root = scratch();
        let old_config = root.join("old-config");
        let new_config = root.join("new-config");
        let old_local = root.join("old-local");
        let new_local = root.join("new-local");
        let blocked_settings = new_config.join("settings.json");
        fs::create_dir_all(&old_config).unwrap();
        fs::create_dir_all(&new_config).unwrap();
        fs::create_dir_all(&blocked_settings).unwrap();
        fs::write(old_config.join("settings.json"), b"old settings").unwrap();

        assert!(
            !migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );
        assert!(old_config.join("settings.json").exists());
        assert!(!new_config.join(MIGRATION_MARKER).exists());

        let preserved_obstruction = root.join("settings-directory-obstruction");
        fs::rename(&blocked_settings, &preserved_obstruction).unwrap();
        assert!(
            migrate_legacy_data_at(&old_config, &new_config, &old_local, &new_local).is_empty()
        );

        assert_eq!(fs::read(new_config.join("settings.json")).unwrap(), b"old settings");
        assert!(old_config.join("settings.json").exists());
        assert!(preserved_obstruction.is_dir());
        assert!(new_config.join(MIGRATION_MARKER).exists());
    }
}
