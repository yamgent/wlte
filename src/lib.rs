mod app;
mod base;

use app::App;

pub fn run() {
    App::new().run().expect("fatal app error");
}
