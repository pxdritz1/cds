mod island;
mod shell;
mod theme;

use amane::App;

fn main() {
    App::new()
        .font("SF Pro")
        .window(shell::dismiss::view)
        .window(island::reveal)
        .window(island::view)
        .run();
}
