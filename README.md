# audiobookfinder (adbf)
An example program to learn [Rust](https://www.rust-lang.org/) and meet its concepts by: find audio books on different machines.

### Why Rust?
Rust is an awesome but difficult to learn programming language using different approaches and concepts to solve the current main software development issues for system programming, such as parallelism (what else to do with this multi-core, we are not getting much faster any more), safety and security (let the computer/compiler do what it can do better than a programmer, more quality but also IOT ... I want safe products at home which cannot be turned into zombie devices by buffer overflow), and more high-level approaches which lets you implement more functionality in less code.

As a C++ developer, I know some of the C++11/14/17 enhancements and some don't convince me at all, just look here about the "costs" you have and what it looks like in Simon Brand's ["How Rust gets polymorphism right"](https://www.youtube.com/watch?v=VSlBhAOLtFA).
For a stunning live-coding example of Rust and an example how to use thrussh, which I'd like to implement, look at
[Pierre-Étienne Meunier - Building SSH servers in minutes](https://www.youtube.com/watch?v=TKQoPQcKKTw). He uses heavily the compiler for development which is not recommended (better with a well setup IDE) but it is nice as how someone can use it. The crypto details he gives get a bit lost.

## My first program in Rust
Actually I plan to do something useful. The program collects all information about (yet) audio books on different devices/clients, stores it and then does something with it, like showing stats, finding duplicates, aggregating everything at one place.

### Documentation
Look [here](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html)!

### Goals
The primary goal is to learn Rust and to cover various aspects of the language, of which some of I already used inside the program, such as:
* borrowing
* shared-data over different threads (not yet lifetime optimized)
* multi-threading ([Rayon](https://github.com/rayon-rs/rayon))
* an optional graphical interface that even runs on console only machines (the [Cursive](https://github.com/gyscos/Cursive) TUI)
* architecture (modules), did some rework with file structure but it is not yet perfect in Rust, really. Now the code is better hidden inside a library... this gives some more opportunities
* high-level functionality of different crates / including/using different crates (I don't want to reinvent the wheel)
* Generics: a little bit about and how to use Generics, really nice, a bit difficult to search and fully adapt for but clear in its usage and powerful!!
* in-code documentation with html generation, really nice! (still needs to be published automatically)
* easy command-line (always was looking for that, nice: [clap](https://github.com/kbknapp/clap-rs))
* channel/thread communication (creating worker threads easily, there are plenty implemented yet, no concurrency problems!!)
* high level networking (mDNS): theoretically working, but 1st package depends on avahi ([register](https://github.com/plietar/rust-mdns)), [2nd](https://github.com/dylanmckay/mdns) even [fork](https://github.com/NervosFoundation/rust-mdns-discover) causes heavy CPU-load ...
* use the test feature of Rust (one mod only yet)
* trait example (a simple drop with print message)
* thread-pool: a simple self written but nice to use implemention :blush:
* simple timers, alive signal in TUI (yet a sleep thread for each timer, not perfect)
* easy cross compile (and test) for raspberry (v1 and v2, v3)... ok the tui update needs adjustment, cross compiling is a bit broken because of dependencies
* logging (own module for that)
* CI with [travis](https://travis-ci.org/electricherd/audiobookfinder/) works
* travis built and deployed own public [documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html)

### Changes:
* [documentation](https://electricherd.github.io/audiobookfinder/audiobookfinder/index.html) deployed, awesome: Rust + github + travis +... :sunglasses: (needs javascript enabled)
* applied single test file for travis run: took Bachs Toccata And Fugue In D Minor by Paul Pitman (licence PD)  [orangefreesounds](www.orangefreesounds.com/toccata-and-fugue-in-d-minor/) in rememberring [Monthy Python's grand rugby match](https://www.youtube.com/watch?v=HKv6o7YqHnE).
* travis CI working
* more documentation locally: `cargo doc --no-deps --open`
* fixed ui with BoxView and correct id finding (looks like bug is in Cursive)
* refactored lookup method in net (needs more comments now)
* file logging in (use [glogg](http://glogg.bonnefon.org/))
* updated all external crates
* logging mechanism introduced (`logit.rs`). It was needed because of tui console output was not readable (either syslog or console)
 * run e.g. with `RUST_LOG=adbflib::net=debug RUST_BACKTRACE=full cargo run -- -n ~/Audiobooks`
* ssh client with example key works, key now external
* found emojis :grin:
* included Rust doctest, since it is mostly a library, works well :smiley:
* using a config mod
* new mDNS crate for searching (which is very cpu consuming, but the new one is just a very recent fork, but hoping)
* common.rs for common helper, such as a thread-pool
* client/server authorization/management in a safe way (some small crypto with [thrussh](https://pijul.org/thrussh/))
* in and used but only as example, not yet understood:  [futures](https://tokio.rs/docs/getting-started/futures/) and ([tokio](https://tokio.rs/)) for async behavior and for networking

### ToDo:
* enhance data collection to more than id3 tags, it was difficult to find a nice public domain original mp3 from [wikimedia](https://commons.wikimedia.org/wiki/Main_Page)
* redo tui messages, ctrl messages (maybe into extra mod)
* understand trussh communication, creating key, authorize
* test more different targets using [this](https://github.com/japaric/trust)
* use state machine like [state_machine_future](https://github.com/fitzgen/state_machine_future) for client and server, the example looks promising
* nicer timer (thread pool is good but still with sleep)
* make cross compiling as easy as possible
* get rid of Avahi


### Issues:
* logging from other modules too detailed/too much
* how to decide if an mDNS device is duplicated (more than 1 ipAdress representation, which is correct?, and do they come not within the same record)
* no net is a problem
* bad mDNS search interface to external crate needs a further timeout, even kill a newly created search thread.
* tui update on Raspberry was slow, better find another way


### Yet in plan:
* create a key yourself!! And store, which is going to be done if not found at startup
* Rust workspace for IDE
* communication is now easy with ssh but how to authenticate as a valid adbf? Look at ssh details, and zero-knowledge or something similar: hiding key or secrecy knowledge in code without being to obvious (first should be a simple string, don't bother too much)
* rework the one stub for worker thread to have many worker threads in net to do something with found addresses (use thrussh simple example)
* using an hopefully nice to use state machine for client server *communication* states
* snap linux packaging / online compiler like [Travis](https://docs.travis-ci.com/user/getting-started/) for various target compilation service
* further lifetimes optimizations
* exchange of data over net (probably de-/serialization using [serde](https://docs.serde.rs/serde/))
* still looking for the right IDE
  * sublime text is good and fast, setup was ok, racer etc.
  * [ATOM](https://atom.io/), looks good, no refactoring though, many plug-ins for rust, has README.md syntax
  * [IntelliJ IDEA](https://intellij-rust.github.io/install.html) [download with snaps](https://blog.jetbrains.com/idea/2017/11/install-intellij-idea-with-snaps/), and then Rust plug-in: easy, refactoring, spell-check, nice (but editor ... column select??, close tab??), but looks professional
* internationalization (which is not really supported yet by Rust)
* a good and fast data collection
* maybe a little AI layer on determining audio books duplicates/same author by similar spelling, etc.

## Dependencies
Unfortunately the program now uses mDNS-register with [dns-sd](https://github.com/plietar/rust-dns-sd) depends on Linux on [Avahi](https://www.avahi.org/)
* `libavahi-client-dev` or `libavahi-compat-libdnssd-dev`. It also breaks first the easy cross compilation :confused: - I will see where this ends.
But it works I can see myself with a mDNS scanner, so I can also find other audiobookfinder clients when I do it correctly
* `libsodium`: Since I started to adapt to thrussh I also need libsodium
