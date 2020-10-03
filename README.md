# audiobookfinder (adbf)
An example program to learn [Rust](https://www.rust-lang.org/) and meet its concepts by: find audio books on
different clients/devices.

![Minimum rustc version](https://img.shields.io/badge/rustc-1.32.0+-green.svg)
[![MIT license](https://img.shields.io/github/license/electricherd/audiobookfinder)](https://lbesson.mit-license.org/)
[![AppVeyor Job branch](https://ci.appveyor.com/api/projects/status/github/electricherd/audiobookfinder?branch=master&svg=true)](https://ci.appveyor.com/project/electricherd/audiobookfinder) <br/>
![shields top language](https://img.shields.io/github/languages/top/electricherd/audiobookfinder)
![shields code size](https://img.shields.io/github/languages/code-size/electricherd/audiobookfinder)
![shields commit date](https://img.shields.io/github/repo-size/electricherd/audiobookfinder) <br/>
[![shields commit date](https://img.shields.io/github/last-commit/electricherd/audiobookfinder/master)](https://github.com/electricherd/audiobookfinder/commits?author=electricherd)
[![shields issues](https://img.shields.io/github/issues/electricherd/audiobookfinder)](https://github.com/electricherd/audiobookfinder/issues)
![shields language count](https://img.shields.io/github/languages/count/electricherd/audiobookfinder)



### Why Rust?
Rust is an awesome but difficult to learn programming language using different approaches and concepts to solve the
current main software development issues for system programming:
 * Secure Programming Concepts: let the computer/compiler do what it can do better than a programmer:
   [safe threading](https://doc.rust-lang.org/book/ch16-00-concurrency.html),
   [error-concepts](https://doc.rust-lang.org/book/ch09-00-error-handling.html), forbid everything non-safe by default
 * Quality: high level language concepts, [easy to embed](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html) and include high quality external packages, which lets you
   implement more functionality in less code
 * [Embedded](https://www.rust-lang.org/what/embedded): easy cross compiling, interfaces to C, becoming better to be stripped down to core system functions for the
   sake of minimum code footprint. [async/await](https://rust-lang.github.io/async-book/) for non OS programs,
   [`no_std`](https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html) and the [`async executors`](https://ferrous-systems.com/blog/async-on-embedded/) (even own ones) will be important.
 * Parallelism and [Concurrency](https://docs.rust-embedded.org/book/concurrency/): what else to do with multi-core cpu, we are not getting much faster anymore, and often
   cpus are idling due to blocking code. With async / await and futures Rust offers with its security features a very
   good way of dealing with it.
 * [Testing](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html) and [Documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html): really nice
   build-in concepts, even [combined](https://doc.rust-lang.org/rust-by-example/testing/doc_testing.html) !!:wink:
 * [Crossplatform](https://doc.rust-lang.org/nightly/rustc/platform-support.html): many good Rust [libraries](https://crates.io) are crossplatform, and building on top of that just works

Especially for IoT: I want secure and thereby safe products at home which cannot be turned into zombie devices by
buffer overflow and injection, always think of what can go wrong, and let the compiler tell you when you do a
common mistake.

As a C++ developer, I know some C++11/14/17 enhancements, and some don't convince me at all, just look here about
the "costs" you have and what it looks like in Simon Brand's
["How Rust gets polymorphism right"](https://www.youtube.com/watch?v=VSlBhAOLtFA).


# Table of Contents
0. [My first program in Rust](#my-first-program-in-rust)
1. [Features](#features)
2. [Documentation](#documentation)
3. [Architecture](#architecture)
4. [Changes](#changes)
5. [Screenshots](#screenshots)
6. [ToDo](#todo)
7. [CI Continuous Integration](#CI)
8. [Goals](#goals)
9. [Dependencies](#dependencies)
10. [Tools](#tools)
11. [Useful links](#useful-links)

## My first program in Rust
I planned to do something useful for myself. The program collects information about audio books on different
devices/clients, stores it and then processes it, e.g. showing stats, finding duplicates, aggregating everything
at one place by a button click.

The task of collecting audio book data can be exchanged with any other task, this basically leads to a local
network agent approach with a libp2p swarm.

There is the [Architectural Design in UML](#architecture).

__It's crossplatform now!__

### Features
* no [unsafe](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html) Rust / no lib dependancies
* crossplatform (Linux/Windows)
* [http-server](https://actix.rs/docs/server/) with [websocket](https://en.wikipedia.org/wiki/WebSocket) communication
   to act as web ui client
* multi-client via [libp2p](https://libp2p.io): [mDNS](https://en.wikipedia.org/wiki/Multicast_DNS),
  [kademlia](https://en.wikipedia.org/wiki/Kademlia) communication over [noise protocol](http://noiseprotocol.org/)
* [tui](https://en.wikipedia.org/wiki/Text-based_user_interface) and [web ui](https://en.wikipedia.org/wiki/Web_application)
* build-in [documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html)
* [appveyor](#CI) and [travis](#CI) CI
* build-in unit-testing the right way
* [BK-tree](https://en.wikipedia.org/wiki/BK-tree) data structure for approximate string matching
* simple mobile app based on [ffi](https://en.wikipedia.org/wiki/Foreign_function_interface) interface [Dart](https://dart.dev/) / [Flutter](https://flutter.dev/) / [flutterust](https://github.com/shekohex/flutterust)

### Documentation

[Documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html) is generated.

It is an inline documentation from [CI](https://travis-ci.org/electricherd/audiobookfinder/) generated documentation - Rust does a nice job here as well!


### Architecture
![Diagram](docs/diag_architecture_general.svg)


### Changes
* since marrying [flutterust](https://github.com/shekohex/flutterust) and old adbflib, package name now is adbflib, the library is now adbfbinlib (yes, but for now)
* added and fixed first shot mobile app mdns feature ... it just works :open_mouth: :blush:
* produced apk can even be design tested in android studio
* had to introduce platform dependent crate compilation, here only [webbrowser crate](https://crates.io/crates/webbrowser) 
* added mobile app build by using [flutterust](https://github.com/shekohex/flutterust) by [Shady Khalifa](https://github.com/shekohex/shekohex) :thumbsup: great
  * [Dart](https://dart.dev/)/[Flutter](https://flutter.dev/) coding required as frontend on mobile app side
  * add github actions as 3rd CI (quite new to me)
  * distinguished lib and binary builds in CI
  * will lead to architectural changes -> parts to: lib, binary, shared
* webui path input dialog working, more results presented
* changed a range based implementation with a regex based implementation, also for the interest of the regex crate in Rust
* moved direct connection to net via ipc out, `net` is the long running slow part, (local) `collection` is the fast part
* added `ADBF_LOG` env variable
* webui now starts with a selection dialog where you can add/change preselected folders/dirs
* defined a trait for tag information, now: id3, mp4, flac but no awesomeness ...
* debugging messages for collection, now collection is a bktree
* webui shows search ongoing on other peers, and then result (number of audio files)
* webui changes, option added for with or without automatic browser opening, bump
* state machine was added, wrapped in a layer of a custom net behaviour in the libp2p swarm ([architecture design](#architecture) updated)
* my unfortunately now unused [observer pattern](../../wiki/Observer-pattern) was added to wiki - a better version of what
 could be found in internet (Rust is very restrictive, some patterns don't work that well).

<details>
  <summary>click for older changes</summary>

    * boostrap, jquery upgraded, some webui animation, better net communication struct, multiaddr on peer per webui tooltip
    * peers in webui can now deregister because of e.g. timeout
    * using libp2p network swarm, replacing single-on mdsn with it, but having same functionality
    * releases for ubuntu, windows, raspberry (20LTS had a upx packing problem due to changed compiler flags, I suppose)
    * fixed webui behavior, now crossplatform (after cursive backend change, taglib replaced by id3)
    * pretty webui design, net messages as good as tui now, fixed thread termination issues to be mostly graceful
    * webui is in sync now, prepare net messages for webui to maybe replace tui
    * back to many threads, but synced and working just fine - webui must be able to replace tui at some time
    * fixed up many older problems, yet ready for libp2p migration for communication over net
    * cleaned up yet inactivated parts: former ssh connection, state machine replacement
    * introducing a nice way to sync threads on startup by creating a channel, send its sender to main thread and block own thread until sender is sent back to self controlled receiver.
    * trying upx in CI builds again
    * migrated first too many native threads to async green threads, as also most dependant external crates use more general futures approach. It's yet a bit confusing and inconsistant but problem were rayon thread iterator and cursive as thread dependant. But I am about to like and understand async/await quite well, also the consequences for embedded development :grin:
    * bumped to Rust 2018 features async/await in net module using futures in few occasion, but will continue with that
    * version changes of different used crates
    * changed this README, to add version changes to *Changes*, re-ordered, and made [Goals](#goals) as a check list, bumped version to v
    * updated various dependency packages from Rust libraries [actix](https://actix.rs/) (there was a dependency lock for a longer time) to [bootstrap](https://getbootstrap.com/), and [jquery](https://jquery.com).
    * added small key generation documentation in `README-release.md`
    * preparing cross compile for banana pi (Dockerfile)
    * added and will later moving from [travis](https://travis-ci.org/) to [AppVeyor](https://www.appveyor.com/) which doesn't lack bionic 18.04 builds and possibility for Windows builds (AppVeyor was primarily used for Windows builds, Travis for Linux in Rust, so documentation for Linux and Rust builds in AppVeyor is quite bad ... testing)
    * cleaned up changes list, switched to [sublime text](https://www.sublimetext.com) instead of [atom](https://atom.io/) as editor
    * trying xargo instead of cargo to compile (possible problems with std in cross compiling, and optimizations), but only works with nightly.
    * cleaned up deployment, added a release readme, licence to deploy as well
    * repaired deployment of binaries, including stripped and packed binaries (upx optimization doesn't work somehow)
    * included (should work fully offline later, all MIT licensed) 3rdparty css, js-scripts ([jquery](https://jquery.com),[bootstrap](https://getbootstrap.com/)) and all pages hard-included in to webserver (no loading of files, yet for development still possible), added state for server, connected websocket, designed a favicon plus logo
    * added basic webui support: http-server with websockets ([actix](https://actix.rs)), a single page application, the page and websockets are already there.
    * added architecture graphics using [draw.io](https://draw.io), which is awesome. Also connectable by [github support](https://about.draw.io/github-support/) directly via [this](https://www.draw.io/?mode=github) ([howTo](https://github.com/jgraph/drawio-github)).
    * state machine not yet used (need to think more about "futures" architecture and understand futures and how to combine)
    * the client ssh connector (com_client) is behind a state machine (to have reconnect and similar easily)
    * replaced [id3](https://github.com/jameshurst/rust-id3) with [taglib](https://github.com/ebassi/taglib-rust/) (more external libs, but many more available media tags). Unfortunately it took me quite some time to find some strange difference (didn't work) between [crates.io](https://crates.io/crates/taglib) and original [github.com](https://github.com/ebassi/taglib-rust/) version, so I had to use the git pull rather than the convenient crate.io dependency usage in `Cargo.toml`.
    * I suspended the usage of [trust](https://github.com/japaric/trust) which uses [cross](https://github.com/japaric/cross), since the develop cross compiling docker images are based on ubuntu 12.04 (deb jessie), and the libsodium, libavahi uses ubuntu ppa from newer versions. I might even go to xenial (deb stretch), then both libs are included by default. But I would have to create my own dockerfile for that, and not just extend the well prepared dockerfiles from cross. :unamused:
    * [documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html) deployed, awesome: Rust + github + travis +... (needs javascript enabled)
    * applied single test file for travis run: took Bach's Toccata And Fugue In D Minor by Paul Pitman (licence PD)  [orangefreesounds](www.orangefreesounds.com/toccata-and-fugue-in-d-minor/) in rememberring [Monthy Python's grand rugby match](https://www.youtube.com/watch?v=HKv6o7YqHnE).
    * travis CI working
    * more documentation locally as html: `cargo doc --no-deps --open`
    * file logging in (use [glogg](http://glogg.bonnefon.org/))
    * logging mechanism introduced (`logit.rs`). It was needed because of tui console output was not readable (either syslog or console)
     * run e.g. with `RUST_LOG=adbfbinlib::net=debug RUST_BACKTRACE=full cargo run -- -n ~/Audiobooks`
    * ssh client with example key works, key now external
    * found emojis :grin:
</details>

### Screenshots
<!-- the files are linked in issue section of https://github.com/electricherd/audiobookfinder/issues/28  -->
| ![Screenshot Linux v0.1.28 Running 1](https://user-images.githubusercontent.com/31503071/93353902-b9f83880-f83c-11ea-9f8d-5054f74a6dbc.png?raw=true) | ![Screenshot Linux v0.1.28 Running 2](https://user-images.githubusercontent.com/31503071/93353903-ba90cf00-f83c-11ea-935d-ebc79127954c.png?raw=true)| ![Screenshot Linux Selection](https://user-images.githubusercontent.com/31503071/91750242-1152a380-ebc3-11ea-8840-dc2576c47785.png?raw=true) | ![Screenshot Linux Running](https://user-images.githubusercontent.com/31503071/91750136-df414180-ebc2-11ea-8508-24c04e000ba9.png?raw=true)|
| :------: | :------: | :------: | :------: |
|  ![Screenshot Windows v0.1.28 Running 1](https://user-images.githubusercontent.com/31503071/93353900-b8c70b80-f83c-11ea-821c-68cc015e4b10.png?raw=true) | ![Screenshot Windows v0.1.28 Running 2](https://user-images.githubusercontent.com/31503071/93353901-b9f83880-f83c-11ea-9558-773279498f87.png?raw=true) | ![Screenshot Windows Selection](https://user-images.githubusercontent.com/31503071/91750262-17488480-ebc3-11ea-97be-54005f012669.png?raw=true) | ![Screenshot Windows Running](https://user-images.githubusercontent.com/31503071/91750257-144d9400-ebc3-11ea-9cb6-93dc4f7225e5.png?raw=true)|
| ![Screenshot Android App 0.0.3 Device](https://user-images.githubusercontent.com/31503071/94609005-85559980-029e-11eb-916a-4e195b932f29.jpg?raw=true) | ![Screenshot Android App v0.0.3 device](https://user-images.githubusercontent.com/31503071/94609007-85ee3000-029e-11eb-8869-03ae0affbf3b.jpg?raw=true)|  ![Screenshot Android App 0.0.3 Simulation](https://user-images.githubusercontent.com/31503071/94597515-f3de2b80-028d-11eb-9ac3-8628fe8b56b2.png?raw=true) | ![Screenshot Android App v0.0.3 device](https://user-images.githubusercontent.com/31503071/94597513-f3459500-028d-11eb-8cd8-f378eeff6e63.jpg?raw=true) |


### ToDo
* fix library documentation on travis, only binary now (due to other libs) 
* fix github actions CI for automatic build
* try [crate vfs](https://github.com/manuel-woelker/rust-vfs) for unit test with files!! interesting and needed!
* look for other tag libraries (e.g. symphonia-metadata [symphonia](https://github.com/pdeljanov/Symphonia))
* a good and fast data collection with few more further lifetimes optimizations
* add a nice way to collect data (string distance + time, maybe 2nd duration hash set?, filter empty tags)
* add memory consumption monitoring for collection - started for BKTree
* look into state and extra data usage of already used [smlang-rs](https://github.com/korken89/smlang-rs/blob/master/examples/event_with_reference_data.rs)
* think of a protocol what adbf clients agree on and exchange (e.g. still searching, files found, etc)
* maybe a little AI layer on determining audio books duplicates/same author by similar spelling, etc.
* internationalization (which is not really supported yet by Rust)
* ~~implement as android/ios app using [flutterust](https://github.com/electricherd/flutterust).~~
* ~~change webui to be started without collection start, to be able use path selection from within webui later~~
* ~~add [ctrlc](http://detegr.github.io/doc/ctrlc/) functionality for signal handling in main~~
* ~~let state machine *talk* (as ipc) with data collection via [crossbeam](https://github.com/crossbeam-rs/crossbeam) (first only the finish search status)~~
* ~~make div from html page to extra single file for later multiple clients on one page~~

### CI
The Continuous Integration is done on 2 services, Travis and AppVeyor but will probably once completely moved to AppVeyor because Travis recently only had old LTS 16.04 images, and no possible Windows compilation (they are working on it), so there is:
* On Appveyor [![common appveyor](https://ci.appveyor.com/api/projects/status/github/electricherd/audiobookfinder?branch=master&svg=true)](https://ci.appveyor.com/project/electricherd/audiobookfinder)
    * [Audiobookfinder Build Windows](https://ci.appveyor.com/project/electricherd/audiobookfinder/) ![appveyor](https://appveyor-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branch/master/2)
    * [Audiobookfinder Build Ubuntu 20.04](https://ci.appveyor.com/project/electricherd/audiobookfinder/) ![appveyor](https://appveyor-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branch/master/1)
    * [Audiobookfinder Build Ubuntu 18.04](https://ci.appveyor.com/project/electricherd/audiobookfinder/) ![appveyor](https://appveyor-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branch/master/3)
* On Travis
    * [Audiobookfinder Build Ubuntu 16.04](https://travis-ci.org/electricherd/audiobookfinder) ![travis](https://travis-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branches/master/1)
    * [Audiobookfinder Build Debian Buster Raspberry](https://travis-ci.org/electricherd/audiobookfinder) ![travis](https://travis-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branches/master/2)
    * only until version 0.1.18: Build Linux 14.04
* On Github with Github Actions
    * Android-Build: [![Actions Status](https://github.com/electricherd/audiobookfinder/workflows/Android%20Build/badge.svg)](https://github.com/electricherd/audiobookfinder/actions)

### Goals
The primary goal is to learn Rust and to cover various aspects of the language, of which some of I already used inside the program, such as:
- [x] borrowing: the borrow checker, some issues but I am fine with it now
- [ ] async/await: almost there
- [ ] easy cross compile (and test) for raspberry (v1 and v2, v3)... ok the tui update needs adjustment
- [x] have the Rust frontend/backend as IOS and/or Android app, with a small glue code (because beside the tui it's a html5 webapp frontend). [WASM](https://www.rust-lang.org/what/wasm) is not reachable since it uses `no_std`
- [x] shared-data over different threads (not yet lifetime optimized)
- [x] multi-threading, a lot of threads and communication is inside, also  ([Rayon](https://github.com/rayon-rs/rayon))
- [x] learning [futures](https://en.wikipedia.org/wiki/Futures_and_promises)
- [x] an optional graphical interface that even runs on console only machines (the [Cursive](https://github.com/gyscos/Cursive) TUI)
- [x] webui, modern and nice with [actix](https://actix.rs/), [bootstrap](https://getbootstrap.com/), and [jquery](https://jquery.com)
- [x] [architecture](#architecture) (modules), did some rework with file structure but it is not yet perfect in Rust, really. Now the code is better hidden inside a library... this gives some more opportunities
- [x] high-level functionality of different crates / including/using different crates (I don't want to reinvent the wheel, and yes, that is very nice)
- [x] in-code documentation with html generation, really nice!
- [x] easy command-line (always was looking for that, nice: [clap](https://github.com/kbknapp/clap-rs))
- [x] channel/thread communication control with [crossbeam](https://github.com/crossbeam-rs/crossbeam) *Waitgroup* instead of std barrier 
- [x] high level networking, client/server authorization/management from libp2p2 (mdns, swarm, noise-protocol transport layer) 
- [x] use the test feature of Rust: that is just very, very nice, even an usage example can be done as a running test!!
- [ ] traits: getting better with unfortunately unused [observer pattern](../../wiki/Observer-pattern) 
- [x] thread-pool: a simple self written but nice to use implemention :blush: but not needed any more
- [x] simple timers: inside async: super easy
- [x] logging (own module for that), good
- [x] CI with [travis](https://travis-ci.org/electricherd/audiobookfinder/) works, cross compiling is still difficult with [trust](https://github.com/japaric/trust), [cross](https://github.com/japaric/cross/), [docker](https://www.docker.com/), need to watch closely to [steed](https://github.com/japaric/steed) for some problem solving.
- [x] travis automatically built and automatically deployed own public [documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html)
- [x] making a library ([adbfbinlib](https://electricherd.github.io/audiobookfinder/adbfbinlib/index.html) as the main part of the program)
- [x] using a Boost-SML style [state machine](https://github.com/korken89/smlang-rs) now, nice one!
- [ ] learning and understanding rust macros (some day)
- [x] exchange of data over all kinds of boundaries (net, thread) via de-/serialization using [serde](https://docs.serde.rs/serde/) and its json feature for webui
- [x] check regular expressions in Rust: it's badly documented, not intuitive

### Dependencies
* no non-Rust libraries, it's crossplatform now :blush:

### Tools
* Editors:
  * [IntelliJ IDEA](https://intellij-rust.github.io/install.html), and then Rust plug-in: easy, refactoring, spell-check, nice, first choice now, because of easy type look-up, and other good features
  * [sublime text](https://www.sublimetext.com) is good and fast, setup was ok, using it now, works very well
  * [atom](https://atom.io/) was for a long time my choice for development, on my Eee Pc [sublime](https://www.sublimetext.com), because of small footprint and performance, but now that is too slow though I really like the Git feature of it, has README.md syntax
* Logging:
  * [glogg](https://glogg.bonnefon.org/) a good logger on linux - since log has a coloring problem it still works pretty good
* Git:
  * [gitahead](https://gitahead.github.io/gitahead.com/) I like that

### Useful links
* https://jsfiddle.net/boilerplate/jquery : for people who don't really like js but need it:
* https://thoughtbot.com/blog/json-event-based-convention-websockets : websockets to js commands
* https://github.com/Ragnaroek/rust-on-raspberry-docker : headachefree compiling for raspberry pi locally with docker
* https://learning-rust.github.io/docs/a5.comments_and_documenting_the_code.html howto documentation

### 3rd party (excluding Rust crates), all [MIT](https://tldrlegal.com/license/mit-license) licenses
* <img src="https://upload.wikimedia.org/wikipedia/commons/c/ce/Unofficial_JavaScript_logo.svg" align="middle" width="21em"/>https://getbootstrap.com/docs/4.3/getting-started/introduction/
* <img src="https://upload.wikimedia.org/wikipedia/commons/c/ce/Unofficial_JavaScript_logo.svg" align="middle" width="21em"/>https://jquery.com/
* <img src="https://upload.wikimedia.org/wikipedia/commons/c/ce/Unofficial_JavaScript_logo.svg" align="middle" width="21em"/>https://gist.github.com/ismasan/299789
* <img src="https://www.rust-lang.org/logos/rust-logo-blk.svg" align="middle" width="21em"/>https://github.com/tempor1s/bktree-rs
* <img src="https://www.vectorlogo.zone/logos/flutterio/flutterio-icon.svg" align="middle" width="21em"/><img src="https://www.rust-lang.org/logos/rust-logo-blk.svg" align="middle" width="21em"/><img src="https://upload.wikimedia.org/wikipedia/commons/thumb/7/7e/Dart-logo.png/240px-Dart-logo.png" align="middle" width="21em"/>https://github.com/shekohex/flutterust
* <img src="https://www.vectorlogo.zone/logos/flutterio/flutterio-icon.svg" align="middle" width="21em"/>https://pub.dev/packages/file_picker