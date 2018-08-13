extern crate failure;
extern crate humantime;
extern crate shiplift;
extern crate termion;
extern crate tui;

mod app;
mod ui;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::{event, input::TermRead};
use tui::{backend::MouseBackend, Terminal};

use app::App;
use ui::draw;

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
    draw(&mut terminal, &app);

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
        draw(&mut terminal, &app);

        // Handle events
        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(key) => {
                match key {
                    event::Key::Char('q') => {
                        break;
                    }
                    event::Key::Down => {
                        app.selected += 1;
                        if app.selected > app.containers.len() - 1 {
                            app.selected = 0;
                        }
                    }
                    event::Key::Up => if app.selected > 0 {
                        app.selected -= 1;
                    } else {
                        app.selected = app.containers.len() - 1;
                    },
                    event::Key::Left => app.tabs.previous(),
                    event::Key::Right => app.tabs.next(),
                    event::Key::Char('a') => {
                        app.only_running = !app.only_running;
                        app.refresh();
                    }
                    _ => {}
                };
            }
            Event::Tick => app.refresh(),
        };
    }

    terminal.show_cursor().unwrap();
    terminal.clear().unwrap();
}
