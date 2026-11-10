use anyhow::{Context, Result, bail};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct AudioSnapshot {
    pub output_volume: u8,
    pub output_muted: bool,
    pub input_volume: u8,
    pub input_muted: bool,
}

fn pactl(args: &[&str]) -> Result<String> {
    let output = Command::new("pactl").args(args).output().context("could not execute pactl (install pipewire-pulse or pulseaudio)")?;
    if !output.status.success() {
        bail!("pactl {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_volume(value: &str) -> u8 {
    value.split_whitespace()
        .filter_map(|part| part.strip_suffix('%')?.parse::<u16>().ok())
        .max()
        .unwrap_or(0)
        .min(150) as u8
}

fn parse_mute(value: &str) -> bool {
    value.lines().any(|line| line.trim().eq_ignore_ascii_case("Mute: yes"))
}

pub fn snapshot() -> Result<AudioSnapshot> {
    let output_volume = pactl(&["get-sink-volume", "@DEFAULT_SINK@"])?;
    let output_mute = pactl(&["get-sink-mute", "@DEFAULT_SINK@"])?;
    let input_volume = pactl(&["get-source-volume", "@DEFAULT_SOURCE@"]).unwrap_or_default();
    let input_mute = pactl(&["get-source-mute", "@DEFAULT_SOURCE@"]).unwrap_or_default();
    Ok(AudioSnapshot {
        output_volume: parse_volume(&output_volume),
        output_muted: parse_mute(&output_mute),
        input_volume: parse_volume(&input_volume),
        input_muted: parse_mute(&input_mute),
    })
}

pub fn adjust_output(delta: i8) -> Result<()> {
    let change = format!("{}{}%", if delta >= 0 { "+" } else { "" }, delta);
    pactl(&["set-sink-volume", "@DEFAULT_SINK@", &change])?;
    Ok(())
}

pub fn toggle_output_mute() -> Result<()> {
    pactl(&["set-sink-mute", "@DEFAULT_SINK@", "toggle"])?;
    Ok(())
}

pub fn toggle_input_mute() -> Result<()> {
    pactl(&["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])?;
    Ok(())
}

pub fn set_output_volume(volume: u8) -> Result<()> {
    let volume = format!("{}%", volume.min(150));
    pactl(&["set-sink-volume", "@DEFAULT_SINK@", &volume])?;
    Ok(())
}
