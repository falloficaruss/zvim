use std::env;

use zvim_app::{run_app_shell, ZvimApp};

fn main() {
    if has_graphical_display() {
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

fn has_graphical_display() -> bool {
    env::var_os("WAYLAND_DISPLAY").is_some() || env::var_os("DISPLAY").is_some()
}

#[cfg(test)]
mod tests {
    use super::has_graphical_display;

    #[test]
    fn headless_detection_is_stable() {
        let _ = has_graphical_display();
    }
}
