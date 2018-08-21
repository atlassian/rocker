//! This module contains all the different views of the application.
use std::sync::Arc;

use shiplift::Docker;
use termion::event::Key;
use tui::{backend::MouseBackend, layout::Rect, Terminal};

use app::{AppCommand, ContainerId};

mod container_details;
mod container_list;
mod container_logs;
mod docker_info;
mod help;

pub use self::container_details::*;
pub use self::container_list::*;
pub use self::container_logs::*;
pub use self::docker_info::*;
pub use self::help::*;

/// This trait represents a view of the application i.e. a component that knows how to display
/// itself, how to handle input, and how to refresh itself.
pub trait View {
    /// Handle input for this view.
    ///
    /// If the view can handle this key, it should return an `AppCommand` (which can potentially be
    /// NoOp). Otherwise, it should return `None`.
    fn handle_input(&mut self, key: Key) -> Option<AppCommand>;

    /// Refresh the data displayed by this view (potentially using the provided handle to the
    /// Docker API). The default implementation doesn't do anything.
    fn refresh(&mut self, _docker: Arc<Docker>) {}

    /// Draws the view in the given area.
    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: Rect);
}

/// The different views that the application supports
#[derive(Debug, Clone, PartialEq)]
pub enum ViewType {
    Help,
    ContainerList,
    ContainerDetails(ContainerId),
    ContainerLogs(ContainerId),
    DockerInfo,
}
