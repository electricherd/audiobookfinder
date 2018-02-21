# audiobookfinder
An example program to learn [Rust](https://www.rust-lang.org/) and meet its concepts by: find audiobooks on different machines.
## Why Rust?
Rust is an awesome but difficult to learn programming language using different approaches and concepts to solve the current main software development issues for system programming, such as parallelism (what else to do with this multi-core, we are not getting much faster any more), safety and security (let the computer/compiler do what it can do better than a programmer, more quality but also IOT ... I want safe products at home which cannot be turned into zombie devices by buffer overflow), and more high-level approaches which lets you implement more functionality in less code.

As a C++ developer, I know some of the C++11/14/17 enhancements and some don't convince me at all, just look here about the "costs" you have and what it looks like in Simon Brand's ["How Rust gets polymorphism right"](https://www.youtube.com/watch?v=VSlBhAOLtFA).
For a stunning live-coding example of Rust and an example of how to use thrussh, which I'd like to implement, look at
[Pierre-Étienne Meunier - Building SSH servers in minutes](https://www.youtube.com/watch?v=TKQoPQcKKTw), this is simply awesome.

## Dependencies
Unfortunately the program now uses mDNS-register with [dns-sd](https://github.com/plietar/rust-dns-sd) depends on Linux on [Avahi](https://www.avahi.org/)
* libavahi-client-dev or libavahi-compat-libdnssd-dev. It also breaks first the easy cross compilation :-( - I will see where this ends.
But it works I can see myself with a mDNS scanner, so I can also find other audiobookfinder clients when I do it correctly
* libsodium: Since I started to adapt to thrussh I also need libsodium

## My first program in Rust
Actually I plan to do something useful. The program collects all information about (yet) audio books on different devices/clients, stores it and then does something with it, like showing stats, finding duplicates, aggregating everything at one place.

The primary goal is to learn Rust and to cover various aspects of the language, of which some of I already used inside the program, such as:
* borrowing
* shared-data over different threads (not yet lifetime optimized)
* multi-threading ([Rayon](https://github.com/rayon-rs/rayon))
* an optional graphical interface that even runs on console only machines (the [Cursive](https://github.com/gyscos/Cursive) TUI)
* architecture (modules), did some rework with file structure but it is not yet perfect in Rust, really. Now the code is better hidden inside a library... this gives some more opportunities
* high-level functionality of different crates / including/using different crates (I don't want to reinvent the wheel)
* Generics: a little bit about and how to use Generics, really nice, a bit difficult to search and fully adapt for but clear in its usage and powerful!!
* in-code documentation (well, few yet)
* easy command-line (always was looking for that, nice: [clap](https://github.com/kbknapp/clap-rs))
* channel/thread communication (creating worker threads easily, there are plenty implemented yet, no concurrency problems!!)
* high level networking (mDNS): theoretically working, but 1st package depends on avahi (register), 2nd causes heavy CPU-load ...
* use the test feature of Rust (simple case)
* trait example (a simple drop with print message)
* simple timers, alive signal in TUI (yet a sleep thread for each timer, not perfect)
* easy cross compile (and test) for raspberry (v1 and v2, v3)... ok the tui update needs adjustment
* still looking for the right IDE
  * sublime text is good and fast, setup was ok, racer etc.
  * [ATOM](https://atom.io/), looks good, no refactoring though, many plug-ins for rust, has README.md syntax :-)
  * [IntelliJ IDEA](https://intellij-rust.github.io/install.html) [download with snaps](https://blog.jetbrains.com/idea/2017/11/install-intellij-idea-with-snaps/), and then Rust plug-in: easy, refactoring, spell-check, nice (but editor ... column select??, close tab??), but looks professional

Changes:
* included Rust doctest - though problems, since in contrary of tests pub must be put to access former private mods or functions :-(
* using a config mod
* new mDNS crate for searching (which is very cpu consuming, but this is just a very recent fork, but hoping)

Issues:
* maybe get rid of DNSService as member and just keep the thread, then the bad thread handle in mdns lookup might work
* no net is a problems
* get rid of Avahi
* nicer timer (until I got tokio timers), at least thread-pool with joinhandle

yet in plan:
* rework the one stub for worker thread to have many worker threads in net to do something with found addresses (use thrussh simple example)
* started: client/server authorization/management in a safe way (some small crypto with [thrussh](https://pijul.org/thrussh/))
* comes with thrussh: [futures](https://tokio.rs/docs/getting-started/futures/) and ([tokio](https://tokio.rs/)) for async behavior and for networking
* further lifetimes optimizations
* exchange of data over net (probably de-/serialization using [serde](https://docs.serde.rs/serde/))
* internationalization (which is not really supported yet by Rust)
* a good and fast data collection
* maybe a little AI layer on determining audio books duplicates/same author by similar spelling, etc.
