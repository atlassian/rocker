use std::collections::VecDeque;
use std::sync::Arc;

use shiplift::{
    rep::{Info, Version},
    Docker,
};
use termion::event::Key;
use tui::{
    backend::{Backend, MouseBackend},
    layout::{Direction, Group, Rect, Size},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Widget},
    Terminal,
};

use views::{ContainerInfo, ContainerListView, DockerInfo, View, ViewType};

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

    pub fn new_view(&mut self, view_type: ViewType) {
        let new_view = match view_type {
            ViewType::ContainerDetails(id) => Box::new(ContainerInfo::new(id)) as Box<dyn View>,
            ViewType::DockerInfo => Box::new(DockerInfo::new()) as Box<dyn View>,
            _ => unimplemented!(),
        };

        self.view_stack.push_front(new_view);
    }

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
        let command = self
            .handle_global_keys(key)
            .or_else(|| self.current_view_mut().handle_input(key))
            .unwrap_or(AppCommand::NoOp);

        match command {
            AppCommand::SwitchToView(view_type) => {
                self.new_view(view_type);
            }
            AppCommand::ExitView => {
                return self.previous_view();
            }
            AppCommand::NoOp => { /* NoOp */ }
        }

        true
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

    fn handle_global_keys(&mut self, key: Key) -> Option<AppCommand> {
        match key {
            Key::Char('q') => Some(AppCommand::ExitView),
            Key::Char('d') => Some(AppCommand::SwitchToView(ViewType::DockerInfo)),
            _ => None,
        }
    }

    fn draw_status_bar<B: Backend>(&self, t: &mut Terminal<B>, rect: &Rect) {
        Paragraph::default()
            .wrap(true)
            .style(
                Style::default()
                    .bg(Color::Black)
                    .fg(Color::White)
                    .modifier(Modifier::Bold),
            )
            .text(&format!(
                " {version}      {containers}, {images}, {docker_version}",
                version = "{fg=black Rocker v0.1}",
                containers = format!("{{fg=light_green {}}} containers", self.info.Containers),
                images = format!("{{fg=green {}}} images", self.info.Images),
                docker_version = format!(
                    "docker v{} ({})",
                    self.version.Version, self.version.ApiVersion
                ),
            ))
            .render(t, rect);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    ExitView,
    SwitchToView(ViewType),
    NoOp,
}
