use std::sync::Arc;

use shiplift::Docker;
use termion::event::Key;
use tui::{
    backend::MouseBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
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

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: Rect) {
        Paragraph::default()
            .block(Block::default().borders(Borders::ALL))
            .text(
                "KEYS:

? - help
q - exit view
i - switch to view: images list
v - switch to view: docker info
k - up
j - down
s - stop container      in view: container list
S - start container     in view: container list
p - pause container     in view: container list
P - unpause container   in view: container list
d - delete container    in view: container list
l - container logs      in view: container list
\u{23CE} - container details   in view: container list
",
            )
            .wrap(true)
            .scroll(self.scroll)
            .raw(true)
            .render(t, &rect);
    }
}
