use std::sync::Arc;

use shiplift::{rep::ContainerDetails, Docker};
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};

use app::{AppCommand, ContainerId};
use views::View;

pub struct HelpView {
    scroll: u16,
}

impl HelpView {
    pub fn new() -> HelpView {
        HelpView { scroll: 0 }
    }
}

impl View for HelpView {
    fn handle_input(&mut self, key: Key) -> Option<AppCommand> {
        match key {
            Key::Up | Key::Char('k') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                Some(AppCommand::NoOp)
            }
            Key::Down | Key::Char('j') => {
                self.scroll += 1;
                Some(AppCommand::NoOp)
            }
            _ => None,
        }
    }

    fn refresh(&mut self, _docker: Arc<Docker>) {}

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: &Rect) {
        Paragraph::default()
            .block(Block::default().borders(Borders::ALL))
            .text("TODO: Insert some help here...")
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, rect);
    }
}
