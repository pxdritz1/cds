mod island;
mod shell;
mod theme;

use amane::App;

fn main() {
    App::new().window(island::view).run();
}
