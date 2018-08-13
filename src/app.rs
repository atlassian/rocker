use std::collections::VecDeque;

use shiplift::{
    rep::{Container, Info, Version},
    ContainerListOptions, Docker,
};
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
    /// List of tabs in the UI
    pub tabs: MyTabs,
    pub current_state: AppState,
    pub previous_states: VecDeque<AppState>,
}

impl App {
    /// Create a new instance of `App`. It will initialize the Docker client and make a couple of
    /// calls to the Docker daemon to get some system info and version info.
    pub fn new() -> App {
        let docker = Docker::new();
        let info = docker.info().unwrap();
        let version = docker.version().unwrap();
        App {
            docker,
            size: Rect::default(),
            version,
            info,
            containers: Vec::new(),
            selected: 0,
            only_running: true,
            tabs: MyTabs::new(),
            current_state: AppState::ContainerList,
            previous_states: VecDeque::new(),
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
        if self.selected >= self.containers.len() {
            self.selected = self.containers.len() - 1;
        }
    }

    /// Returns the currently selected container, or `None` if there are no containers.
    pub fn get_selected_container(&self) -> Option<&Container> {
        self.containers.get(self.selected)
    }

    pub fn new_view(&mut self, state: AppState) {
        self.previous_states.push_front(self.current_state);
        self.current_state = state;
    }

    pub fn previous_view(&mut self) -> bool {
        if let Some(state) = self.previous_states.pop_front() {
            self.current_state = state;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ContainerId(usize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AppState {
    ContainerList,
    DaemonInfo,
    ContainerLogs(ContainerId),
    ContainerStats(ContainerId),
}

pub struct MyTabs {
    pub titles: Vec<String>,
    pub selected: usize,
}

impl MyTabs {
    pub fn new() -> MyTabs {
        MyTabs {
            titles: vec!["Containers".into(), "Docker".into()],
            selected: 0,
        }
    }

    pub fn next(&mut self) {
        self.selected += 1;
        if self.selected >= self.titles.len() {
            self.selected = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.titles.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}
