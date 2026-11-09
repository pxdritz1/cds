mod app;
mod cli;
mod ipc;
mod paths;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("cds: {error:#}");
        std::process::exit(1);
    }
}
