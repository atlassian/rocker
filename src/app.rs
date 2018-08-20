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
    style::{Color, Style},
    widgets::{Paragraph, Widget},
    Terminal,
};

use views::{ContainerListView, View};

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
