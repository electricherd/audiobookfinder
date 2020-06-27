# audiobookfinder (adbf)
An example program to learn [Rust](https://www.rust-lang.org/) and meet its concepts by: find audio books on different machines.

### Why Rust?
Rust is an awesome but difficult to learn programming language using different approaches and concepts to solve the current main software development issues for system programming:
 * Secure Programming Concepts: let the computer/compiler do what it can do better than a programmer: safe threading, error-concepts, forbid everything non-safe by default
 * Quality: high level language concepts, easy to embed and include high quality external packages, which lets you implement more functionality in less code
 * Embedded: easy cross compiling, interfaces to C, becoming better to be stripped down to core system functions for the sake of minimum code footprint. Async/Await for non OS programs, `no_std` and the `async executors` (even own ones) will be important. 
 * Parallelism and Concurrency: what else to do with multi-core cpu, we are not getting much faster anymore, and often cpus are idling due to blocking code. With async / await and futures Rust offers with its security features a very good way of dealing with it. 
 * Testing and Documentation: some build-in concepts

Especially for IoT: I want secure and thereby safe products at home which cannot be turned into zombie devices by buffer overflow and injection, always think of what can go wrong, and let the compiler tell you when you do a common mistake.

As a C++ developer, I know some C++11/14/17 enhancements, and some don't convince me at all, just look here about the "costs" you have and what it looks like in Simon Brand's ["How Rust gets polymorphism right"](https://www.youtube.com/watch?v=VSlBhAOLtFA).


# Table of Contents
0. [My first program in Rust](#my-first-program-in-rust)
1. [Documentation](#documentation)
2. [Changes](#changes)
3. [ToDo](#todo)
4. [Architecture](#architecture)
5. [CI Continuous Integration](#CI)
6. [Goals](#goals)
7. [Dependencies](#dependencies)
8. [Issues](#issues)
9. [Yet in plan](#yet-in-plan)
10. [Useful links](#useful-links)

## My first program in Rust
I planned to do something useful for myself. The program collects information about audio books on different devices/clients, stores it and then processes it, e.g. showing stats, finding duplicates, aggregating everything at one place by a button click.
The task of collecting audio book data can be exchanged with any other task, this basically leads to a local network agent approach with zero-config.

There is an [Architectural Design in UML](#architecture) to understand a bit beforehand, and see some goals which have not been accomplished.

So far only the state charts and their connection is not done but the general communication/lookup works, collecting some data as well.


### Documentation
It is an inline [CI](https://travis-ci.org/electricherd/audiobookfinder/) generated documentation which can be found [here](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html)! Rust does a nice job here as well!

### Changes
* webui gets paths by websocket json, synchronising browser opening with program is a little issue, I don't like javascript but hey ... it has `let` for variables  
* webui now is connected, event-loop yet strangly implemented, working on json and jquery/javascript
* working on Actor, Arbiter, Websockets in actix ... passing updates to web ui
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
 * run e.g. with `RUST_LOG=adbflib::net=debug RUST_BACKTRACE=full cargo run -- -n ~/Audiobooks`
* ssh client with example key works, key now external
* found emojis :grin:

### ToDo
* kick ping-pong from webui - websockets don't need it, I suppose, fix boostrap and html issues
* fixing webui to show all updates and infos as libp2p network preparation 
* libp2p changes
    + uuid not needed any more: identification is done via a hash from public key, so called peer id
    + change from old [state_machine_future](https://docs.rs/state_machine_future/) to [smlang](https://crates.io/crates/smlang) state machine
* change/replace all net functionality to [libp2p](https://crates.parity.io/libp2p/index.html)
    + thrussh ssh server/client communication
    + mDNS from [mDNS](https://crates.parity.io/libp2p/mdns/index.html) 
* with new feature of [alternative cargo registries](https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html), ready-made libraries and crates, like [parity](https://crates.parity.io/) with [libp2p](https://crates.parity.io/libp2p/index.html), in a first step [mDNS](https://crates.parity.io/libp2p/mdns/index.html) gets interesting
* ~~looking into crates like [rust_sodium](https://crates.io/crates/rust_sodium), which might simplify cross compiling~~ replace with libp2p
* ~~further client thrussh improvements, add secure identification by using zero-knowledge~~ replace with libp2p
* ~~understand trussh communication, creating key, authorize~~ replace with libp2p
* ~~test more different targets using 
for client and server~~ hopes on libp2p
* nicer timer (thread pool is good but still with sleep)
* make cross compiling as easy as possible
* ~~get rid of Avahi~~ hopefully libp2p covers it

### Architecture
![Diagram](diag_architecture_general.svg)
(still early version of drawing, and directly [editable](https://www.draw.io/?mode=github))

### CI
The Continuous Integration is done on 2 services, Travis and AppVeyor but will probably once completely moved to AppVeyor because Travis recently only had old LTS 16.04 images, and no possible Windows compilation (they are working on it), so there is:
* On Appveyor
    * [Audiobookfinder Build Linux 18.04](https://ci.appveyor.com/project/electricherd/audiobookfinder/) ![appveyor](https://ci.appveyor.com/api/projects/status/jkpo45oocs8tmmo3?svg=true)
* On Travis
    * [Audiobookfinder Build Linux 16.04](https://travis-ci.org/electricherd/audiobookfinder) ![travis](https://travis-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branches/master/2)
    * only until version 0.1.18: [Audiobookfinder Build Linux 14.04](https://travis-ci.org/electricherd/audiobookfinder) ![travis](https://travis-matrix-badges.herokuapp.com/repos/electricherd/audiobookfinder/branches/master/1)


Let's see where also cross compiling for embedded will lead ... (a stub and not working Windows compile is already in AppVeyor but cannot yet be compiled due to the usage of crate `Cursive` which uses crate `Termion` which doesn't work on Windows, only when cursive is taken out it will be possible).

### Goals
The primary goal is to learn Rust and to cover various aspects of the language, of which some of I already used inside the program, such as:
- [x] borrowing: the borrow checker, some issues but I am fine with it now
- [x] shared-data over different threads (not yet lifetime optimized)
- [x] multi-threading, a lot of threads and communication is inside, also  ([Rayon](https://github.com/rayon-rs/rayon))
- [x] learning [futures](https://en.wikipedia.org/wiki/Futures_and_promises)
- [x] an optional graphical interface that even runs on console only machines (the [Cursive](https://github.com/gyscos/Cursive) TUI), but probably better...
- [ ] webui, modern and nice with [actix](https://actix.rs/), [bootstrap](https://getbootstrap.com/), and [jquery](https://jquery.com) - but this is only alpha yet
- [x] [architecture](#architecture) (modules), did some rework with file structure but it is not yet perfect in Rust, really. Now the code is better hidden inside a library... this gives some more opportunities
- [x] high-level functionality of different crates / including/using different crates (I don't want to reinvent the wheel, and yes, that is very nice)
- [x] in-code documentation with html generation, really nice!
- [x] easy command-line (always was looking for that, nice: [clap](https://github.com/kbknapp/clap-rs))
- [x]  channel/thread communication (creating worker threads easily, there are plenty implemented yet, no concurrency problems!!) - I implemented own [futures](https://tokio.rs/docs/getting-started/futures/) in a pure inefficient way ... I need to replace them with [futures pattern](https://en.wikipedia.org/wiki/Futures_and_promises) from Tokio or the planned build-in async IO in Rust
- [x] high level networking (mDNS): theoretically working, but 1st package depends on avahi ([register](https://github.com/plietar/rust-mdns)), [2nd](https://github.com/dylanmckay/mdns) even [fork](https://github.com/NervosFoundation/rust-mdns-discover) causes heavy CPU-load ...
- [ ] use the test feature of Rust (one mod only yet), also with example test being tested! - it's in but very few and in the *to-be-removed* modules cursive aka tui
- [ ] traits (first a simple drop with print message), but then more, need to be more comfortable with debug for formatting - not really defined an own trait but needed to write little trait implementations
- [x] thread-pool: a simple self written but nice to use implemention :blush:
- [x] simple timers, alive signal in TUI (yet a sleep thread for each timer, not perfect but since thread-pool quite ok) - should be taken care of by futures in the future or even a small futures timer implementation
- [ ] before multiple c-library dependency: easy cross compile (and test) for raspberry (v1 and v2, v3)... ok the tui update needs adjustment
- [x] logging (own module for that), good
- [x] client/server authorization/management in a safe way (some small crypto with [thrussh](https://pijul.org/thrussh/)), looking at [Pierre-Étienne Meunier - Building SSH servers in minutes](https://www.youtube.com/watch?v=TKQoPQcKKTw) - maybe switching after some time to [Parity's libp2p](https://crates.parity.io/libp2p/index.html)
 cross compiling in general, mix of local (with and without docker) and remote compilation (travis with [xargo](https://github.com/japaric/xargo), [cross](https://github.com/japaric/cross/))
- [x] CI with [travis](https://travis-ci.org/electricherd/audiobookfinder/) works, cross compiling is still difficult with [trust](https://github.com/japaric/trust), [cross](https://github.com/japaric/cross/), [docker](https://www.docker.com/), need to watch closely to [steed](https://github.com/japaric/steed) for some problem solving.
- [x] travis automatically built and automatically deployed own public [documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html)
- [x] making a library ([adbflib](https://electricherd.github.io/audiobookfinder/adbflib/index.html) as the main part of the program)
- [ ] using a [state machine](https://github.com/fitzgen/state_machine_future) where it fits, here for client server *communication* states - inside but yet to really be used (and to decide when to use smaller *futures*)
- [ ] learning and understanding rust macros (some day)

### Dependencies
* `libtag1-dev` and `libtagc0-dev` for libtag
* `libssl-dev` as a clean setup might not have it


### Issues
* AppVeyor deployment is stuck, it builds but the deployment by git tags is not well documented, and different to Travis.
* versioning of AppVeyor and Git tag, plus own versioning
* cross compiling for windows is almost impossible yet, though travis now can support, even in a local VM it is very difficult. OpenSSL, Libsodium are possible but since mDNS or bonjour is used, this looks frustratingly impossible
* logging from other modules too detailed/too much
* how to decide if an mDNS device is duplicated (more than 1 ipAdress representation, which is correct?, and do they come not within the same record)
* no net is a problem
* bad mDNS search interface to external crate needs a further timeout, even kill a newly created search thread.
* tui update on Raspberry was slow, better find another way


### Yet in plan
* trying first new feature of adding non-crates.io crates by looking into [parity](https://www.parity.io/) crates, especially [mdns](https://crates.parity.io/libp2p/mdns/index.html).
* want to get rid of mDNS and sd_dns (due to cross compiling problems, etc.) . Maybe this [multicasting](https://bluejekyll.github.io/blog/rust/2018/03/18/multicasting-in-rust.html) and this [rust_dns](https://bluejekyll.github.io/blog/rust/2018/03/18/multicasting-in-rust.html) crate helps
* create a key yourself!! And store, which is going to be done if not found at startup
* communication is now easy with ssh but how to authenticate as a valid adbf? Look at ssh details, and zero-knowledge or something similar: hiding key or secrecy knowledge in code without being to obvious (first should be a simple string, don't bother too much by:\
`ssh-keygen -f ~/.adbf/client_key.priv -N 'adbf' -t ed25519`)
* rework the one stub for worker thread to have many worker threads in net to do something with found addresses (use thrussh simple example)
* further lifetimes optimizations
* exchange of data over net (probably de-/serialization using [serde](https://docs.serde.rs/serde/)) - for sure
* Editors:
  * [IntelliJ IDEA](https://intellij-rust.github.io/install.html) [download with snaps](https://blog.jetbrains.com/idea/2017/11/install-intellij-idea-with-snaps/), and then Rust plug-in: easy, refactoring, spell-check, nice, first choice now, because of easy type look-up, and other good features 
  * [sublime text](https://www.sublimetext.com) is good and fast, setup was ok, using it now, works very well
  * [atom](https://atom.io/) was for a long time my choice for development, on my Eee Pc [sublime](https://www.sublimetext.com), because of small footprint and performance, but now that is too slow though I really like the Git feature of it, has README.md syntax
* internationalization (which is not really supported yet by Rust)
* a good and fast data collection
* maybe a little AI layer on determining audio books duplicates/same author by similar spelling, etc.

### Useful links
* https://jsfiddle.net/boilerplate/jquery : for people who don't really like js but need it:
* https://thoughtbot.com/blog/json-event-based-convention-websockets : websockets to js commands

### 3rd party (excluding Rust crates)
* https://getbootstrap.com/docs/4.3/getting-started/introduction/
* https://jquery.com/
* https://gist.github.com/ismasan/299789