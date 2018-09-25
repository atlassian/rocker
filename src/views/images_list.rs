use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytesize;

use shiplift::{rep::Image, Docker, ImageListOptions};
use termion::event::Key;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Widget},
    Frame,
};

use app::AppCommand;
use views::{human_duration, View};
use Backend;

pub struct ImagesListView {
    images: Vec<Image>,
    selected: usize,
}

impl ImagesListView {
    pub fn new() -> ImagesListView {
        ImagesListView {
            images: Vec::new(),
            selected: 0,
        }
    }
}

impl View for ImagesListView {
    fn handle_input(&mut self, key: Key, _docker: Arc<Docker>) -> Option<AppCommand> {
        match key {
            Key::Down | Key::Char('j') => {
                if !self.images.is_empty() {
                    self.selected += 1;
                    if self.selected > self.images.len() - 1 {
                        self.selected = 0;
                    }
                }
                Some(AppCommand::NoOp)
            }
            Key::Up | Key::Char('k') => {
                if !self.images.is_empty() {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = self.images.len() - 1;
                    }
                }
                Some(AppCommand::NoOp)
            }
            _ => None,
        }
    }

    fn refresh(&mut self, docker: Arc<Docker>) {
        let options = ImageListOptions::builder().all(true).build();
        let images = docker.images().list(&options).unwrap();
        self.images = images;
        if self.images.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.images.len() {
            self.selected = self.images.len() - 1;
        }
    }

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
        let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
        let normal_style = Style::default().fg(Color::White);
        let header = ["Image ID", "Tag", "Created", "Virtual Size"];
        let rows: Vec<_> = self
            .images
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let creation_timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(c.Created);
                let duration = creation_timestamp.elapsed().unwrap();
                let mut duration_str = human_duration(&duration);
                duration_str.push_str(" ago");
                let id = if c.Id.starts_with("sha256:") {
                    (&c.Id[7..]).to_string()
                } else {
                    c.Id.clone()
                };
                let data: Vec<String> = vec![
                    id,
                    c.RepoTags
                        .as_ref()
                        .and_then(|tags| tags.first())
                        .cloned()
                        .unwrap_or_else(|| "<none>".to_string()),
                    duration_str,
                    bytesize::to_string(c.VirtualSize, false),
                ];
                if i == self.selected {
                    Row::StyledData(data.into_iter(), selected_style)
                } else {
                    Row::StyledData(data.into_iter(), normal_style)
                }
            }).collect();

        Table::new(header.into_iter(), rows.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .widths(&[10, 45, 15, 20]) // TODO be smarter with sizes here
            .render(t, rect);
    }
}
