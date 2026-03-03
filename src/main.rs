mod eww;
mod hypr;

use crate::hypr::{get_active_monitor_id, get_monitors_list};
use chrono::Local;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use which::which;

fn capture_screenshot(args: &[&str], path: &str) -> bool {
    thread::sleep(Duration::from_millis(200));
    Command::new("grim")
        .args(args)
        .arg(path)
        .status()
        .map_or(false, |s| s.success())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-V" => {
                println!("dumbshot {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                println!("dumbshot {}", env!("CARGO_PKG_VERSION"));
                println!("");
                println!(
                    "An elegant, painless one-click screenshot utility for Wayland (grim + slurp)"
                );
                println!("");
                println!("Usage: dumbshot [OPTIONS]");
                println!("");
                println!("Options:");
                println!("  -h, --help     Print help");
                println!("  -V, --version  Print version");
                return;
            }
            _ => {}
        }
    }

    if which("eww").is_err() || which("grim").is_err() || which("slurp").is_err() {
        eprintln!("Error: 'eww', 'grim' and 'slurp' are required.");
        return;
    }

    let main_menu = vec![
        eww::MenuOption {
            label: "Area".into(),
            id: "area".into(),
        },
        eww::MenuOption {
            label: "Monitor".into(),
            id: "monitor".into(),
        },
        eww::MenuOption {
            label: "All".into(),
            id: "all".into(),
        },
    ];

    let choice = match eww::run_eww_menu("Screenshot Tool", &main_menu) {
        Some(c) => c,
        None => return,
    };

    let tmp_path =
        std::env::temp_dir().join(format!("shot-{}.png", Local::now().format("%Y%m%d%H%M%S")));
    let tmp_str = tmp_path.to_string_lossy();

    let success = match choice.as_str() {
        "area" => {
            let geom = Command::new("slurp")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            if let Some(g) = geom {
                capture_screenshot(&["-g", &g], &tmp_str)
            } else {
                false
            }
        }
        "monitor" => get_monitors_list().map_or(false, |monitors| {
            let id = get_active_monitor_id() as usize;
            let m = monitors.get(id).unwrap_or(&monitors[0]);
            capture_screenshot(&["-o", m], &tmp_str)
        }),
        "all" => capture_screenshot(&[], &tmp_str),
        _ => false,
    };

    if !success {
        return;
    }

    let actions_menu = vec![
        eww::MenuOption {
            label: "Save".into(),
            id: "save".into(),
        },
        eww::MenuOption {
            label: "Copy".into(),
            id: "copy".into(),
        },
        eww::MenuOption {
            label: "Edit".into(),
            id: "edit".into(),
        },
        eww::MenuOption {
            label: "Save&Copy".into(),
            id: "savencopy".into(),
        },
    ];

    let action = eww::run_eww_menu("Action", &actions_menu);

    if let Some(act) = action {
        match act.as_str() {
            "save" | "savencopy" => {
                let mut dst = dirs::picture_dir()
                    .unwrap_or(PathBuf::from("."))
                    .join("Screenshots");
                let _ = fs::create_dir_all(&dst);
                dst.push(format!(
                    "screenshot_{}.png",
                    Local::now().format("%Y-%m-%d_%H-%M-%S")
                ));
                if fs::copy(&tmp_path, &dst).is_ok() {
                    if act == "savencopy" {
                        if let Ok(file) = fs::File::open(&dst) {
                            let _ = Command::new("wl-copy")
                                .arg("--type")
                                .arg("image/png")
                                .stdin(file)
                                .status();
                        }
                    }
                    let _ = Command::new("notify-send")
                        .arg("Saved")
                        .arg(dst.to_string_lossy().as_ref())
                        .status();
                }
            }
            "copy" => {
                if let Ok(file) = fs::File::open(&tmp_path) {
                    let _ = Command::new("wl-copy")
                        .arg("--type")
                        .arg("image/png")
                        .stdin(file)
                        .status();
                    let _ = Command::new("notify-send")
                        .arg("Copied to clipboard")
                        .status();
                }
            }
            "edit" => {
                let editor = if which("satty").is_ok() {
                    "satty"
                } else {
                    "xdg-open"
                };
                let mut cmd = Command::new(editor);
                if editor == "satty" {
                    cmd.arg("-f");
                }
                let _ = cmd.arg(&*tmp_str).status();
            }
            _ => {}
        }
    }

    let _ = fs::remove_file(&tmp_path);
}
