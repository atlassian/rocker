extern crate failure;
extern crate humantime;
extern crate shiplift;
extern crate termion;
extern crate tui;

mod app;
mod views;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::input::TermRead;
use tui::{backend::MouseBackend, Terminal};

use app::{App, AppEvent};

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
            input_tx.send(AppEvent::Input(key)).unwrap();
        }
    });

    // Ticking thread
    thread::spawn(move || loop {
        tick_tx.send(AppEvent::Tick).unwrap();
        thread::sleep(Duration::from_secs(2));
    });

    // Main event loop
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
            AppEvent::Input(key) => {
                if !app.handle_input(key) {
                    break;
                }
            }
            AppEvent::Tick => app.refresh(),
        };
    }

    terminal.show_cursor().unwrap();
    terminal.clear().unwrap();
}
