extern crate failure;
extern crate shiplift;
extern crate termion;
extern crate tui;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event;
use termion::input::TermRead;

use tui::backend::MouseBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Item, List, Paragraph, Row, Table, Widget};
use tui::Terminal;

use shiplift::builder::ContainerListOptions;
use shiplift::rep::{Container, Info};
use shiplift::Docker;

struct App {
    docker: Docker,
    size: Rect,
    info: Info,
    containers: Vec<Container>,
    selected: usize,
}

impl App {
    pub fn new() -> App {
        let docker = Docker::new();
        let info = docker.info().unwrap();
        App {
            docker,
            size: Rect::default(),
            info,
            containers: Vec::new(),
            selected: 0,
        }
    }

    pub fn refresh(&mut self) {
        let containers = self
            .docker
            .containers()
            .list(&ContainerListOptions::builder().all().build())
            .unwrap();
        let info = self.docker.info().unwrap();
        self.containers = containers;
        self.info = info;
    }

    pub fn containers_data(&self) -> Vec<Vec<&str>> {
        self.containers
            .iter()
            .map(|c| {
                vec![
                    c.Id.as_ref(),
                    c.Image.as_ref(),
                    c.Command.as_ref(),
                    c.Status.as_ref(),
                ]
            })
            .collect()
    }
}

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
                    _ => {}
                };
            }
            Event::Tick => app.refresh(),
        };
    }

    terminal.show_cursor().unwrap();
    terminal.clear().unwrap();
}

fn draw(t: &mut Terminal<MouseBackend>, app: &App) {
    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
    let normal_style = Style::default().fg(Color::White);
    let header = ["Container ID", "Image", "Command", "Status"];
    let data = app.containers_data();
    let rows = data.iter().enumerate().map(|(i, item)| {
        if i == app.selected {
            Row::StyledData(item.into_iter(), &selected_style)
        } else {
            Row::StyledData(item.into_iter(), &normal_style)
        }
    });
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Fixed(1), Size::Percent(50), Size::Percent(50)])
        .margin(0)
        .render(t, &app.size, |t, chunks| {
            // Status bar
            Paragraph::default()
                .style(Style::default().bg(Color::Blue).fg(Color::White))
                .text(&format!(
                    "{{mod=bold Rocker v0.1}}   {} containers, {} images, {}",
                    app.info.Containers,
                    app.info.Images,
                    app.info
                        .SystemTime
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or("")
                ))
                .render(t, &chunks[0]);

            // Table
            Table::new(header.into_iter(), rows)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Running Containers"),
                )
                .widths(&[15, 20, 30, 20])
                .render(t, &chunks[1]);

            let data = vec![Item::Data("Foo"), Item::Data("Bar"), Item::Data("Doo")];
            List::new(data.into_iter())
                .block(Block::default().borders(Borders::ALL))
                .render(t, &chunks[2]);
        });

    t.draw().unwrap();
}
