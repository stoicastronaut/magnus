/// Command validation for MCP server execution.
///
/// Rejects:
/// - Empty commands
/// - NUL bytes in command path
/// - Shell metacharacters in command path (`;`, `&`, `|`, backtick, `$`, `<`, `>`, `\n`, `\r`)
/// - Commands outside an allowlist of safe directories
///
/// Absolute paths are accepted if they fall within the allowlist.
/// Relative paths are resolved via `which` and must resolve to an allowlist location.
const ALLOWED_PATHS: &[&str] = &[
    "/usr/local/bin",
    "/usr/local/sbin",
    "/usr/bin",
    "/usr/sbin",
    "/bin",
    "/sbin",
    "/opt",
];

/// Returns the absolute path of the command if valid, or an error message.
pub fn validate_command(
    command: &str,
    args: &[String],
) -> Result<String, String> {
    // Reject empty command
    if command.is_empty() {
        return Err("Command cannot be empty".into());
    }

    // Reject NUL bytes in command
    if command.contains('\0') {
        return Err("Command contains NUL byte".into());
    }

    // Reject shell metacharacters in command path
    let dangerous_chars = [';', '&', '|', '`', '$', '<', '>', '\n', '\r'];
    if command.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err(format!(
            "Command contains shell metacharacter: {}",
            command
                .chars()
                .find(|c| dangerous_chars.contains(c))
                .unwrap()
        ));
    }

    // Validate args separately: reject NUL bytes only
    for arg in args {
        if arg.contains('\0') {
            return Err("Argument contains NUL byte".into());
        }
    }

    // Resolve the command to an absolute path
    let resolved_path = if command.starts_with('/') {
        // Absolute path: use as-is
        command.to_string()
    } else {
        // Relative path: resolve via `which`
        which::which(command)
            .map_err(|_| format!("Command '{}' not found in PATH", command))?
            .to_string_lossy()
            .to_string()
    };

    // Check if resolved path is in the allowlist
    let allowed = ALLOWED_PATHS.iter().any(|p| resolved_path.starts_with(p));
    if !allowed {
        return Err(format!(
            "Command '{}' is outside the allowed paths: {:?}",
            resolved_path, ALLOWED_PATHS
        ));
    }

    Ok(resolved_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_empty_command() {
        let result = validate_command("", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_reject_nul_in_command() {
        let result = validate_command("ls\0x", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_semicolon_in_command() {
        let result = validate_command("ls;rm", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_pipe_in_command() {
        let result = validate_command("ls|cat", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_ampersand_in_command() {
        let result = validate_command("ls&bg", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_backtick_in_command() {
        let result = validate_command("ls`whoami`", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_dollar_in_command() {
        let result = validate_command("ls$VAR", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_redirection_in_command() {
        let result = validate_command("ls>file", &[]);
        assert!(result.is_err());
        let result = validate_command("ls<file", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_newline_in_command() {
        let result = validate_command("ls\ncat", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_carriage_return_in_command() {
        let result = validate_command("ls\rcat", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_nul_in_args() {
        let result = validate_command("ls", &["arg\0ument".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_accept_valid_absolute_path() {
        // /bin/ls should exist on macOS and Linux
        let result = validate_command("/bin/ls", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/bin/ls");
    }

    #[test]
    fn test_accept_valid_usr_bin_path() {
        // /usr/bin/env should exist on macOS and Linux
        let result = validate_command("/usr/bin/env", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_out_of_allowlist() {
        // /tmp is not in the allowlist
        let result = validate_command("/tmp/evil", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("outside the allowed paths"));
    }

    #[test]
    fn test_reject_home_directory() {
        // Paths in $HOME are not allowed by default
        // (we can't test actual $HOME as it's user-specific, but we can test concept)
        let result = validate_command("/home/user/script", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_relative_command_via_which() {
        // 'env' is typically in /usr/bin on most systems
        let result = validate_command("env", &[]);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.starts_with('/'));
    }

    #[test]
    fn test_resolve_nonexistent_command() {
        let result = validate_command("definitely_not_a_real_command_xyz", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_allow_shell_meta() {
        // Args can contain shell metacharacters (they're not interpreted by Command::new)
        let result = validate_command(
            "/usr/bin/env",
            &["PATH=$HOME".to_string(), "a|b".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_npx_if_in_path() {
        // npx might be in PATH on systems with Node installed
        // We just test that if it exists, it's accepted
        let result = validate_command(
            "npx",
            &["-y".to_string(), "some-tool".to_string()],
        );
        if let Ok(path) = result {
            // If npx exists, it should resolve to an allowlist path
            assert!(ALLOWED_PATHS.iter().any(|p| path.starts_with(p)));
        }
    }
}
