use std::sync::Arc;

use shiplift::rep::ContainerDetails;
use termion::event::Key;
use tui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

use crate::app::{AppCommand, ContainerId};
use crate::docker::DockerExecutor;
use crate::views::View;
use crate::Backend;

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
    fn handle_input(&mut self, key: Key, _docker: Arc<DockerExecutor>) -> Option<AppCommand> {
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

    fn refresh(&mut self, docker: Arc<DockerExecutor>) {
        // self.details = docker.containers().get(&self.name).inspect().ok();
        self.details = docker.container(&self.name).ok();
    }

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
        let data = if let Some(ref info) = self.details {
            Text::raw(format!("{:#?}", info))
        } else {
            Text::raw("Could not retrieve container details.")
        };
        let text = vec![data];

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, rect);
    }
}
