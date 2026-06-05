use std::fs;
use std::path::PathBuf;

/// Detect the current shell.
fn detect_shell() -> Option<String> {
    std::env::var("SHELL").ok()
}

/// Check if the user is using Fish shell.
/// Fish sets the `FISH_VERSION` environment variable, which is the most
/// reliable way to detect it even when $SHELL points to zsh/bash.
#[must_use]
pub fn is_fish() -> bool {
    std::env::var("FISH_VERSION").is_ok()
}

/// Check if the user is using Zsh shell.
fn is_zsh() -> bool {
    if is_fish() {
        return false;
    }
    detect_shell().is_some_and(|s| s.contains("zsh"))
}

/// Check if the user is using Bash shell.
fn is_bash() -> bool {
    if is_fish() {
        return false;
    }
    detect_shell().is_some_and(|s| s.contains("bash"))
}

/// Path to the shell config file.
fn config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    if is_fish() {
        Some(PathBuf::from(home).join(".config/fish/config.fish"))
    } else if is_zsh() {
        Some(PathBuf::from(home).join(".zshrc"))
    } else if is_bash() {
        Some(PathBuf::from(home).join(".bashrc"))
    } else {
        None
    }
}

/// Start marker for the injected shell override block.
const SHELL_START: &str = "# FinderReroute: open command override BEGIN";
/// End marker for the injected shell override block.
const SHELL_END: &str = "# FinderReroute: open command override END";

/// Generate the shell code that overrides the `open` command.
fn shell_override_code(app_name: &str) -> String {
    if is_fish() {
        format!(
            r#"{SHELL_START}
function open
    if test -d "$argv[1]"
        command open -a {app_name} $argv
    else
        command open $argv
    end
end
{SHELL_END}
"#
        )
    } else {
        format!(
            r#"{SHELL_START}
open() {{
    if [ -d "$1" ]; then
        command open -a {app_name} "$@"
    else
        command open "$@"
    fi
}}
{SHELL_END}
"#
        )
    }
}

/// Install the shell override so `open` on directories opens in the target app.
///
/// # Arguments
///
/// * `app_name` — The name of the app to open directories with (e.g., "Bloom").
///
/// # Errors
///
/// Returns an error if the shell config file cannot be read or written.
pub fn install(app_name: &str) -> Result<(), String> {
    let config =
        config_path().ok_or("Could not detect shell config file. Supported: fish, zsh, bash")?;

    // Create parent directories if needed.
    if let Some(parent) = config.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
    }

    // Read existing content.
    let mut content = if config.exists() {
        fs::read_to_string(&config).map_err(|e| format!("Failed to read config file: {e}"))?
    } else {
        String::new()
    };

    // Remove any existing override.
    content = remove_override_from_content(&content);

    // Append the new override.
    content.push('\n');
    content.push_str(&shell_override_code(app_name));
    content.push('\n');

    // Write atomically (tmp file + rename) to avoid corrupting the shell config.
    let tmp = config.with_extension("tmp");
    fs::write(&tmp, content).map_err(|e| format!("Failed to write temp file: {e}"))?;
    fs::rename(&tmp, &config).map_err(|e| format!("Failed to rename temp file: {e}"))?;

    Ok(())
}

/// Uninstall the shell override.
///
/// # Errors
///
/// Returns an error if the shell config file cannot be read or written.
pub fn uninstall() -> Result<(), String> {
    let config = config_path().ok_or("Could not detect shell config file")?;

    if !config.exists() {
        return Err("Shell config file not found".to_string());
    }

    let content =
        fs::read_to_string(&config).map_err(|e| format!("Failed to read config file: {e}"))?;

    let new_content = remove_override_from_content(&content);

    if new_content == content {
        return Err("Shell override not found in config file".to_string());
    }

    fs::write(&config, new_content).map_err(|e| format!("Failed to write config file: {e}"))?;

    Ok(())
}

/// Check if the shell override is installed.
#[must_use]
pub fn is_installed() -> bool {
    let Some(config) = config_path() else {
        return false;
    };

    if !config.exists() {
        return false;
    }

    let Ok(content) = fs::read_to_string(&config) else {
        return false;
    };

    content.contains(SHELL_START)
}

/// Print shell status.
pub fn print_status() {
    let shell = detect_shell().unwrap_or_else(|| "unknown".to_string());
    let config = config_path().map_or_else(|| "unknown".to_string(), |p| p.display().to_string());
    let installed = is_installed();

    println!("Shell override status:");
    println!("  Shell:     {shell}");
    println!("  Config:    {config}");
    println!("  Installed: {}", if installed { "yes" } else { "no" });
    if installed {
        println!("  Behavior:  Directories open with the configured app; files use default app.");
    }
}

/// Remove our override block from shell config content.
///
/// Matches lines between `SHELL_START` and `SHELL_END` (inclusive) and
/// strips them, preserving the rest of the file.
fn remove_override_from_content(content: &str) -> String {
    let mut result = String::new();
    let mut skip = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == SHELL_START {
            skip = true;
            continue;
        }
        if trimmed == SHELL_END {
            skip = false;
            continue;
        }
        if !skip {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}
