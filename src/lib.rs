mod app;
mod base;

use app::App;

pub fn run() {
    App::run().expect("fatal app error");
}
