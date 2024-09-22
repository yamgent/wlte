use anyhow::Result;

use crate::base_app::{BaseApp, BaseAppEvent, BaseAppLogic, BaseAppRenderer};

struct AppLogic;

impl BaseAppLogic for AppLogic {
    fn handle_events(&mut self, event: BaseAppEvent) {
        // TODO: Handle events
        println!("{:?}", event);
    }

    fn render(&mut self, renderer: &mut BaseAppRenderer) {
        // TODO: Handle rendering
        renderer.draw_dummy();
    }
}

pub struct App {
    base_app: BaseApp<AppLogic>,
}

impl App {
    pub fn new() -> Self {
        Self {
            base_app: BaseApp::new(AppLogic),
        }
    }

    pub fn run(self) -> Result<()> {
        self.base_app.run()
    }
}
