//! This module contains all the different views of the application.
use std::sync::Arc;
use std::time::Duration;

use termion::event::Key;
use tui::{layout::Rect, Frame};

use crate::app::{AppCommand, ContainerId};
use crate::docker::DockerExecutor;
use crate::Backend;

mod app_logs;
mod container_details;
mod container_list;
// mod container_logs;
mod docker_info;
mod help;
mod images_list;

pub use self::app_logs::*;
pub use self::container_details::*;
pub use self::container_list::*;
// pub use self::container_logs::*;
pub use self::docker_info::*;
pub use self::help::*;
pub use self::images_list::*;

/// This trait represents a view of the application i.e. a component that knows how to display
/// itself, how to handle input, and how to refresh itself.
pub trait View {
    /// Handle input for this view.
    ///
    /// If the view can handle this key, it should return an `AppCommand` (which can potentially be
    /// NoOp). Otherwise, it should return `None`.
    fn handle_input(&mut self, key: Key, docker: Arc<DockerExecutor>) -> Option<AppCommand>;

    /// Refresh the data displayed by this view (potentially using the provided handle to the
    /// Docker API). The default implementation doesn't do anything.
    fn refresh(&mut self, _docker: Arc<DockerExecutor>) {}

    /// Draws the view in the given area.
    fn draw(&self, t: &mut Frame<Backend>, rect: Rect);
}

/// The different views that the application supports
#[derive(Debug, Clone, PartialEq)]
pub enum ViewType {
    Help,
    AppLogs,
    ContainerList,
    ContainerDetails(ContainerId),
    ContainerLogs(ContainerId),
    DockerInfo,
    ImagesList,
}

pub fn human_duration(d: &Duration) -> String {
    let seconds = d.as_secs();

    if seconds < 1 {
        return "Less than a second".to_string();
    } else if seconds == 1 {
        return "1 second".to_string();
    } else if seconds < 60 {
        return format!("{} seconds", seconds);
    }

    let minutes = seconds / 60;
    if minutes == 1 {
        return "About 1 minute".to_string();
    } else if minutes < 46 {
        return format!("{} minutes", minutes);
    }

    let hours = ((minutes as f64 / 60.0) + 0.5) as u64;
    if hours == 1 {
        return "About 1 hour".to_string();
    } else if hours < 48 {
        return format!("{} hours", hours);
    } else if hours < 24 * 7 * 2 {
        return format!("{} days", hours / 24);
    } else if hours < 24 * 30 * 2 {
        return format!("{} weeks", hours / 24 / 7);
    } else if hours < 24 * 365 * 2 {
        return format!("{} months", hours / 24 / 30);
    }

    return format!("{} years", hours / 24 / 365);
}
