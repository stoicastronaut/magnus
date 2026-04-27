use std::fs;
use std::path::Path;

/// On Unix systems, restrict file permissions to 0600 (owner read/write only).
/// On other systems, this is a no-op.
#[cfg(unix)]
pub fn restrict_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
}

/// On non-Unix systems, this is a no-op.
#[cfg(not(unix))]
pub fn restrict_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg(unix)]
    fn test_restrict_permissions_sets_mode_to_0600() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // Set initial permissions to something different
        let initial_perms = fs::Permissions::from_mode(0o644);
        fs::set_permissions(path, initial_perms).unwrap();

        // Apply restriction
        restrict_permissions(path).unwrap();

        // Verify it's now 0600
        let perms = fs::metadata(path).unwrap().permissions();
        use std::os::unix::fs::PermissionsExt;
        let mode = perms.mode() & 0o777;
        assert_eq!(mode, 0o600, "Expected mode 0600, got {:#o}", mode);
    }

    #[test]
    #[cfg(unix)]
    fn test_restrict_permissions_on_nonexistent_file_fails() {
        let result = restrict_permissions(Path::new("/nonexistent/file/path"));
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(unix))]
    fn test_restrict_permissions_noop_on_non_unix() {
        let file = NamedTempFile::new().unwrap();
        let result = restrict_permissions(file.path());
        assert!(result.is_ok());
    }
}
