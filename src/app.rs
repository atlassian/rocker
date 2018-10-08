use std::collections::VecDeque;
use std::sync::Arc;

use failure::*;
use shiplift::{
    rep::{Info, Version},
    Docker,
};
use termion::event::Key;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Text, Widget},
    Frame, Terminal,
};

use views::{
    AppLogsView, ContainerInfo, ContainerListView, ContainerLogsView, DockerInfo, HelpView,
    ImagesListView, View, ViewType,
};
use Backend;

// Stolen from clap
macro_rules! crate_version {
    () => {
        format!(
            "{}.{}.{}-{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
        )
    };
}

/// The event type used in the main event loop of the application.
pub enum AppEvent {
    /// Represents a key press
    Input(Key),
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
            AppCommand::Refresh => self.refresh(),
        }

        true
    }

    /// Draws the application in the given terminal.
    pub fn draw(&self, t: &mut Terminal<Backend>) {
        let size = t.size().unwrap();
        let main_view_height = size.height - 2;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(main_view_height),
                Constraint::Length(1),
            ]).margin(0)
            .split(size);

        t.draw(|mut f| {
            // title bar
            self.draw_status_bar(&mut f, chunks[0]);

            // current view
            self.current_view().draw(&mut f, chunks[1]);

            // Status message
            self.draw_status_message(&mut f, chunks[2]);
        }).unwrap();
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
            Key::Char('R') => Some(AppCommand::Refresh),
            _ => None,
        }
    }

    /// Draws the title bar at the top
    fn draw_status_bar(&self, t: &mut Frame<Backend>, rect: Rect) {
        let green = Style::default().fg(Color::LightGreen);
        let text = vec![
            Text::raw(format!("Rocker v{}     ", crate_version!())),
            Text::styled(format!("{}", self.info.containers), green),
            Text::raw(" containers, "),
            Text::styled(format!("{}", self.info.images), green),
            Text::raw(" images, "),
            Text::raw(format!(
                "docker v{} ({})",
                self.docker_version.version, self.docker_version.api_version
            )),
        ];

        Paragraph::new(text.iter())
            .wrap(true)
            .style(
                Style::default()
                    .bg(Color::Black)
                    .fg(Color::White)
                    .modifier(Modifier::Bold),
            ).render(t, rect);
    }

    fn draw_status_message(&self, t: &mut Frame<Backend>, rect: Rect) {
        let text = if let Some(ref msg) = self.err_msg {
            Text::styled(
                msg,
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .modifier(Modifier::Bold),
            )
        } else {
            Text::styled(
                "No message",
                Style::default().bg(Color::Black).fg(Color::White),
            )
        };

        Paragraph::new(vec![text].iter()).wrap(true).render(t, rect);
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
    Refresh,
}
