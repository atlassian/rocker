use std::sync::Arc;

use shiplift::{rep::Info, Docker};
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

use app::AppCommand;
use views::View;

pub struct DockerInfo {
    info: Option<Info>,
}

impl DockerInfo {
    pub fn new() -> DockerInfo {
        DockerInfo { info: None }
    }
}

impl View for DockerInfo {
    fn handle_input(&mut self, _key: Key, _docker: Arc<Docker>) -> Option<AppCommand> {
        None
    }

    fn refresh(&mut self, docker: Arc<Docker>) {
        self.info = docker.info().ok();
    }

    fn draw(&self, t: &mut Frame<MouseBackend>, rect: Rect) {
        let display_string = if let Some(ref info) = self.info {
            format!("{:#?}", info)
        } else {
            "Could not retrieve information from the Docker daemon.".to_string()
        };
        let text = vec![Text::raw(display_string)];

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .raw(true)
            .render(t, rect);
    }
}
