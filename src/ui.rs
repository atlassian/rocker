use std::time::Duration;

use humantime;

use app::{App, AppState, ContainerId};
use shiplift::rep::Port;
use tui::{
    backend::{Backend, MouseBackend},
    layout::{Direction, Group, Rect, Size},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Widget},
    Terminal,
};

pub fn draw(t: &mut Terminal<MouseBackend>, app: &App) {
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Fixed(1), Size::Percent(100)])
        .margin(0)
        .render(t, &app.size, |t, chunks| {
            // Status bar
            draw_status_bar(app, t, &chunks[0]);

            match *app.current_view() {
                AppState::ContainerList => draw_container_tab(app, t, &chunks[1]),
                AppState::ContainerDetails(id) => draw_container_details(app, id, t, &chunks[1]),
                AppState::DaemonInfo => draw_docker_tab(app, t, &chunks[1]),
                _ => unimplemented!(),
            }
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

fn draw_container_tab<B: Backend>(app: &App, t: &mut Terminal<B>, rect: &Rect) {
    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Percent(50), Size::Percent(50)])
        .margin(0)
        .render(t, rect, |t, chunks| {
            // Containers
            draw_container_list(app, t, &chunks[0]);

            // Container details
            draw_container_info(app, t, &chunks[1]);
        });
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
        .block(Block::default().borders(Borders::ALL))
        .widths(&[15, 20, 30, 20])
        .render(t, rect);
}

fn draw_container_info<B: Backend>(app: &App, t: &mut Terminal<B>, rect: &Rect) {
    let current_container = app.get_selected_container();
    Paragraph::default()
        .block(Block::default().borders(Borders::ALL))
        .wrap(true)
        .text(
            current_container
                .map(|c| {
                    let created_time = ::std::time::UNIX_EPOCH + Duration::from_secs(c.Created);
                    let duration = created_time.elapsed().unwrap();
                    // Truncate to second precision
                    let duration = Duration::new(duration.as_secs(), 0);
                    let mut ports = c.Ports.clone();
                    let ports_slice: &mut [Port] = ports.as_mut();
                    ports_slice.sort_by_key(|p: &Port| p.PrivatePort);
                    let ports_displayed = ports_slice
                        .iter()
                        .map(|p: &Port| display_port(p))
                        .collect::<Vec<_>>()
                        .join("\n                ");

                    format!(
                        "{{mod=bold {:15}}} {} ago\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {:?}\n\
                         {{mod=bold {:15}}} {}\n\
                         {{mod=bold {:15}}} {}\n\
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
                        c.Names.join(", "),
                        "Ports:",
                        ports_displayed,
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

fn draw_docker_tab<B: Backend>(app: &App, t: &mut Terminal<B>, rect: &Rect) {
    Paragraph::default()
        .text(&format!("{:#?}", app.info))
        .wrap(true)
        .raw(true)
        .render(t, rect);
}

fn draw_container_details<B: Backend>(
    app: &App,
    ContainerId(id): ContainerId,
    t: &mut Terminal<B>,
    rect: &Rect,
) {
    let container = &app.containers[id];
    let containers_api = app.docker.containers();
    let container_api = containers_api.get(container.Id.as_ref());
    let container_details = container_api.inspect().unwrap();

    Paragraph::default()
        .text(&format!("{:#?}", container_details))
        .wrap(true)
        .raw(true)
        .render(t, rect);
}

fn display_port(port: &Port) -> String {
    let mut s = String::new();
    if let Some(ref ip) = port.IP {
        s.push_str(&format!("{}:", ip));
    }
    s.push_str(&format!("{}", port.PrivatePort));
    if let Some(ref public_port) = port.PublicPort {
        s.push_str(&format!(" → {}", public_port));
    }
    s.push_str(&format!("/{}", port.Type));

    s
}
