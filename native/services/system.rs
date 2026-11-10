use anyhow::{Context, Result, bail};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SystemSnapshot {
    pub wifi_enabled: bool,
    pub bluetooth_blocked: bool,
    pub brightness: u8,
}

fn run(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output()
        .with_context(|| format!("could not execute {program}"))?;
    if !output.status.success() {
        bail!("{program} {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn snapshot() -> Result<SystemSnapshot> {
    let wifi_enabled = run("nmcli", &["radio", "wifi"]).map(|value| value.trim() == "enabled").unwrap_or(false);
    let bluetooth = run("rfkill", &["list", "bluetooth"]).unwrap_or_default();
    let bluetooth_blocked = bluetooth.lines().any(|line| line.to_ascii_lowercase().contains("soft blocked: yes"));
    let brightness = match (run("brightnessctl", &["get"]), run("brightnessctl", &["max"])) {
        (Ok(current), Ok(maximum)) => {
            let current = current.trim().parse::<u32>().unwrap_or(0);
            let maximum = maximum.trim().parse::<u32>().unwrap_or(1).max(1);
            ((current.saturating_mul(100) / maximum).min(100)) as u8
        }
        _ => 0,
    };
    Ok(SystemSnapshot { wifi_enabled, bluetooth_blocked, brightness })
}

pub fn set_wifi(enabled: bool) -> Result<()> {
    run("nmcli", &["radio", "wifi", if enabled { "on" } else { "off" }])?;
    Ok(())
}

pub fn set_bluetooth(enabled: bool) -> Result<()> {
    run("rfkill", &[if enabled { "unblock" } else { "block" }, "bluetooth"])?;
    Ok(())
}

pub fn adjust_brightness(delta: i8) -> Result<()> {
    let amount = format!("{}{}%", if delta >= 0 { "+" } else { "" }, delta);
    run("brightnessctl", &["set", &amount])?;
    Ok(())
}
