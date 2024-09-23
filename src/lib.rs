mod app;
mod base_app;

use app::App;

pub fn run() {
    App::new().run().expect("fatal app error");
}
