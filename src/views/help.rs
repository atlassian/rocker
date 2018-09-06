use std::sync::Arc;

use shiplift::Docker;
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

use app::AppCommand;
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
    fn handle_input(&mut self, key: Key, _docker: Arc<Docker>) -> Option<AppCommand> {
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

    fn draw(&self, t: &mut Frame<MouseBackend>, rect: Rect) {
        let text = vec![
            Text::data("KEYS:\n"),
            Text::data("? - help\n"),
            Text::data("q - exit view\n"),
            Text::data("i - switch to view: images list\n"),
            Text::data("v - switch to view: docker info\n"),
            Text::data("L - switch to view: application logs\n"),
            Text::data("k - up\n"),
            Text::data("j - down\n"),
            Text::data("s - stop container      in view: container list\n"),
            Text::data("S - start container     in view: container list\n"),
            Text::data("p - pause container     in view: container list\n"),
            Text::data("P - unpause container   in view: container list\n"),
            Text::data("d - delete container    in view: container list\n"),
            Text::data("l - container logs      in view: container list\n"),
            Text::data("\u{23CE} - container details   in view: container list\n"),
        ];

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, rect);
    }
}
