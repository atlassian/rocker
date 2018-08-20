use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use humantime;

use shiplift::{
    rep::{Container, Info, Port, Version},
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

// use ui;

/// Contains the state of the application.
pub struct App {
    /// The client used to access the Docker daemon
    pub docker: Arc<Docker>,
    /// The current size of the application
    pub size: Rect,
    /// Version info of the Docker daemon
    pub version: Version,
    /// System info of the Docker daemon
    pub info: Info,
    /// View stack: The top (=front) of the stack is the view that is displayed
    pub view_stack: VecDeque<Box<View>>,
}

impl App {
    /// Create a new instance of `App`. It will initialize the Docker client and make a couple of
    /// calls to the Docker daemon to get some system info and version info.
    pub fn new() -> App {
        let docker = Arc::new(Docker::new());
        let info = docker.info().unwrap();
        let version = docker.version().unwrap();
        let mut views: VecDeque<Box<View>> = VecDeque::new();
        views.push_front(Box::new(ContainerListView::new()));
        App {
            docker,
            size: Rect::default(),
            version,
            info,
            view_stack: views,
        }
    }

    /// Refreshes the state of the application (i.e. list of containers, system information, etc).
    pub fn refresh(&mut self) {
        let info = self.docker.info().unwrap();
        self.info = info;
        let docker = self.docker.clone();
        self.current_view_mut().refresh(docker);
    }

    // /// Returns the currently selected container, or `None` if there are no containers.
    // pub fn get_selected_container(&self) -> Option<&Container> {
    //     self.containers.get(self.selected)
    // }

    // pub fn new_view(&mut self, state: AppState) {
    //     self.view_stack.push_front(state);
    // }

    pub fn previous_view(&mut self) -> bool {
        if let Some(_old_state) = self.view_stack.pop_front() {
            !self.view_stack.is_empty()
        } else {
            panic!("View stack was empty!");
        }
    }

    pub fn current_view(&self) -> &dyn View {
        self.view_stack
            .front()
            .expect("View stack is empty!")
            .as_ref()
    }

    pub fn current_view_mut(&mut self) -> &mut dyn View {
        self.view_stack
            .front_mut()
            .expect("View stack is empty!")
            .as_mut()
    }

    pub fn handle_input(&mut self, key: Key) -> bool {
        self.handle_global_keys(key)
            .or_else(|| self.current_view_mut().handle_input(key))
            .unwrap_or(true)
    }

    pub fn draw(&self, t: &mut Terminal<MouseBackend>) {
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(1), Size::Percent(100)])
            .margin(0)
            .render(t, &self.size, |t, chunks| {
                // Status bar
                self.draw_status_bar(t, &chunks[0]);

                self.current_view().draw(t, &chunks[1]);
            });

        t.draw().unwrap();
    }

    fn handle_global_keys(&mut self, key: Key) -> Option<bool> {
        match key {
            Key::Char('q') => Some(self.previous_view()),
            // Key::Char('\n') => {
            //     let container = self.selected;
            //     self.new_view(AppState::ContainerDetails(ContainerId(container)));
            // }
            // Key::Char('d') => self.new_view(AppState::DaemonInfo),
            _ => None,
        }
    }

    fn draw_status_bar<B: Backend>(&self, t: &mut Terminal<B>, rect: &Rect) {
        Paragraph::default()
            .wrap(true)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .text(&format!(
                "{{mod=bold Rocker \\\\m/ v0.1}}   {} containers, {} images, docker v{} ({})",
                self.info.Containers,
                self.info.Images,
                self.version.Version,
                self.version.ApiVersion,
            ))
            .render(t, rect);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ContainerId(pub usize);

pub trait View {
    fn handle_input(&mut self, key: Key) -> Option<bool>;
    fn refresh(&mut self, _docker: Arc<Docker>) {}
    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: &Rect);
}

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

    fn draw_container_list<B: Backend>(&self, t: &mut Terminal<B>, rect: &Rect) {
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
                } else {
                    if c.Status.starts_with("Up ") {
                        Row::StyledData(data.into_iter(), &running_style)
                    } else {
                        Row::StyledData(data.into_iter(), &normal_style)
                    }
                }
            })
            .collect();

        Table::new(header.into_iter(), rows.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .widths(&[15, 20, 30, 20])
            .render(t, rect);
    }

    fn draw_container_info<B: Backend>(&self, t: &mut Terminal<B>, rect: &Rect) {
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
                    .unwrap_or("".to_string())
                    .as_str(),
            )
            .render(t, rect);
    }
}

impl View for ContainerListView {
    fn handle_input(&mut self, key: Key) -> Option<bool> {
        match key {
            Key::Down => {
                if !self.containers.is_empty() {
                    self.selected += 1;
                    if self.selected > self.containers.len() - 1 {
                        self.selected = 0;
                    }
                }
                Some(true)
            }
            Key::Up => {
                if !self.containers.is_empty() {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = self.containers.len() - 1;
                    }
                }
                Some(true)
            }
            Key::Char('a') => {
                self.only_running = !self.only_running;
                // self.refresh();
                Some(true)
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

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: &Rect) {
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Percent(50), Size::Percent(50)])
            .margin(0)
            .render(t, rect, |t, chunks| {
                // Containers
                self.draw_container_list(t, &chunks[0]);

                // Container details
                self.draw_container_info(t, &chunks[1]);
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
