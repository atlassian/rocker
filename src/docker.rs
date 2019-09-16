use std::sync::Mutex;

use failure::*;
use shiplift::{
    rep::{Container, ContainerDetails, Image, Info, Version},
    ContainerListOptions, Docker, ImageListOptions,
};
use tokio::prelude::Future;
use tokio::runtime::Runtime;

pub struct DockerExecutor {
    docker: Docker,
    runtime: Mutex<Runtime>,
}

impl DockerExecutor {
    pub fn new() -> Result<DockerExecutor, Error> {
        let rt = Runtime::new()?;
        let docker = Docker::new();
        Ok(DockerExecutor {
            docker,
            runtime: Mutex::new(rt),
        })
    }

    fn execute<F, U>(&self, fut: F) -> Result<U, Error>
    where
        U: Send + 'static,
        F: Send + 'static + Future<Item = U, Error = shiplift::errors::Error>,
    {
        let mut rt = self.runtime.lock().unwrap();
        Ok(rt.block_on(fut)?)
    }

    pub fn info(&self) -> Result<Info, Error> {
        self.execute(self.docker.info())
    }

    pub fn version(&self) -> Result<Version, Error> {
        self.execute(self.docker.version())
    }

    pub fn container(&self, name: &str) -> Result<ContainerDetails, Error> {
        self.execute(self.docker.containers().get(name).inspect())
    }

    pub fn containers(&self, opts: &ContainerListOptions) -> Result<Vec<Container>, Error> {
        self.execute(self.docker.containers().list(opts))
    }

    pub fn images(&self, opts: &ImageListOptions) -> Result<Vec<Image>, Error> {
        self.execute(self.docker.images().list(opts))
    }

    pub fn container_pause(&self, name: &str) -> Result<(), Error> {
        self.execute(self.docker.containers().get(name).pause())
    }

    pub fn container_unpause(&self, name: &str) -> Result<(), Error> {
        self.execute(self.docker.containers().get(name).unpause())
    }

    pub fn container_start(&self, name: &str) -> Result<(), Error> {
        self.execute(self.docker.containers().get(name).start())
    }

    pub fn container_stop(&self, name: &str) -> Result<(), Error> {
        self.execute(self.docker.containers().get(name).stop(None))
    }

    pub fn container_delete(&self, name: &str) -> Result<(), Error> {
        self.execute(self.docker.containers().get(name).delete())
    }
}
