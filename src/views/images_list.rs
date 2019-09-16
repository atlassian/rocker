use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytesize;

use shiplift::{rep::Image, ImageListOptions};
use termion::event::Key;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Widget},
    Frame,
};

use crate::app::AppCommand;
use crate::docker::DockerExecutor;
use crate::views::{human_duration, View};
use crate::Backend;

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
    fn handle_input(&mut self, key: Key, _docker: Arc<DockerExecutor>) -> Option<AppCommand> {
        let max_index = self.images.len() - 1;
        match key {
            Key::Down | Key::Char('j') => {
                if !self.images.is_empty() {
                    self.selected = (self.selected + 1).min(max_index);
                }
                Some(AppCommand::NoOp)
            }
            Key::Up | Key::Char('k') => {
                if !self.images.is_empty() && self.selected > 0 {
                    self.selected -= 1;
                }
                Some(AppCommand::NoOp)
            }
            Key::PageDown | Key::Ctrl('d') => {
                if !self.images.is_empty() {
                    self.selected = (self.selected + 10).min(max_index);
                }
                Some(AppCommand::NoOp)
            }
            Key::PageUp | Key::Ctrl('u') => {
                if !self.images.is_empty() {
                    self.selected = if self.selected >= 10 {
                        self.selected - 10
                    } else {
                        0
                    };
                }
                Some(AppCommand::NoOp)
            }
            Key::End | Key::Char('G') => {
                if !self.images.is_empty() {
                    self.selected = max_index;
                }
                Some(AppCommand::NoOp)
            }
            Key::Home | Key::Char('g') => {
                if !self.images.is_empty() {
                    self.selected = 0;
                }
                Some(AppCommand::NoOp)
            }
            _ => None,
        }
    }

    fn refresh(&mut self, docker: Arc<DockerExecutor>) {
        let options = ImageListOptions::builder().all(true).build();
        let images = docker.images(&options).unwrap();
        self.images = images;
        if self.images.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.images.len() {
            self.selected = self.images.len() - 1;
        }
    }

    fn draw(&self, t: &mut Frame<Backend>, rect: Rect) {
        let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);
        let header = ["Image ID", "Parent", "Tag", "Created", "Virtual Size"];
        let height = rect.height as usize - 4; // 2 for border + 2 for header
        let offset = if self.selected >= height {
            self.selected - height + 1
        } else {
            0
        };
        let rows: Vec<_> = self
            .images
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let creation_timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(c.created);
                let duration = creation_timestamp.elapsed().unwrap();
                let mut duration_str = human_duration(&duration);
                duration_str.push_str(" ago");
                let id = if c.id.starts_with("sha256:") {
                    (&c.id[7..17]).to_string()
                } else {
                    c.id.clone()
                };
                let parent = if c.parent_id.starts_with("sha256:") {
                    (&c.parent_id[7..17]).to_string()
                } else {
                    c.parent_id.clone()
                };
                let data: Vec<String> = vec![
                    id,
                    parent,
                    c.repo_tags
                        .as_ref()
                        .and_then(|tags| tags.first())
                        .cloned()
                        .unwrap_or_else(|| "<none>".to_string()),
                    duration_str,
                    bytesize::to_string(c.virtual_size, false),
                ];
                if i == self.selected {
                    Row::StyledData(data.into_iter(), selected_style)
                } else {
                    Row::StyledData(data.into_iter(), normal_style)
                }
            })
            .skip(offset)
            .collect();

        Table::new(header.iter(), rows.into_iter())
            .block(Block::default().borders(Borders::ALL))
            .widths(&[10, 10, 45, 15, 20]) // TODO be smarter with sizes here
            .render(t, rect);
    }
}
