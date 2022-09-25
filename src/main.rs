extern crate core;

mod content;
mod editor;
mod line;
mod terminal;

pub use content::Content;
use editor::Editor;
pub use editor::Position;
pub use line::Line;
pub use log::{info, LevelFilter};
pub use terminal::Terminal;

fn main() {
    simple_logging::log_to_file("test.log", LevelFilter::Info).unwrap();
    info!("Welcome to the edek!");
    Editor::create().run().unwrap();
}
