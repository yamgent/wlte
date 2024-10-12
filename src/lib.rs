mod app;
mod base;
mod editor;

use anyhow::Result;

use app::App;

pub fn run() -> Result<()> {
    App::run()
}
