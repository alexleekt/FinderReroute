use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Detect the current shell.
fn detect_shell() -> Option<String> {
    std::env::var("SHELL").ok()
}

/// Check if the user is using Fish shell.
/// Fish sets the FISH_VERSION environment variable, which is the most
/// reliable way to detect it even when $SHELL points to zsh/bash.
pub fn is_fish() -> bool {
    std::env::var("FISH_VERSION").is_ok()
}

/// Check if the user is using Zsh shell.
fn is_zsh() -> bool {
    if is_fish() {
        return false;
    }
    detect_shell()
        .map(|s| s.contains("zsh"))
        .unwrap_or(false)
}

/// Check if the user is using Bash shell.
fn is_bash() -> bool {
    if is_fish() {
        return false;
    }
    detect_shell()
        .map(|s| s.contains("bash"))
        .unwrap_or(false)
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

/// The marker comment we use to identify our injected code.
const SHELL_MARKER: &str = "# FinderReroute: open command override";

/// Generate the shell code that overrides the `open` command.
fn shell_override_code() -> String {
    if is_fish() {
        format!(
            r#"{}
function open
    if test -d "$argv[1]"
        command open -a Bloom $argv
    else
        command open $argv
    end
end
"#,
            SHELL_MARKER
        )
    } else {
        format!(
            r#"{}
open() {{
    if [ -d "$1" ]; then
        command open -a Bloom "$@"
    else
        command open "$@"
    fi
}}
"#,
            SHELL_MARKER
        )
    }
}

/// Install the shell override so `open` on directories opens in Bloom.
pub fn install() -> Result<(), String> {
    let config = config_path().ok_or("Could not detect shell config file. Supported: fish, zsh, bash")?;
    
    // Create parent directories if needed.
    if let Some(parent) = config.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }

    // Read existing content.
    let mut content = if config.exists() {
        fs::read_to_string(&config)
            .map_err(|e| format!("Failed to read config file: {}", e))?
    } else {
        String::new()
    };

    // Remove any existing override.
    content = remove_override_from_content(&content);

    // Append the new override.
    content.push('\n');
    content.push_str(&shell_override_code());
    content.push('\n');

    // Write back.
    fs::write(&config, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Uninstall the shell override.
pub fn uninstall() -> Result<(), String> {
    let config = config_path().ok_or("Could not detect shell config file")?;

    if !config.exists() {
        return Err("Shell config file not found".to_string());
    }

    let content = fs::read_to_string(&config)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let new_content = remove_override_from_content(&content);

    if new_content == content {
        return Err("Shell override not found in config file".to_string());
    }

    fs::write(&config, new_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Check if the shell override is installed.
pub fn is_installed() -> bool {
    let config = match config_path() {
        Some(p) => p,
        None => return false,
    };

    if !config.exists() {
        return false;
    }

    let content = match fs::read_to_string(&config) {
        Ok(c) => c,
        Err(_) => return false,
    };

    content.contains(SHELL_MARKER)
}

/// Print shell status.
pub fn print_status() {
    let shell = detect_shell().unwrap_or_else(|| "unknown".to_string());
    let config = config_path().map(|p| p.display().to_string()).unwrap_or_else(|| "unknown".to_string());
    let installed = is_installed();

    println!("Shell override status:");
    println!("  Shell:     {}", shell);
    println!("  Config:    {}", config);
    println!("  Installed: {}", if installed { "yes" } else { "no" });
    if installed {
        println!("  Behavior:  Directories open with Bloom; files use default app.");
    }
}

/// Remove our override from shell config content.
fn remove_override_from_content(content: &str) -> String {
    let mut result = String::new();
    let mut skip = false;

    for line in content.lines() {
        if line.trim() == SHELL_MARKER {
            skip = true;
            continue;
        }
        if skip {
            // Skip until we hit a blank line or end of our block.
            // We detect the end by checking if the line is a comment marker 
            // or a function definition that doesn't belong to our block.
            if line.trim().is_empty() {
                skip = false;
            }
            // Continue skipping...
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}
