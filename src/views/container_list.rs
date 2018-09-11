use std::sync::Arc;
use std::time::Duration;

use shiplift::{
    rep::{Container, Port},
    ContainerListOptions, Docker,
};
use termion::event::Key;
use tui::{
    backend::{Backend, MouseBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget},
    Frame,
};

use app::{AppCommand, ContainerId};
use views::{human_duration, View, ViewType};

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

    fn draw_container_list<B: Backend>(&self, t: &mut Frame<B>, rect: Rect) {
        let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
        let normal_style = Style::default().fg(Color::White);
        let running_style = Style::default().fg(Color::Green);
        let header = ["Container ID", "Name", "Image", "Command", "Status"];
        let rows: Vec<_> = self
            .containers
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let data: Vec<String> = vec![
                    c.Id.clone(),
                    Self::container_name(c).unwrap_or_default().to_string(),
                    c.Image.clone(),
                    c.Command.clone(),
                    c.Status.clone(),
                ];
                if i == self.selected {
                    Row::StyledData(data.into_iter(), selected_style)
                } else if c.Status.starts_with("Up ") {
                    Row::StyledData(data.into_iter(), running_style)
                } else {
                    Row::StyledData(data.into_iter(), normal_style)
                }
            })
            .collect();

        Table::new(header.into_iter(), rows.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .widths(&[15, 20, 20, 30, 20]) // TODO be smarter with sizes here
            .render(t, rect);
    }

    fn draw_container_info<B: Backend>(&self, t: &mut Frame<B>, rect: Rect) {
        if let Some(c) = self.get_selected_container() {
            let created_time = ::std::time::UNIX_EPOCH + Duration::from_secs(c.Created);
            let duration = created_time.elapsed().unwrap();
            let mut ports = c.Ports.clone();
            let ports_slice: &mut [Port] = ports.as_mut();
            ports_slice.sort_by_key(|p: &Port| p.PrivatePort);
            let ports_displayed = ports_slice
                .iter()
                .map(|p: &Port| display_port(p))
                .collect::<Vec<_>>()
                .join("\n                ");

            let duration_text = format!("{:15} {} ago\n", "Created:", human_duration(&duration));
            let text = vec![Text::raw(duration_text)];
            Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::ALL))
                .wrap(true)
                .render(t, rect);
        } else {
            Paragraph::new(vec![].iter())
                .block(Block::default().borders(Borders::ALL))
                .wrap(true)
                .render(t, rect);
        }

        // .text(
        //     current_container
        //         .map(|c| {
        //             format!(
        //                 "{{mod=bold {:15}}} {} ago\n\
        //                  {{mod=bold {:15}}} {}\n\
        //                  {{mod=bold {:15}}} {}\n\
        //                  {{mod=bold {:15}}} {}\n\
        //                  {{mod=bold {:15}}} {:?}\n\
        //                  {{mod=bold {:15}}} {}\n\
        //                  {{mod=bold {:15}}} {}\n\
        //                  {{mod=bold {:15}}} {}\n\
        //                  {{mod=bold {:15}}} {:?}\n\
        //                  {{mod=bold {:15}}} {:?}",
        //                 "Created:",
        //                 human_duration(&duration),
        //                 "Command:",
        //                 c.Command,
        //                 "Id:",
        //                 c.Id,
        //                 "Image:",
        //                 c.Image,
        //                 "Labels:",
        //                 c.Labels,
        //                 "Name:",
        //                 Self::container_name(c).unwrap_or_else(|| ""),
        //                 "Ports:",
        //                 ports_displayed,
        //                 "Status:",
        //                 c.Status,
        //                 "SizeRW:",
        //                 c.SizeRw,
        //                 "SizeRootFs:",
        //                 c.SizeRootFs,
        //             )
        //         })
        //         .unwrap_or_else(|| "".to_string())
        //         .as_str(),
        // )
    }

    fn container_name(container: &Container) -> Option<&str> {
        container.Names.first().map(|name| {
            if name.starts_with('/') {
                &name[1..]
            } else {
                name.as_str()
            }
        })
    }
}

impl View for ContainerListView {
    fn handle_input(&mut self, key: Key, docker: Arc<Docker>) -> Option<AppCommand> {
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
            Key::Char('p') => {
                let selected_container = self.get_selected_container().unwrap();
                let containers = docker.containers();
                let container = containers.get(&selected_container.Id);
                info!("Pausing container {}", selected_container.Id);
                match container.pause() {
                    Ok(_) => Some(AppCommand::NoOp),
                    Err(err) => {
                        error!("Failed to pause container: {}", err);
                        Some(AppCommand::ErrorMsg(format!(
                            "Failed to pause container: {}",
                            err
                        )))
                    }
                }
            }
            Key::Char('P') => {
                let selected_container = self.get_selected_container().unwrap();
                let containers = docker.containers();
                let container = containers.get(&selected_container.Id);
                info!("Un-pausing container {}", selected_container.Id);
                match container.unpause() {
                    Ok(_) => Some(AppCommand::NoOp),
                    Err(err) => {
                        error!("Failed to un-pause container: {}", err);
                        Some(AppCommand::ErrorMsg(format!(
                            "Failed to unpause container: {}",
                            err
                        )))
                    }
                }
            }
            Key::Char('s') => {
                let selected_container = self.get_selected_container().unwrap();
                let containers = docker.containers();
                let container = containers.get(&selected_container.Id);
                // TODO use a timeout?
                info!("Stopping container {}", selected_container.Id);
                match container.stop(None) {
                    Ok(_) => Some(AppCommand::NoOp),
                    Err(err) => {
                        error!("Failed to stop container: {}", err);
                        Some(AppCommand::ErrorMsg(format!(
                            "Failed to stop container: {}",
                            err
                        )))
                    }
                }
            }
            Key::Char('S') => {
                let selected_container = self.get_selected_container().unwrap();
                let containers = docker.containers();
                let container = containers.get(&selected_container.Id);
                info!("Starting container {}", selected_container.Id);
                match container.start() {
                    Ok(_) => Some(AppCommand::NoOp),
                    Err(err) => {
                        error!("Failed to start container: {}", err);
                        Some(AppCommand::ErrorMsg(format!(
                            "Failed to start container: {}",
                            err
                        )))
                    }
                }
            }
            Key::Char('d') => {
                // delete
                let selected_container = self.get_selected_container().unwrap();
                let containers = docker.containers();
                let container = containers.get(&selected_container.Id);
                info!("Deleting container {}", selected_container.Id);
                match container.delete() {
                    Ok(_) => Some(AppCommand::NoOp),
                    Err(err) => {
                        error!("Failed to delete container: {}", err);
                        Some(AppCommand::ErrorMsg(format!("{}", err)))
                    }
                }
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

    fn draw(&self, t: &mut Frame<MouseBackend>, rect: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
            .margin(0)
            .split(rect);

        // Containers
        self.draw_container_list(t, chunks[0]);

        // Container details
        self.draw_container_info(t, chunks[1]);
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
