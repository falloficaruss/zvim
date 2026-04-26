use std::env;

use zvim_app::{run_app_shell, ZvimApp};

fn main() {
    if should_run_graphical_shell() {
        if let Err(error) = run_app_shell() {
            eprintln!("zvim bootstrap failed: {error}");
            std::process::exit(1);
        }
        return;
    }

    match ZvimApp::bootstrap() {
        Ok(app) => println!("{}", app.render_boot_report()),
        Err(error) => {
            eprintln!("zvim bootstrap failed: {error}");
            std::process::exit(1);
        }
    }
}

fn should_run_graphical_shell() -> bool {
    if env::var_os("ZVIM_HEADLESS").is_some() || env::var_os("CI").is_some() {
        return false;
    }

    platform_graphical_session_available()
}

#[cfg(target_os = "linux")]
fn platform_graphical_session_available() -> bool {
    env::var_os("WAYLAND_DISPLAY").is_some() || env::var_os("DISPLAY").is_some()
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn platform_graphical_session_available() -> bool {
    true
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_graphical_session_available() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::platform_graphical_session_available;
    use super::should_run_graphical_shell;
    use std::env;

    #[test]
    fn platform_detection_is_stable() {
        let _ = platform_graphical_session_available();
    }

    #[test]
    fn explicit_headless_mode_disables_graphical_shell() {
        env::set_var("ZVIM_HEADLESS", "1");
        assert!(!should_run_graphical_shell());
        env::remove_var("ZVIM_HEADLESS");
    }
}
