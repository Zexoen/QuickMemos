mod api;
mod config;
mod ui;

use config::AppConfig;
use relm4::prelude::*;

fn main() {
    let config = AppConfig::load();
    let app = RelmApp::new("dev.quickmemos.QuickMemos");
    app.run_async::<ui::app::App>(config);
}