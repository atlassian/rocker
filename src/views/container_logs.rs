use std::sync::Arc;

use termion::event::Key;
use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, Text, Widget},
    Frame,
};

use app::{AppCommand, ContainerId};
use docker::DockerExecutor;
use tty::{InterlacedTty, TtyLine};
use views::View;
use Backend;

pub struct ContainerLogsView {
    id: ContainerId,
    scroll: u16,
    logs: Option<InterlacedTty>,
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
        let logs_reader = docker.container_logs(
            &self.id.0,
            &LogsOptions::builder()
                .follow(false)
                .tail("100")
                .stdout(true)
                .stderr(true)
                .build(),
        );
        let tty = InterlacedTty::new(logs_reader);
        self.logs = Some(tty);
    }

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
        let stdout_style = Style::default().bg(Color::Black).fg(Color::White);
        let stderr_style = Style::default().bg(Color::Black).fg(Color::Red);

        let style = |l: &TtyLine| match l {
            TtyLine::StdOut(_) => stdout_style,
            TtyLine::StdErr(_) => stderr_style,
        };
        let formatted_lines = self
            .logs
            .as_ref()
            .map(|t| {
                t.lines
                    .iter()
                    .map(|l| Text::styled(format!("{}", l), style(l)))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);

        List::new(formatted_lines.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .render(t, rect);
    }
}
