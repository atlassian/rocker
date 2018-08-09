extern crate failure;
extern crate rust_docker;
extern crate termion;
extern crate tui;

use std::io;

use termion::event;
use termion::input::TermRead;

use tui::backend::MouseBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::Terminal;

use rust_docker::api::containers::Containers;
use rust_docker::api::version::Version;
use rust_docker::DockerClient;

struct App<'a> {
    size: Rect,
    items: Vec<Vec<&'a str>>,
    selected: usize,
}

impl<'a> App<'a> {
    fn new(data: Vec<Vec<&'a str>>) -> App<'a> {
        App {
            size: Rect::default(),
            items: data,
            selected: 0,
        }
    }
}

fn main() {
    println!("Connecting to docker daemon...");
    let docker = DockerClient::new("unix:///var/run/docker.sock").unwrap();
    // let version = docker.get_version_info().unwrap();
    // println!("Docker daemon version {}", version);
    println!("Getting list of running containers...");
    let containers = docker.list_running_containers(None).unwrap();
    let data = containers
        .iter()
        .map(|c| vec![c.Id.as_ref(), c.Image.as_ref(), c.Command.as_ref()])
        .collect();
    // App
    let mut app = App::new(data);

    // Terminal initialization
    let backend = MouseBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();

    // First draw call
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    app.size = terminal.size().unwrap();
    draw(&mut terminal, &app);

    // Input
    let stdin = io::stdin();
    for c in stdin.keys() {
        let size = terminal.size().unwrap();
        if size != app.size {
            terminal.resize(size).unwrap();
            app.size = size;
        }

        let evt = c.unwrap();
        match evt {
            event::Key::Char('q') => {
                break;
            }
            event::Key::Down => {
                app.selected += 1;
                if app.selected > app.items.len() - 1 {
                    app.selected = 0;
                }
            }
            event::Key::Up => if app.selected > 0 {
                app.selected -= 1;
            } else {
                app.selected = app.items.len() - 1;
            },
            _ => {}
        };
        draw(&mut terminal, &app);
    }

    terminal.show_cursor().unwrap();
    terminal.clear().unwrap();
}

fn draw(t: &mut Terminal<MouseBackend>, app: &App) {
    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
    let normal_style = Style::default().fg(Color::White);
    let header = ["Container ID", "Image", "Command"];
    let rows = app.items.iter().enumerate().map(|(i, item)| {
        if i == app.selected {
            Row::StyledData(item.into_iter(), &selected_style)
        } else {
            Row::StyledData(item.into_iter(), &normal_style)
        }
    });
    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(100)])
        .margin(0)
        .render(t, &app.size, |t, chunks| {
            Table::new(header.into_iter(), rows)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Running Containers"),
                )
                .widths(&[15, 20, 50])
                .render(t, &chunks[0]);
        });

    t.draw().unwrap();
}
