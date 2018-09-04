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

## License

This library is dual licensed under either of the following, at your option:

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)

## Contributors

Pull requests, issues and comments welcome. For pull requests:

* Add tests for new features and bug fixes
* Follow the existing style
* Separate unrelated changes into multiple pull requests
* See the existing issues for things to start contributing.

For bigger changes, make sure you start a discussion first by creating an issue and explaining the intended change.

Atlassian requires contributors to sign a Contributor License Agreement, known as a CLA. This serves as a record stating that the contributor is entitled to contribute the code/documentation/translation to the project and is willing to have it used in distributions and derivative works (or is willing to transfer ownership).

Prior to accepting your contributions we ask that you please follow the appropriate link below to digitally sign the CLA. The Corporate CLA is for those who are contributing as a member of an organization and the individual CLA is for those contributing as an individual.

* [CLA for corporate contributors](https://na2.docusign.net/Member/PowerFormSigning.aspx?PowerFormId=e1c17c66-ca4d-4aab-a953-2c231af4a20b)
* [CLA for individuals](https://na2.docusign.net/Member/PowerFormSigning.aspx?PowerFormId=3f94fbdc-2fbe-46ac-b14c-5d152700ae5d)

## Disclaimer

This is not an official Atlassian product (experimental or otherwise), it is just code that happens to be owned by Atlassian.
