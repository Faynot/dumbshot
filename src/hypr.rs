use serde_json::Value;
use std::process::Command;

pub fn get_active_monitor_id() -> i64 {
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

pub fn get_monitors_list() -> Option<Vec<String>> {
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
