extern crate failure;
extern crate humantime;
extern crate shiplift;
extern crate termion;
extern crate tui;

mod app;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use humantime::Timestamp;
use termion::event;
use termion::input::TermRead;
use tui::backend::{Backend, MouseBackend};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Widget};
use tui::Terminal;

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
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Fixed(1), Size::Percent(50), Size::Percent(50)])
        .margin(0)
        .render(t, &app.size, |t, chunks| {
            // Status bar
            draw_status_bar(app, t, &chunks[0]);

            // Containers
            draw_container_list(app, t, &chunks[1]);

            // Container details
            draw_container_details(app, t, &chunks[2]);
        });

    t.draw().unwrap();
}

fn draw_status_bar<B: Backend>(app: &App, t: &mut Terminal<B>, rect: &Rect) {
    Paragraph::default()
        .wrap(true)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .text(&format!(
            "{{mod=bold Rocker \\\\m/ v0.1}}   {} containers, {} images, docker v{} ({})",
            app.info.Containers, app.info.Images, app.version.Version, app.version.ApiVersion,
        ))
        .render(t, rect);
}

fn draw_container_list<B: Backend>(app: &App, t: &mut Terminal<B>, rect: &Rect) {
    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
    let normal_style = Style::default().fg(Color::White);
    let running_style = Style::default().fg(Color::Green);
    let header = ["Container ID", "Image", "Command", "Status"];
    let rows: Vec<_> = app
        .containers
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let data: Vec<&str> = vec![
                c.Id.as_ref(),
                c.Image.as_ref(),
                c.Command.as_ref(),
                c.Status.as_ref(),
            ];
            if i == app.selected {
                Row::StyledData(data.into_iter(), &selected_style)
            } else {
                if c.Status.starts_with("Up ") {
                    Row::StyledData(data.into_iter(), &running_style)
                } else {
                    Row::StyledData(data.into_iter(), &normal_style)
                }
            }
        })
        .collect();

    Table::new(header.into_iter(), rows.into_iter())
        .block(Block::default().borders(Borders::ALL).title(" Containers "))
        .widths(&[15, 20, 30, 20])
        .render(t, rect);
}

fn draw_container_details<B: Backend>(app: &App, t: &mut Terminal<B>, rect: &Rect) {
    let current_container = app.get_selected_container();
    Paragraph::default()
        .block(Block::default().borders(Borders::ALL))
        .wrap(true)
        .text(
            current_container
                .map(|c| {
                    let create_time = c.Created;
                    let formatted_time = ::std::time::UNIX_EPOCH + Duration::from_secs(create_time);
                    let duration = formatted_time.elapsed().unwrap();

                    format!(
                        "{{mod=bold {:15}}} {} ago\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {:?}\n\
                         {{mod=bold {:15}}} {:?}\n\
                         {{mod=bold {:15}}} {:?}\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {:?}\n\
                         {{mod=bold {:15}}} {:?}",
                        "Created:",
                        humantime::format_duration(duration),
                        "Command:",
                        c.Command,
                        "Id:",
                        c.Id,
                        "Image:",
                        c.Image,
                        "Labels:",
                        c.Labels,
                        "Names:",
                        c.Names,
                        "Ports:",
                        c.Ports,
                        "Status:",
                        c.Status,
                        "SizeRW:",
                        c.SizeRw,
                        "SizeRootFs:",
                        c.SizeRootFs,
                    )
                })
                .unwrap_or("".to_string())
                .as_str(),
        )
        .render(t, rect);
}
