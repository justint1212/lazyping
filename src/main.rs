use crate::{model::App, ping::PingSession};

mod model;
pub mod ping;
mod update;
mod view;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
