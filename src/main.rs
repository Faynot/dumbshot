use chrono::Local;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use which::which;

const ICON_AREA: &[u8] = include_bytes!("../assets/icons/Area.svg");
const ICON_MONITOR: &[u8] = include_bytes!("../assets/icons/Monitor.svg");
const ICON_COPY: &[u8] = include_bytes!("../assets/icons/Copy.svg");
const ICON_EDIT: &[u8] = include_bytes!("../assets/icons/Edit.svg");
const ICON_SAVE: &[u8] = include_bytes!("../assets/icons/Save.svg");
const ICON_SAVENCOPY: &[u8] = include_bytes!("../assets/icons/SavenCopy.svg");

const EWW_YUCK: &str = include_str!("../assets/eww/eww.yuck");
const EWW_CSS: &str = include_str!("../assets/eww/eww.css");

fn get_config_content(file_name: &str, default_content: &str) -> String {
    let user_path = dirs::config_dir().map(|p| p.join("dumbshot").join("eww").join(file_name));

    if let Some(path) = user_path {
        if path.exists() {
            return fs::read_to_string(path).unwrap_or_else(|_| default_content.to_string());
        }
    }
    default_content.to_string()
}

fn prepare_runtime_config() -> PathBuf {
    let runtime_dir = std::env::temp_dir().join("dumbshot_runtime");
    let icons_dir = runtime_dir.join("icons");

    let _ = fs::create_dir_all(&icons_dir);

    let mut write_icon = |name: &str, data: &[u8]| {
        let _ = fs::write(icons_dir.join(name), data);
    };

    write_icon("area.svg", ICON_AREA);
    write_icon("monitor.svg", ICON_MONITOR);
    write_icon("all.svg", ICON_MONITOR);
    write_icon("copy.svg", ICON_COPY);
    write_icon("edit.svg", ICON_EDIT);
    write_icon("save.svg", ICON_SAVE);
    write_icon("savencopy.svg", ICON_SAVENCOPY);

    let yuck = get_config_content("eww.yuck", EWW_YUCK);
    let css = get_config_content("eww.css", EWW_CSS);

    //let monitor_id = get_active_monitor_id();
    //let final_yuck = yuck.replace("MONITOR_ID_HERE", &monitor_id.to_string());

    let _ = fs::write(runtime_dir.join("eww.yuck"), yuck);
    let _ = fs::write(runtime_dir.join("eww.css"), css);

    runtime_dir
}

fn get_active_monitor_id() -> i64 {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok();

    if let Some(out) = output {
        if let Ok(v) = serde_json::from_slice::<Value>(&out.stdout) {
            if let Some(monitors) = v.as_array() {
                for m in monitors {
                    if m["focused"].as_bool().unwrap_or(false) {
                        return m["id"].as_i64().unwrap_or(0);
                    }
                }
            }
        }
    }
    0
}

fn run_eww_menu(title: &str, options: &[(String, Vec<u8>, String)]) -> Option<String> {
    let runtime_path = prepare_runtime_config();
    let icons_dir = runtime_path.join("icons");

    let mut buttons_yuck = String::from("(box :orientation \"h\" :spacing 15 ");
    for (label, _, id) in options {
        let icon_file = icons_dir.join(format!("{}.svg", id));

        buttons_yuck.push_str(&format!(
            r#"(button :class "menu-btn" :onclick "echo '{}' > /tmp/dumbshot_res"
                (box :orientation "v" :spacing 5 :space-evenly false
                    (image :path "{}" :image-width 48 :image-height 48)
                    (label :text "{}"))) "#,
            id,
            icon_file.to_string_lossy(),
            label
        ));
    }
    buttons_yuck.push(')');

    let res_file = Path::new("/tmp/dumbshot_res");
    let _ = fs::remove_file(res_file);

    let config_arg = runtime_path.to_string_lossy();

    let mut daemon = Command::new("eww")
        .args(["--config", &config_arg, "daemon", "--no-daemonize"])
        .spawn()
        .ok()?;

    thread::sleep(Duration::from_millis(300));

    let _ = Command::new("eww")
        .args([
            "--config",
            &config_arg,
            "update",
            &format!("title={}", title),
        ])
        .status();
    let _ = Command::new("eww")
        .args([
            "--config",
            &config_arg,
            "update",
            &format!("buttons_json={}", buttons_yuck),
        ])
        .status();

    let monitor_id = get_active_monitor_id();
    let _ = Command::new("eww")
        .args([
            "--config",
            &config_arg,
            "open",
            "menu",
            "--arg",
            &format!("mon={}", monitor_id),
        ])
        .status();

    let mut result = None;
    for _ in 0..600 {
        if res_file.exists() {
            if let Ok(content) = fs::read_to_string(res_file) {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    result = Some(trimmed);
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    let _ = Command::new("eww")
        .args(["--config", &config_arg, "kill"])
        .status();
    let _ = daemon.kill();
    let _ = fs::remove_file(res_file);

    if result == Some("Cancel".into()) {
        None
    } else {
        result
    }
}

fn get_monitors_list() -> Option<Vec<String>> {
    let out = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;
    let v: Value = serde_json::from_slice(&out.stdout).ok()?;
    let names: Vec<String> = v
        .as_array()?
        .iter()
        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
        .collect();
    Some(names)
}

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

    let main_menu: Vec<(String, Vec<u8>, String)> = vec![
        ("Area".into(), ICON_AREA.to_vec(), "area".into()),
        ("Monitor".into(), ICON_MONITOR.to_vec(), "monitor".into()),
        ("All".into(), ICON_MONITOR.to_vec(), "all".into()),
    ];

    let choice = run_eww_menu("Screenshot Tool", &main_menu);
    let choice = match choice {
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
        "monitor" => {
            if let Some(monitors) = get_monitors_list() {
                let id = get_active_monitor_id() as usize;
                let m = monitors.get(id).unwrap_or(&monitors[0]);
                capture_screenshot(&["-o", m], &tmp_str)
            } else {
                capture_screenshot(&[], &tmp_str)
            }
        }
        "all" => capture_screenshot(&[], &tmp_str),
        _ => false,
    };

    if !success {
        return;
    }

    let actions_menu: Vec<(String, Vec<u8>, String)> = vec![
        ("Save".into(), ICON_SAVE.to_vec(), "save".into()),
        ("Copy".into(), ICON_COPY.to_vec(), "copy".into()),
        ("Edit".into(), ICON_EDIT.to_vec(), "edit".into()),
        (
            "Save&Copy".into(),
            ICON_SAVENCOPY.to_vec(),
            "savencopy".into(),
        ),
    ];

    let action = run_eww_menu("Action", &actions_menu);

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
