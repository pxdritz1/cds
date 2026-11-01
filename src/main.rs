mod island;
mod theme;

use amane::App;

fn main() {
    App::new().window(island::view).run();
}
