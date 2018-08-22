use std::sync::Arc;

use shiplift::{rep::Info, Docker};
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Paragraph, Widget},
    Terminal,
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

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: Rect) {
        let display_string = if let Some(ref info) = self.info {
            format!("{:#?}", info)
        } else {
            "Could not retrieve information from the Docker daemon.".to_string()
        };
        Paragraph::default()
            .text(&display_string)
            .wrap(true)
            .raw(true)
            .render(t, &rect);
    }
}
