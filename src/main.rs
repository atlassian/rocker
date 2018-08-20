extern crate failure;
extern crate humantime;
extern crate shiplift;
extern crate termion;
extern crate tui;

mod app;
// mod ui;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::{event, input::TermRead};
use tui::{backend::MouseBackend, Terminal};

use app::App;

pub enum Event {
    Input(event::Key),
    Tick,
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let input_tx = tx.clone();
    let tick_tx = tx.clone();

    // App
    let mut app = App::new();

    // Terminal initialization
    let backend = MouseBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();

    // First draw call
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    app.size = terminal.size().unwrap();
    app.draw(&mut terminal);

    // Input handling thread
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            let key = c.unwrap();
            input_tx.send(Event::Input(key)).unwrap();
        }
    });
    // Ticking thread
    thread::spawn(move || loop {
        tick_tx.send(Event::Tick).unwrap();
        thread::sleep(Duration::from_secs(2));
    });

    app.refresh();
    loop {
        // handle resize
        let size = terminal.size().unwrap();
        if size != app.size {
            terminal.resize(size).unwrap();
            app.size = size;
        }

        // Draw app
        app.draw(&mut terminal);

        // Handle events
        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(key) => {
                if !app.handle_input(key) {
                    break;
                }
            }
            Event::Tick => app.refresh(),
        };
    }

    terminal.show_cursor().unwrap();
    terminal.clear().unwrap();
}
