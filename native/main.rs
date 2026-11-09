mod app;
mod cli;
mod ipc;
mod paths;
mod services;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("cds: {error:#}");
        std::process::exit(1);
    }
}
