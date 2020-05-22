# dfiles

dfiles is primarily two things:

* a collection of containerized desktop/GUI applications intended for Linux desktops
* a library designed to facilitate the quick development of new containerized
desktop/GUI applications by abstracting away concerns such as providing
container access to host services like pulseaudio, dbus, X11

## About

Running your desktop application in a container has a number of benefits:

* Applications are isolated from one another as much as possible at the process
level.
* You get to choose what parts of your filesystem are "seen" by each
application.
* Many applications assume only you only ever want to use it in a single
context (login, session state, project data, etc). With `dfiles`-managed
applications you get the opportunity to specify differente runtime profiles
using the `-p`/`--profile` flag to effectively isolate different use cases from
one another.
* Containerized applications are highly portable between not only Linux
distributions, but between different computers and across reinstalls; thanks to
`Cargo` (rust's package manager), installing `dfiles`-based apps is trivially
easy on new computers and (on the roadmap) will eventually allow you to choose
the version of the app installed.
* Some people don't like installing proprietary libraries and applications onto
their main operating system but some use case (Steam, Skype, Zoom, etc) require
it; containerization lets them limit those applications' access to the rest of
the system.

However, there can be some drawbacks:

* Since each application runs in its own Linux container/namespace with its own
distinct filesystem, there will be reduced memory efficiencies from the use of
shared/dynamically-linked libraries.

The target audience, at least during early stages of development, are
technically savvy people who don't mind a few rough edges here and there while
I (and anyone who wants to contribute) work to smooth them out.


## Building & Installing

`dfiles` currently assumes that you have a reasonably up-to-date Rust toolchain
and package manager installed. The [rust-lang book installation
guide](https://doc.rust-lang.org/book/ch01-01-installation.html) is the best
source of information on how to get Rust up and running.


## Usage

### Installing dfiles apps

#### Build a dfiles app

Assuming you have not `cargo install`'d the app directly from crates.io and
assuming that your cargo bin directory is at the front of your `PATH`:

```bash
git clone git@github.com:waynr/dfiles
cd dfiles
cargo install -p firefox
firefox build
```

You should see the same output as you would see building a docker image using
`docker build .`.

#### TODO: Install from crates.io

### Run a dfiles app

After installing an app and building its associated container image:

```
firefox run
```

This will run firefox in the "default" profile. This is not to be confused with the
built-in firefox concept of profiles but relates instead to the host system
directories mounted into the container at run time. To mount a different set of
profile directories, which in the case of firefox results in total isolation of
session data, plugins, browse history, etc:

```bash
firefox run -p a-different-profile
```

For firefox and other applications with a built-in notion of profiles it may
not make as much sense to do this, but for other applications where there is
one assumed set of profile/session data for a given user, you may get more use
out of this.

### Configure a dfiles app

In addition to default behaviors built into applications it is possible to
configure them the application level with customizations
such as volume mounts, cpu/memory limits, etc:

```
firefox config --mount <hostpath>:<containerpath>
firefox config --memory 1024mb
```

Configuration specified in this way will apply to all of the application's
profiles. To limit config settings to a specific profile:

```
firefox config --profile --mount <hostpath>:<containerpath>
```

## Roadmap

* Before open source:
  * [x] Aspect-oriented configuration schema for applications with support for
  profiles.
  * Implement configurable aspects:
    * [x] CPU shares
    * [x] Memory
    * [x] Network mode
    * [x] Locale
    * [ ] CurrentUser
      * should take configurable mode to facilitate choice of userns vs
      entrypoint
      * [ ] Replace buildtime user setup with entrypoint script user setup.
  * [ ] Improve README.
  * [ ] Implement some kind of automated image build and push.
  * [x] Replace all `unwrap` and `Box<dyn Error>` instances with better error
  handling
  * [x] Figure out some kind of data directory approach that isolates dfiles
  application data directories to some kind of dfiles-specific XDG data
  directory. For example, don't let firefox use $HOME/.mozilla/firefox --
  dfiles apps should not mess with application data managed by vanilla installs
  of the same app
  * [ ] Figure out container versioning schema of some kind.
  * [x] Prune current apps of unnecessary aspects (most probably don't need
  SysAdmin or Shm)
* [x] Remove NetworkHost default behavior.
* [ ] `userns-remap` alternative to entrypoint user script.
* [ ] Consider framework for lightweight runtime container setup:
  * generate and inject entrypoint script at runtime
    * simple, easy to understand
    * but could burn through seconds every program startup
  * build user-specific image from prebuilt static base image with
  user-specific, config-dependent dynamic setup steps
    * enables one-time extra build per user/config change
    * probably more complicated to get right

## Similar Projects

### jessfraz/dockerfiles

`dfiles` is strongly influenced by github.com/jessfraz/dockerfiles and this
[blog post](https://blog.jessfraz.com/post/docker-containers-on-the-desktop/).

Significant differences include:

* `dfiles` generates Dockerfiles dynamically based on a combination of
hard-coded and user-configured "Aspects"
  * makes it easy to share code between different applications
* `dfiles` hides implementation details of running containers by generating
`docker run` command lines

### snap

Snap is a distro-agnostic package manager that distributes containerized
applications, similar in principle to the end result produce by `dfiles` but
more well-fleshed out and probably more suitable if all you care about is just
installing a thing and getting it running.

Significant differences include:

* `dfiles` targets Docker as its container runtime and aims to (eventually)
leverage the docker registry ecosystem for distribution of images
* `dfiles` is more developer-oriented for people who like to build their own images
