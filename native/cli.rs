use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use crate::app;
use crate::ipc::{self, IpcCommand, IpcResponse};

#[derive(Debug, Parser)]
#[command(name = "cds", version, about = "Chloe Dynamic Shell")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the native shell daemon.
    Daemon,
    /// Start CDS if it is not already running.
    Run,
    /// Build and run the development version from a checkout.
    Dev,
    /// Build the optimized shell binary.
    Compile,
    /// Install the cds executable from this checkout.
    Install,
    /// Remove the installed cds executable.
    Uninstall,
    /// Open a shell panel.
    Open { panel: String },
    /// Close the active panel.
    Close,
    /// Open the dashboard, optionally on a specific tab.
    Dash { tab: Option<String> },
    /// Lock the current session.
    Lock,
    /// Control the StatusNotifier tray.
    Tray {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Read or change shell preferences.
    Settings {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run Matugen theme actions.
    Theme {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Refresh and print weather information.
    Weather,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Run) => {
            if ipc::is_running() {
                println!("CDS is already running.");
                return Ok(());
            }
            app::run()
        }
        Some(Commands::Daemon) => app::run(),
        Some(Commands::Dev) => cargo_in_checkout(&["run", "--bin", "cds", "--", "daemon"]),
        Some(Commands::Compile) => cargo_in_checkout(&["build", "--release", "--bin", "cds"]),
        Some(Commands::Install) => cargo_in_checkout(&[
            "install",
            "--path",
            ".",
            "--bin",
            "cds",
            "--force",
        ]),
        Some(Commands::Uninstall) => cargo_command(&["uninstall", "chloe-dynamic-shell"]),
        Some(Commands::Open { panel }) => print_response(ipc::send(IpcCommand::Open { panel })?),
        Some(Commands::Close) => print_response(ipc::send(IpcCommand::Close)?),
        Some(Commands::Dash { tab }) => print_response(ipc::send(IpcCommand::Dash { tab })?),
        Some(Commands::Lock) => print_response(ipc::send(IpcCommand::Lock)?),
        Some(Commands::Tray { args }) => print_response(ipc::send(IpcCommand::Tray { args })?),
        Some(Commands::Settings { args }) => {
            print_response(ipc::send(IpcCommand::Settings { args })?)
        }
        Some(Commands::Theme { args }) => print_response(ipc::send(IpcCommand::Theme { args })?),
        Some(Commands::Weather) => print_response(ipc::send(IpcCommand::Weather)?),
    }
}

fn print_response(response: IpcResponse) -> Result<()> {
    println!("{}", response.message);
    if !response.ok {
        bail!("{}", response.message);
    }
    Ok(())
}

fn cargo_in_checkout(arguments: &[&str]) -> Result<()> {
    let root = find_checkout_root()?;
    let status = ProcessCommand::new("cargo")
        .args(arguments)
        .current_dir(&root)
        .status()
        .context("could not start Cargo")?;

    if !status.success() {
        bail!("Cargo exited with status {status}");
    }
    Ok(())
}

fn cargo_command(arguments: &[&str]) -> Result<()> {
    let status = ProcessCommand::new("cargo")
        .args(arguments)
        .status()
        .context("could not start Cargo")?;

    if !status.success() {
        bail!("Cargo exited with status {status}");
    }
    Ok(())
}

fn find_checkout_root() -> Result<PathBuf> {
    let current = std::env::current_dir().context("could not read the current directory")?;

    for directory in current.ancestors() {
        let manifest = directory.join("Cargo.toml");
        if let Ok(contents) = std::fs::read_to_string(&manifest) {
            if contents.contains("name = \"chloe-dynamic-shell\"") {
                return Ok(directory.to_path_buf());
            }
        }
    }

    bail!("this command must be run inside a CDS source checkout")
}
