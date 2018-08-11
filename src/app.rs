use shiplift::builder::ContainerListOptions;
use shiplift::rep::{Container, Info, Version};
use shiplift::Docker;
use tui::layout::Rect;

pub struct App {
    pub docker: Docker,
    pub size: Rect,
    pub version: Version,
    pub info: Info,
    pub containers: Vec<Container>,
    pub selected: usize,
}

impl App {
    pub fn new() -> App {
        let docker = Docker::new();
        let info = docker.info().unwrap();
        let version = docker.version().unwrap();
        App {
            docker,
            size: Rect::default(),
            version,
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

    pub fn get_selected_container(&self) -> Option<&Container> {
        self.containers.get(self.selected)
    }
}
