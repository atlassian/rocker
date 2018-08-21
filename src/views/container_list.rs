use std::sync::Arc;
use std::time::Duration;

use humantime;

use shiplift::{
    rep::{Container, Port},
    ContainerListOptions, Docker,
};
use termion::event::Key;
use tui::{
    backend::{Backend, MouseBackend},
    layout::{Direction, Group, Rect, Size},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Widget},
    Terminal,
};

use app::{AppCommand, ContainerId};
use views::{View, ViewType};

pub struct ContainerListView {
    /// List of containers to display
    pub containers: Vec<Container>,
    /// Index of the currently selected container from the above list
    pub selected: usize,
    /// Whether to only display currently running containers
    pub only_running: bool,
}

impl ContainerListView {
    pub fn new() -> ContainerListView {
        ContainerListView {
            containers: Vec::new(),
            selected: 0,
            only_running: false,
        }
    }

    pub fn get_selected_container(&self) -> Option<&Container> {
        self.containers.get(self.selected)
    }

    fn draw_container_list<B: Backend>(&self, t: &mut Terminal<B>, rect: Rect) {
        let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
        let normal_style = Style::default().fg(Color::White);
        let running_style = Style::default().fg(Color::Green);
        let header = ["Container ID", "Image", "Command", "Status"];
        let rows: Vec<_> = self
            .containers
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let data: Vec<&str> = vec![
                    c.Id.as_ref(),
                    c.Image.as_ref(),
                    c.Command.as_ref(),
                    c.Status.as_ref(),
                ];
                if i == self.selected {
                    Row::StyledData(data.into_iter(), &selected_style)
                } else if c.Status.starts_with("Up ") {
                    Row::StyledData(data.into_iter(), &running_style)
                } else {
                    Row::StyledData(data.into_iter(), &normal_style)
                }
            })
            .collect();

        Table::new(header.into_iter(), rows.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .widths(&[15, 20, 30, 20])
            .render(t, &rect);
    }

    fn draw_container_info<B: Backend>(&self, t: &mut Terminal<B>, rect: Rect) {
        let current_container = self.get_selected_container();
        Paragraph::default()
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .text(
                current_container
                    .map(|c| {
                        let created_time = ::std::time::UNIX_EPOCH + Duration::from_secs(c.Created);
                        let duration = created_time.elapsed().unwrap();
                        // Truncate to second precision
                        let duration = Duration::new(duration.as_secs(), 0);
                        let mut ports = c.Ports.clone();
                        let ports_slice: &mut [Port] = ports.as_mut();
                        ports_slice.sort_by_key(|p: &Port| p.PrivatePort);
                        let ports_displayed = ports_slice
                            .iter()
                            .map(|p: &Port| display_port(p))
                            .collect::<Vec<_>>()
                            .join("\n                ");

                        format!(
                            "{{mod=bold {:15}}} {} ago\n\
                             {{mod=bold {:15}}} {}\n\
                             {{mod=bold {:15}}} {}\n\
                             {{mod=bold {:15}}} {}\n\
                             {{mod=bold {:15}}} {:?}\n\
                             {{mod=bold {:15}}} {}\n\
                             {{mod=bold {:15}}} {}\n\
                             {{mod=bold {:15}}} {}\n\
                             {{mod=bold {:15}}} {:?}\n\
                             {{mod=bold {:15}}} {:?}",
                            "Created:",
                            humantime::format_duration(duration),
                            "Command:",
                            c.Command,
                            "Id:",
                            c.Id,
                            "Image:",
                            c.Image,
                            "Labels:",
                            c.Labels,
                            "Names:",
                            c.Names.join(", "),
                            "Ports:",
                            ports_displayed,
                            "Status:",
                            c.Status,
                            "SizeRW:",
                            c.SizeRw,
                            "SizeRootFs:",
                            c.SizeRootFs,
                        )
                    })
                    .unwrap_or_else(|| "".to_string())
                    .as_str(),
            )
            .render(t, &rect);
    }
}

impl View for ContainerListView {
    fn handle_input(&mut self, key: Key) -> Option<AppCommand> {
        match key {
            Key::Down | Key::Char('j') => {
                if !self.containers.is_empty() {
                    self.selected += 1;
                    if self.selected > self.containers.len() - 1 {
                        self.selected = 0;
                    }
                }
                Some(AppCommand::NoOp)
            }
            Key::Up | Key::Char('k') => {
                if !self.containers.is_empty() {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = self.containers.len() - 1;
                    }
                }
                Some(AppCommand::NoOp)
            }
            Key::Char('a') => {
                self.only_running = !self.only_running;
                // self.refresh();
                Some(AppCommand::NoOp)
            }
            Key::Char('\n') => {
                let container = self.get_selected_container().unwrap();
                let id = ContainerId(container.Id.clone());
                Some(AppCommand::SwitchToView(ViewType::ContainerDetails(id)))
            }
            Key::Char('l') => {
                let container = self.get_selected_container().unwrap();
                let id = ContainerId(container.Id.clone());
                Some(AppCommand::SwitchToView(ViewType::ContainerLogs(id)))
            }
            _ => None,
        }
    }

    fn refresh(&mut self, docker: Arc<Docker>) {
        let options = if self.only_running {
            ContainerListOptions::builder().build()
        } else {
            ContainerListOptions::builder().all().build()
        };
        let containers = docker.containers().list(&options).unwrap();
        self.containers = containers;
        if self.containers.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.containers.len() {
            self.selected = self.containers.len() - 1;
        }
    }

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: Rect) {
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Percent(70), Size::Percent(30)])
            .margin(0)
            .render(t, &rect, |t, chunks| {
                // Containers
                self.draw_container_list(t, chunks[0]);

                // Container details
                self.draw_container_info(t, chunks[1]);
            });
    }
}

fn display_port(port: &Port) -> String {
    let mut s = String::new();
    if let Some(ref ip) = port.IP {
        s.push_str(&format!("{}:", ip));
    }
    s.push_str(&format!("{}", port.PrivatePort));
    if let Some(ref public_port) = port.PublicPort {
        s.push_str(&format!(" → {}", public_port));
    }
    s.push_str(&format!("/{}", port.Type));

    s
}
