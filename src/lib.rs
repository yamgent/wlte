mod app;
mod base;
mod editor;

use app::App;

pub fn run() {
    App::run().expect("fatal app error");
}
