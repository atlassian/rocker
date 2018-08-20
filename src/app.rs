use std::collections::VecDeque;

use shiplift::{
    rep::{Container, Info, Version},
    ContainerListOptions, Docker,
};
use termion::event::Key;
use tui::layout::Rect;

/// Contains the state of the application.
pub struct App {
    /// The client used to access the Docker daemon
    pub docker: Docker,
    /// The current size of the application
    pub size: Rect,
    /// Version info of the Docker daemon
    pub version: Version,
    /// System info of the Docker daemon
    pub info: Info,
    /// List of containers to display
    pub containers: Vec<Container>,
    /// Index of the currently selected container from the above list
    pub selected: usize,
    /// Whether to only display currently running containers
    pub only_running: bool,
    /// View stack: The top (=front) of the stack is the view that is displayed
    pub view_stack: VecDeque<AppState>,
}

impl App {
    /// Create a new instance of `App`. It will initialize the Docker client and make a couple of
    /// calls to the Docker daemon to get some system info and version info.
    pub fn new() -> App {
        let docker = Docker::new();
        let info = docker.info().unwrap();
        let version = docker.version().unwrap();
        let mut views = VecDeque::new();
        views.push_front(AppState::ContainerList);
        App {
            docker,
            size: Rect::default(),
            version,
            info,
            containers: Vec::new(),
            selected: 0,
            only_running: true,
            view_stack: views,
        }
    }

    /// Refreshes the state of the application (i.e. list of containers, system information, etc).
    pub fn refresh(&mut self) {
        let options = if self.only_running {
            ContainerListOptions::builder().build()
        } else {
            ContainerListOptions::builder().all().build()
        };
        let containers = self.docker.containers().list(&options).unwrap();
        let info = self.docker.info().unwrap();
        self.containers = containers;
        self.info = info;
        if self.containers.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.containers.len() {
            self.selected = self.containers.len() - 1;
        }
    }

    /// Returns the currently selected container, or `None` if there are no containers.
    pub fn get_selected_container(&self) -> Option<&Container> {
        self.containers.get(self.selected)
    }

    pub fn new_view(&mut self, state: AppState) {
        self.view_stack.push_front(state);
    }

    pub fn previous_view(&mut self) -> bool {
        if let Some(_state) = self.view_stack.pop_front() {
            true
        } else {
            false
        }
    }

    pub fn current_view(&self) -> &AppState {
        self.view_stack.front().expect("View stack is empty!")
    }

    pub fn handle_input(&mut self, key: Key) -> bool {
        match key {
            Key::Char('q') => {
                if !self.previous_view() {
                    return false;
                }
            }
            Key::Down => {
                if !self.containers.is_empty() {
                    self.selected += 1;
                    if self.selected > self.containers.len() - 1 {
                        self.selected = 0;
                    }
                }
            }
            Key::Up => if !self.containers.is_empty() {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.containers.len() - 1;
                }
            },
            Key::Char('\n') => {
                let container = self.selected;
                self.new_view(AppState::ContainerDetails(ContainerId(container)));
            }
            Key::Char('d') => self.new_view(AppState::DaemonInfo),
            // event::Key::Left => app.tabs.previous(),
            // event::Key::Right => app.tabs.next(),
            Key::Char('a') => {
                self.only_running = !self.only_running;
                self.refresh();
            }
            _ => {}
        };
        true
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ContainerId(pub usize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AppState {
    ContainerList,
    ContainerDetails(ContainerId),
    ContainerLogs(ContainerId),
    ContainerStats(ContainerId),
    DaemonInfo,
}

pub trait View {
    fn handle_input(&mut self, app: &App, key: Key);
}

pub struct ContainerListView {}

impl View for ContainerListView {
    fn handle_input(&mut self, app: &App, key: Key) {}
}
