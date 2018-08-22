use std::sync::Arc;

use shiplift::{tty::Tty, Docker, LogsOptions};
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};

use app::{AppCommand, ContainerId};
use views::View;

pub struct ContainerLogsView {
    id: ContainerId,
    scroll: u16,
    logs: Option<Tty>,
}

impl ContainerLogsView {
    pub fn new(id: ContainerId) -> ContainerLogsView {
        ContainerLogsView {
            id,
            scroll: 0,
            logs: None,
        }
    }
}

impl View for ContainerLogsView {
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
        let containers = docker.containers();
        let container = containers.get(&self.id.0);
        let logs_reader = container
            .logs(
                &LogsOptions::builder()
                    .follow(false)
                    .tail("100")
                    .stdout(true)
                    .stderr(true)
                    .build(),
            )
            .unwrap();
        let tty = Tty::new(logs_reader);
        self.logs = Some(tty);
    }

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: Rect) {
        let logs = self
            .logs
            .as_ref()
            .map(|t| t.stdout.as_ref())
            .unwrap_or_else(|| "");
        Paragraph::default()
            .block(Block::default().borders(Borders::ALL))
            .text(&logs)
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, &rect);
    }
}
