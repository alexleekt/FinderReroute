use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Identifier for the LaunchAgent.
pub const LAUNCHD_LABEL: &str = "com.alexleekt.finder-reroute";

/// Returns the path to the LaunchAgent plist file.
pub fn plist_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME env var not set");
    PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LAUNCHD_LABEL))
}

/// Returns the path to the current binary.
/// 
/// This is public so the CLI can show the exact path to the user for
/// granting Accessibility permission.
pub fn binary_path() -> PathBuf {
    // In the app bundle, the executable is at FinderReroute.app/Contents/MacOS/FinderReroute
    std::env::current_exe().expect("Failed to get current executable path")
}

/// Returns the path to the .app bundle in /Applications.
pub fn app_bundle_path() -> PathBuf {
    PathBuf::from("/Applications/FinderReroute.app")
}

/// Generate the LaunchAgent plist XML.
fn plist_content() -> String {
    let app_path = app_bundle_path().display().to_string();
    let binary = format!("{}/Contents/MacOS/FinderReroute", app_path);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/{}.out.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/{}.err.log</string>
</dict>
</plist>"#,
        LAUNCHD_LABEL, binary, LAUNCHD_LABEL, LAUNCHD_LABEL
    )
}

/// Install the LaunchAgent so the app auto-starts on login.
pub fn install() -> Result<(), String> {
    let plist = plist_path();
    let agents_dir = plist.parent().unwrap();

    // Create LaunchAgents directory if it doesn't exist.
    if !agents_dir.exists() {
        fs::create_dir_all(agents_dir).map_err(|e| format!("Failed to create LaunchAgents dir: {}", e))?;
    }

    // Write the plist file.
    let content = plist_content();
    fs::write(&plist, content).map_err(|e| format!("Failed to write plist: {}", e))?;

    // Load the LaunchAgent.
    let status = Command::new("launchctl")
        .args(["load", "-w", plist.to_str().unwrap()])
        .status()
        .map_err(|e| format!("Failed to run launchctl load: {}", e))?;

    if !status.success() {
        return Err("launchctl load failed".to_string());
    }

    // Start it now.
    let status = Command::new("launchctl")
        .args(["start", LAUNCHD_LABEL])
        .status()
        .map_err(|e| format!("Failed to run launchctl start: {}", e))?;

    if !status.success() {
        return Err("launchctl start failed".to_string());
    }

    Ok(())
}

/// Uninstall the LaunchAgent.
pub fn uninstall() -> Result<(), String> {
    let plist = plist_path();

    if !plist.exists() {
        return Err("LaunchAgent is not installed".to_string());
    }

    // Unload the LaunchAgent.
    let status = Command::new("launchctl")
        .args(["unload", "-w", plist.to_str().unwrap()])
        .status()
        .map_err(|e| format!("Failed to run launchctl unload: {}", e))?;

    if !status.success() {
        return Err("launchctl unload failed".to_string());
    }

    // Remove the plist file.
    fs::remove_file(&plist).map_err(|e| format!("Failed to remove plist: {}", e))?;

    Ok(())
}

/// Check if the LaunchAgent is installed.
pub fn is_installed() -> bool {
    plist_path().exists()
}

/// Check if the LaunchAgent is currently running.
pub fn is_running() -> bool {
    let output = Command::new("launchctl")
        .args(["list", LAUNCHD_LABEL])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        });
    output.status.success()
}

/// Print status info.
pub fn print_status() {
    let installed = is_installed();
    let running = is_running();

    println!("LaunchAgent status:");
    println!("  Installed:   {}", if installed { "yes" } else { "no" });
    println!("  Running:     {}", if running { "yes" } else { "no" });
    println!("  Plist:       {}", plist_path().display());
    println!("  App Bundle:  /Applications/FinderReroute.app");
    println!("  Binary:      {}", binary_path().display());
}
