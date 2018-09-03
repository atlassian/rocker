use std::collections::VecDeque;
use std::sync::Arc;

use failure::*;
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

use views::{
    AppLogsView, ContainerInfo, ContainerListView, ContainerLogsView, DockerInfo, HelpView,
    ImagesListView, View, ViewType,
};

/// The event type used in the main event loop of the application.
pub enum AppEvent {
    /// Represents a key press
    Input(Key),
    /// Represents a periodic tick, used to trigger refresh of the views
    Tick,
}

/// Contains the state of the application.
pub struct App {
    /// The client used to access the Docker daemon
    docker: Arc<Docker>,
    /// The current size of the application
    pub size: Rect,
    /// Version info of the Docker daemon
    docker_version: Version,
    /// System info of the Docker daemon
    info: Info,
    /// View stack: The top (=front) of the stack is the view that is displayed
    view_stack: VecDeque<Box<View>>,
    err_msg: Option<String>,
}

impl App {
    /// Create a new instance of `App`. It will initialize the Docker client and make a couple of
    /// calls to the Docker daemon to get some system info and version info.
    pub fn new() -> Result<App, Error> {
        let docker = Arc::new(Docker::new());
        let info = docker.info()?;
        let docker_version = docker.version()?;
        let mut app = App {
            docker,
            size: Rect::default(),
            docker_version,
            info,
            view_stack: VecDeque::new(),
            err_msg: None,
        };
        app.new_view(ViewType::ContainerList);

        Ok(app)
    }

    /// Refreshes the state of the application (i.e. list of containers, system information, etc).
    pub fn refresh(&mut self) {
        let info = self.docker.info().unwrap();
        self.info = info;
        let docker = self.docker.clone();
        self.current_view_mut().refresh(docker);
    }

    /// Handles the given key press. Returns `false` to signify to the main loop that the
    /// application should exit.
    pub fn handle_input(&mut self, key: Key) -> bool {
        let docker = self.docker.clone();
        let command = self
            .handle_global_keys(key)
            .or_else(|| self.current_view_mut().handle_input(key, docker))
            .unwrap_or(AppCommand::NoOp);

        match command {
            AppCommand::SwitchToView(view_type) => {
                self.new_view(view_type);
                self.refresh();
            }
            AppCommand::ExitView => {
                return self.previous_view();
            }
            AppCommand::NoOp => { /* NoOp */ }
            AppCommand::ErrorMsg(msg) => self.err_msg = Some(msg),
        }

        self.refresh();

        true
    }

    /// Draws the application in the given terminal.
    pub fn draw(&self, t: &mut Terminal<MouseBackend>) {
        let size = t.size().unwrap();
        let main_view_height = size.height - 2;

        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[
                Size::Fixed(1),
                Size::Fixed(main_view_height),
                Size::Fixed(1),
            ])
            .margin(0)
            .render(t, &self.size, |t, chunks| {
                // title bar
                self.draw_status_bar(t, chunks[0]);

                // current view
                self.current_view().draw(t, chunks[1]);

                // Status message
                self.draw_status_message(t, chunks[2]);
            });

        t.draw().unwrap();
    }

    /// Instantiate a view of the type `view_type` and pushes it onto the view stack.
    fn new_view(&mut self, view_type: ViewType) {
        let new_view = match view_type {
            ViewType::ContainerList => Box::new(ContainerListView::new()) as Box<dyn View>,
            ViewType::ContainerDetails(id) => Box::new(ContainerInfo::new(id)) as Box<dyn View>,
            ViewType::ContainerLogs(id) => Box::new(ContainerLogsView::new(id)) as Box<dyn View>,
            ViewType::DockerInfo => Box::new(DockerInfo::new()) as Box<dyn View>,
            ViewType::Help => Box::new(HelpView::new()) as Box<dyn View>,
            ViewType::ImagesList => Box::new(ImagesListView::new()) as Box<dyn View>,
            ViewType::AppLogs => Box::new(AppLogsView::new()) as Box<dyn View>,
        };

        self.view_stack.push_front(new_view);
    }

    /// Pop the current view from the top of the stack. Returns `true` if there are still views in
    /// the stack afterwards, `false` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if the view stack is empty.
    fn previous_view(&mut self) -> bool {
        if let Some(_old_state) = self.view_stack.pop_front() {
            !self.view_stack.is_empty()
        } else {
            panic!("View stack was empty!");
        }
    }

    /// Returns a reference to the currently displayed view (i.e. at the top of the stack).
    ///
    /// # Panics
    ///
    /// Panics if the view stack is empty.
    fn current_view(&self) -> &dyn View {
        self.view_stack
            .front()
            .expect("View stack is empty!")
            .as_ref()
    }

    /// Returns a mutable reference to the currently displayed view (i.e. at the top of the stack).
    ///
    /// # Panics
    ///
    /// Panics if the view stack is empty.
    fn current_view_mut(&mut self) -> &mut dyn View {
        self.view_stack
            .front_mut()
            .expect("View stack is empty!")
            .as_mut()
    }

    /// Handle global shortcuts. If not handled here, then the current view will get a chance to
    /// handle it.
    fn handle_global_keys(&mut self, key: Key) -> Option<AppCommand> {
        match key {
            Key::Char('q') => Some(AppCommand::ExitView),
            Key::Char('i') => Some(AppCommand::SwitchToView(ViewType::ImagesList)),
            Key::Char('v') => Some(AppCommand::SwitchToView(ViewType::DockerInfo)),
            Key::Char('?') => Some(AppCommand::SwitchToView(ViewType::Help)),
            Key::Char('L') => Some(AppCommand::SwitchToView(ViewType::AppLogs)),
            _ => None,
        }
    }

    /// Draws the title bar at the top
    fn draw_status_bar<B: Backend>(&self, t: &mut Terminal<B>, rect: Rect) {
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
                version = "{fg=white Rocker v0.1}",
                containers = format!("{{fg=light_green {}}} containers", self.info.Containers),
                images = format!("{{fg=light_green {}}} images", self.info.Images),
                docker_version = format!(
                    "docker v{} ({})",
                    self.docker_version.Version, self.docker_version.ApiVersion
                ),
            ))
            .render(t, &rect);
    }

    fn draw_status_message<B: Backend>(&self, t: &mut Terminal<B>, rect: Rect) {
        if let Some(ref msg) = self.err_msg {
            Paragraph::default()
                .wrap(true)
                .style(
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::White)
                        .modifier(Modifier::Bold),
                )
                .text(msg)
                .render(t, &rect);
        } else {
            Paragraph::default()
                .wrap(true)
                .style(Style::default().bg(Color::Black).fg(Color::White))
                .text("No message")
                .render(t, &rect);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    ExitView,
    SwitchToView(ViewType),
    NoOp,
    ErrorMsg(String),
}
