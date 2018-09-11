use std::sync::Arc;

use shiplift::{rep::ContainerDetails, Docker};
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

use app::{AppCommand, ContainerId};
use views::View;

pub struct ContainerInfo {
    name: String,
    details: Option<ContainerDetails>,
    scroll: u16,
}

impl ContainerInfo {
    pub fn new(id: ContainerId) -> ContainerInfo {
        let ContainerId(id) = id;
        ContainerInfo {
            name: id,
            details: None,
            scroll: 0,
        }
    }
}

impl View for ContainerInfo {
    fn handle_input(&mut self, key: Key, _docker: Arc<Docker>) -> Option<AppCommand> {
        match key {
            Key::Up | Key::Char('k') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                Some(AppCommand::NoOp)
            }
            Key::Down | Key::Char('j') => {
                self.scroll += 1;
                Some(AppCommand::NoOp)
            }
            _ => None,
        }
    }

    fn refresh(&mut self, docker: Arc<Docker>) {
        self.details = docker.containers().get(&self.name).inspect().ok();
    }

    fn draw(&self, t: &mut Frame<MouseBackend>, rect: Rect) {
        let display_string = if let Some(ref info) = self.details {
            format!("{:#?}", info)
        } else {
            "Could not retrieve container details.".to_string()
        };
        let text = vec![Text::raw(display_string)];

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, rect);
    }
}
