use std::sync::Arc;

use shiplift::rep::Info;
use termion::event::Key;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

use crate::app::AppCommand;
use crate::docker::DockerExecutor;
use crate::views::View;
use crate::Backend;

pub struct DockerInfo {
    info: Option<Info>,
}

impl DockerInfo {
    pub fn new() -> DockerInfo {
        DockerInfo { info: None }
    }
}

impl View for DockerInfo {
    fn handle_input(&mut self, _key: Key, _docker: Arc<DockerExecutor>) -> Option<AppCommand> {
        None
    }

    fn refresh(&mut self, docker: Arc<DockerExecutor>) {
        self.info = docker.info().ok();
    }

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
        let text = if let Some(ref info) = self.info {
            vec![
                Text::styled(
                    "Host details\n",
                    Style::default().modifier(Modifier::BOLD).fg(Color::Blue),
                ),
                Text::styled(format!("Hostname:       {}\n", info.name), Style::default()),
                Text::styled(
                    format!("OS Information: {}\n", info.operating_system),
                    Style::default(),
                ),
                Text::styled(
                    format!("Kernel Version: {}\n", info.kernel_version),
                    Style::default(),
                ),
                Text::styled(
                    format!("Total CPU:      {}\n", info.n_cpu),
                    Style::default(),
                ),
                Text::styled(
                    format!("Total Memory:   {}\n", info.mem_total),
                    Style::default(),
                ),
                Text::raw("\n"),
                Text::styled(
                    "Engine details\n",
                    Style::default().modifier(Modifier::BOLD).fg(Color::Blue),
                ),
                Text::styled(
                    format!("Root Directory:       {}\n", info.docker_root_dir),
                    Style::default(),
                ),
                Text::styled(
                    format!("Storage Driver:       {}\n", info.driver),
                    Style::default(),
                ),
            ]
        } else {
            vec![Text::styled(
                "Could not retrieve information from the Docker daemon.".to_string(),
                Style::default(),
            )]
        };

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .raw(true)
            .render(t, rect);
    }
}
