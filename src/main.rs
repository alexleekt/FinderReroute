use finder_reroute::detector::DockDetector;
use finder_reroute::launcher::AppLauncher;
use finder_reroute::tap::EventTap;
use finder_reroute::{
    has_accessibility_permission, prompt_accessibility_permission, read_config_app, RerouteState,
};
use std::sync::Arc;

#[allow(clippy::too_many_lines)]
fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    // Handle CLI commands.
    if args.len() > 1 {
        match args[1].as_str() {
            "--install" => {
                println!("Installing LaunchAgent for auto-start...");
                match finder_reroute::launchd::install() {
                    Ok(()) => {
                        println!("✓ LaunchAgent installed.");
                        println!("  The app will auto-start on your next login.");
                        println!(
                            "  Logs: /tmp/{}.out.log",
                            finder_reroute::launchd::LAUNCHD_LABEL
                        );
                        println!();
                        println!("IMPORTANT: Grant Accessibility permission to the app.");
                        println!("  System Settings → Privacy & Security → Accessibility → +");
                        println!("  Add: /Applications/FinderReroute.app");
                        println!("  OR: {}", finder_reroute::launchd::binary_path().display());
                    }
                    Err(e) => {
                        eprintln!("ERROR: {e}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            "--uninstall" => {
                println!("Uninstalling LaunchAgent...");
                match finder_reroute::launchd::uninstall() {
                    Ok(()) => {
                        println!("✓ LaunchAgent uninstalled.");
                    }
                    Err(e) => {
                        eprintln!("ERROR: {e}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            "--status" => {
                finder_reroute::launchd::print_status();
                finder_reroute::shell::print_status();
                return;
            }
            "--setup-shell" => {
                let app_name = read_config_app();
                println!("Installing shell override for `open` command...");
                match finder_reroute::shell::install(&app_name) {
                    Ok(()) => {
                        println!("✓ Shell override installed.");
                        println!("  Directories now open with {app_name}.");
                        println!();
                        println!("  Reload your shell or run:");
                        if finder_reroute::shell::is_fish() {
                            println!("    source ~/.config/fish/config.fish");
                        } else {
                            println!("    source ~/.bashrc   (or ~/.zshrc)");
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR: {e}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            "--uninstall-shell" => {
                println!("Removing shell override for `open` command...");
                match finder_reroute::shell::uninstall() {
                    Ok(()) => {
                        println!("✓ Shell override removed.");
                        println!();
                        println!("  Reload your shell or open a new terminal.");
                    }
                    Err(e) => {
                        eprintln!("ERROR: {e}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    println!("FinderReroute — Intercept Finder clicks and open alternative apps.");
    println!();

    // 1. Check Accessibility permission.
    if !has_accessibility_permission() {
        eprintln!("ERROR: Accessibility permission is required.");
        eprintln!("Please grant it in: System Settings > Privacy & Security > Accessibility");
        prompt_accessibility_permission();
        std::process::exit(1);
    }

    println!("✓ Accessibility permission granted.");

    // 2. Read the configured app from the shared config.
    let app_name = read_config_app();
    println!("✓ Target app: {app_name}");

    // 3. Build the reroute state.
    let state = Arc::new(RerouteState::new(&app_name, "com.apple.finder"));
    let detector = DockDetector::new();
    let launcher = AppLauncher::new(&app_name);

    // 4. Install the event tap.
    let tap = match EventTap::new(state, detector, launcher) {
        Ok(tap) => {
            println!("✓ Event tap installed. Click the Finder icon to open {app_name}.");
            println!("  Press Ctrl-C to quit.");
            tap
        }
        Err(e) => {
            eprintln!("ERROR: Failed to create event tap: {e}");
            eprintln!("Make sure Accessibility and Input Monitoring permissions are granted.");
            std::process::exit(1);
        }
    };

    // 4. Block the thread on the Core Foundation run loop.
    // The tap callback runs on this thread.
    tap.run();
}

fn print_help() {
    println!("FinderReroute — Intercept Finder clicks and open alternative apps.");
    println!();
    println!("Usage:");
    println!("  finder-reroute                  Run the interceptor (foreground)");
    println!("  finder-reroute --install        Install LaunchAgent for auto-start");
    println!("  finder-reroute --uninstall      Remove LaunchAgent");
    println!("  finder-reroute --setup-shell    Install shell override for `open` command");
    println!("  finder-reroute --uninstall-shell  Remove shell override");
    println!("  finder-reroute --status         Show auto-start + shell status");
    println!("  finder-reroute --help           Show this help");
    println!();
    println!("Requires: Accessibility + Input Monitoring permissions.");
}
