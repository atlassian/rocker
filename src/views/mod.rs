use std::sync::Arc;

use shiplift::Docker;
use termion::event::Key;
use tui::{backend::MouseBackend, layout::Rect, Terminal};

use app::AppCommand;

mod container_details;
mod container_list;
mod docker_info;

pub use self::container_details::*;
pub use self::container_list::*;
pub use self::docker_info::*;

pub trait View {
    fn handle_input(&mut self, key: Key) -> Option<AppCommand>;
    fn refresh(&mut self, _docker: Arc<Docker>) {}
    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: &Rect);
}
