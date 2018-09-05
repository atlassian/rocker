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
            Text::Data("KEYS:\n"),
            Text::Data("? - help"),
            Text::Data("q - exit view"),
            Text::Data("i - switch to view: images list"),
            Text::Data("v - switch to view: docker info"),
            Text::Data("L - switch to view: application logs"),
            Text::Data("k - up"),
            Text::Data("j - down"),
            Text::Data("s - stop container      in view: container list"),
            Text::Data("S - start container     in view: container list"),
            Text::Data("p - pause container     in view: container list"),
            Text::Data("P - unpause container   in view: container list"),
            Text::Data("d - delete container    in view: container list"),
            Text::Data("l - container logs      in view: container list"),
            Text::Data("\u{23CE} - container details   in view: container list"),
        ];

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, rect);
    }
}
