use crate::hypr::get_active_monitor_id;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{fs, thread};

const ICON_AREA: &[u8] = include_bytes!("../assets/icons/Area.svg");
const ICON_MONITOR: &[u8] = include_bytes!("../assets/icons/Monitor.svg");
const ICON_COPY: &[u8] = include_bytes!("../assets/icons/Copy.svg");
const ICON_EDIT: &[u8] = include_bytes!("../assets/icons/Edit.svg");
const ICON_SAVE: &[u8] = include_bytes!("../assets/icons/Save.svg");
const ICON_SAVENCOPY: &[u8] = include_bytes!("../assets/icons/SavenCopy.svg");

const EWW_YUCK: &str = include_str!("../assets/eww/eww.yuck");
const EWW_CSS: &str = include_str!("../assets/eww/eww.css");

pub struct MenuOption {
    pub label: String,
    pub id: String,
}

pub fn run_eww_menu(title: &str, options: &[MenuOption]) -> Option<String> {
    let runtime_path = prepare_runtime_config();
    let icons_dir = runtime_path.join("icons");

    let mut buttons_yuck = String::from("(box :orientation \"h\" :spacing 15 ");
    for option in options {
        let id = &option.id;
        let label = &option.label;
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

fn prepare_runtime_config() -> PathBuf {
    let runtime_dir = std::env::temp_dir().join("dumbshot_runtime");
    let icons_dir = runtime_dir.join("icons");

    let _ = fs::create_dir_all(&icons_dir);

    let write_icon = |name: &str, data: &[u8]| {
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

    let _ = fs::write(runtime_dir.join("eww.yuck"), yuck);
    let _ = fs::write(runtime_dir.join("eww.css"), css);

    runtime_dir
}

fn get_config_content(file_name: &str, default_content: &str) -> String {
    let user_path = dirs::config_dir().map(|p| p.join("dumbshot").join("eww").join(file_name));

    if let Some(path) = user_path {
        if path.exists() {
            return fs::read_to_string(path).unwrap_or_else(|_| default_content.to_string());
        }
    }
    default_content.to_string()
}
