mod island;
mod shell;
mod theme;

use amane::App;

fn main() {
    App::new()
        .window(shell::dismiss::view)
        .window(island::view)
        .run();
}
