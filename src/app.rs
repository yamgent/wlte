use anyhow::Result;

use crate::base_app::{BaseApp, BaseAppEvent, BaseAppLogic};

struct AppLogic;

impl BaseAppLogic for AppLogic {
    fn handle_events(&mut self, event: BaseAppEvent) {
        // TODO: Handle events
        println!("{:?}", event);
    }

    fn render(&mut self) {
        // TODO: Handle rendering
        println!("Drawing");
    }
}

pub struct App<'b> {
    base_app: BaseApp<'b, AppLogic>,
}

impl<'b> App<'b> {
    pub fn new() -> Self {
        Self {
            base_app: BaseApp::new(AppLogic),
        }
    }

    pub fn run(self) -> Result<()> {
        self.base_app.run()
    }
}
