extern crate byteorder;
extern crate bytesize;
extern crate crossbeam_channel;
extern crate failure;
#[macro_use]
extern crate log;
extern crate shiplift;
extern crate termion;
extern crate tui;
extern crate tui_logger;

mod app;
mod tty;
mod views;

use crossbeam_channel::unbounded;
use std::io;
use std::thread;
use std::time::Duration;

use log::LevelFilter;
use termion::{
    input::{MouseTerminal, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, Terminal};
use tui_logger::{init_logger, set_default_level, set_level_for_target};

use app::{App, AppEvent};

type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>;

fn main() {
    // Initialise logger
    init_logger(LevelFilter::Trace).unwrap();
    set_default_level(LevelFilter::Info);
    set_level_for_target("rkr", LevelFilter::Trace);
    info!("Logging system initialised");

    let (tx, rx) = unbounded();
    let input_tx = tx.clone();
    let tick_tx = tx.clone();

    // App
    let mut app =
        App::new().unwrap_or_else(|e| panic!("Failed to connect to the Docker daemon: {}", e));

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
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
            input_tx.send(AppEvent::Input(key));
        }
    });

    // Ticking thread
    thread::spawn(move || loop {
        tick_tx.send(AppEvent::Tick);
        trace!("Tick");
        thread::sleep(Duration::from_secs(2));
    });

    // Main event loop
    info!("Starting main event loop");
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
}
