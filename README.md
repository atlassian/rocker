# Docker + Rust = Rocker \m/

Rocker is a text-mode UI to manage your docker containers and images. It communicates with the docker container via a local UNIX socket.

## Installing rocker

To compile and install rocker you need a working Rust environment. You can use [Rustup](https://rustup.rs/) to install Rust on your system. Rocker should compile fine on stable or nightly.

Once you've cloned this repository, simply type `cargo install` (or `cargo install -f` if you've previously installed it). You can then type `rkr` to start it.

## Current features

At the moment, rocker supports:
* Viewing containers, both running and stopped
* Pausing / unpausing containers
* Viewing logs for a running container
* View details of a container
* View image list
* View docker daemon info

## TODO
* Lots!
* Add missing features (start, stop, kill, restart containers, pull image, view image details...)
* Make column sizes more dynamic based on terminal size
* Improve error management
* Add proper build pipeline to build static binaries for Linux and MacOS
* Make key bindings configurable
* Connect to remote Docker servers? Is that useful?
* Package for Linux distros and homebrew
* Publish on crates.io
* Probably lots more...
