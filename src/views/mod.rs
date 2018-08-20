use std::sync::Arc;

use shiplift::Docker;
use termion::event::Key;
use tui::{backend::MouseBackend, layout::Rect, Terminal};

mod container_list;

pub use self::container_list::*;

pub trait View {
    fn handle_input(&mut self, key: Key) -> Option<bool>;
    fn refresh(&mut self, _docker: Arc<Docker>) {}
    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: &Rect);
}
