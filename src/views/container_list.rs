use std::sync::Arc;
use std::time::Duration;

use shiplift::{
    rep::{Container, Port},
    ContainerListOptions, Docker,
};
use termion::event::Key;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Row, Table, Text, Widget},
    Frame,
};

use app::{AppCommand, ContainerId};
use views::{human_duration, View, ViewType};
use Backend;

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

    fn draw_container_list(&self, t: &mut Frame<Backend>, rect: Rect) {
        let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
        let normal_style = Style::default().fg(Color::White);
        let running_style = Style::default().fg(Color::Green);
        let header = ["Container ID", "Name", "Image", "Command", "Status"];
        let height = rect.height as usize - 4; // 2 for border + 2 for header
        let offset = if self.selected >= height {
            self.selected - height + 1
        } else {
            0
        };
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
            }).skip(offset)
            .collect();

        Table::new(header.into_iter(), rows.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .widths(&[15, 20, 20, 30, 20]) // TODO be smarter with sizes here
            .render(t, rect);
    }

    fn draw_container_info(&self, t: &mut Frame<Backend>, rect: Rect) {
        let mut text = vec![];
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

            text.push(Text::raw(format!(
                "{:>15}: {} ago",
                "Created",
                human_duration(&duration)
            )));
            text.push(Text::raw(format!("{:>15}: {}", "Command", c.Command)));
            text.push(Text::raw(format!("{:>15}: {}", "Image", c.Image)));
            text.push(Text::raw(format!("{:>15}: {:?}", "Labels", c.Labels)));
            text.push(Text::raw(format!(
                "{:>15}: {}",
                "Name",
                Self::container_name(c).unwrap_or_else(|| "")
            )));
            text.push(Text::raw(format!("{:>15}: {}", "Ports", ports_displayed)));
            text.push(Text::raw(format!("{:>15}: {}", "Status", c.Status)));
            text.push(Text::raw(format!("{:>15}: {:?}", "SizeRW", c.SizeRw)));
            text.push(Text::raw(format!(
                "{:>15}: {:?}",
                "SizeRootFs", c.SizeRootFs
            )));
        }

        List::new(text.into_iter())
            .block(Block::default().borders(Borders::ALL))
            // .wrap(true)
            .render(t, rect);
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
        let max_index = self.containers.len() - 1;
        match key {
            Key::Down | Key::Char('j') => {
                if !self.containers.is_empty() {
                    self.selected = (self.selected + 1).min(max_index);
                }
                Some(AppCommand::NoOp)
            }
            Key::Up | Key::Char('k') => {
                if !self.containers.is_empty() && self.selected > 0 {
                    self.selected -= 1;
                }
                Some(AppCommand::NoOp)
            }
            Key::PageDown | Key::Ctrl('d') => {
                if !self.containers.is_empty() {
                    self.selected = (self.selected + 10).min(max_index);
                }
                Some(AppCommand::NoOp)
            }
            Key::PageUp | Key::Ctrl('u') => {
                if !self.containers.is_empty() {
                    self.selected = if self.selected >= 10 {
                        self.selected - 10
                    } else {
                        0
                    };
                }
                Some(AppCommand::NoOp)
            }
            Key::End | Key::Char('G') => {
                if !self.containers.is_empty() {
                    self.selected = max_index;
                }
                Some(AppCommand::NoOp)
            }
            Key::Home | Key::Char('g') => {
                if !self.containers.is_empty() {
                    self.selected = 0;
                }
                Some(AppCommand::NoOp)
            }
            Key::Char('a') => {
                self.only_running = !self.only_running;
                Some(AppCommand::Refresh)
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

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
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
