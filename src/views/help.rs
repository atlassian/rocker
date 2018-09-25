use std::sync::Arc;

use shiplift::Docker;
use termion::event::Key;
use tui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

use app::AppCommand;
use views::View;
use Backend;

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

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
        let text = vec![
            Text::raw("KEYS:\n"),
            Text::raw("? - help\n"),
            Text::raw("q - exit view\n"),
            Text::raw("i - switch to view: images list\n"),
            Text::raw("v - switch to view: docker info\n"),
            Text::raw("L - switch to view: application logs\n"),
            Text::raw("k - up\n"),
            Text::raw("j - down\n"),
            Text::raw("s - stop container      in view: container list\n"),
            Text::raw("S - start container     in view: container list\n"),
            Text::raw("p - pause container     in view: container list\n"),
            Text::raw("P - unpause container   in view: container list\n"),
            Text::raw("d - delete container    in view: container list\n"),
            Text::raw("l - container logs      in view: container list\n"),
            Text::raw("\u{23CE} - container details   in view: container list\n"),
        ];

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, rect);
    }
}
