# audiobookfinder
An example program to learn [Rust](https://www.rust-lang.org/) and meet its concepts. Find my audiobooks on different machines.
## Why Rust?
Rust is an awesome but difficult to learn programming language using different approaches and concepts to solve the current main software development issues for system programming, such as parallism (what else to do with this multi-core, we are not getting much faster any more), safety (let the computer/compiler do what it can do better than a programmer, more quality but also IOT ... I want safe products at home, that cannot be turned into zombie devices by buffer overflow), and more high-level approaches which lets you implement more functionality in less code.

## My first program in Rust
Actually I plan to do something even useful. The program would get all information about (yet) audiobooks on different devices/clients, collect it and then do some actions, like stats, find duplicates, aggregate everything at some place.

The primary goal actually is to learn Rust and to cover various aspects of the language, of which some of I already put inside, such as:
* borrowing
* shared-data (not yet lifetime optimized)
* multi-threading ([Rayon](https://github.com/rayon-rs/rayon))
* an optional graphical interface that even runs on console only machines (the [Cursive](https://github.com/gyscos/Cursive) TUI)
* architecture (modules), also these includes more high-level functionality of different crates
* including/using different crates (I don't want to reinvent the wheel)
* in-code documentation (well, small yet)
* easy command-line (always was looking for that, nice: [clap](https://github.com/kbknapp/clap-rs))
* high level networking (mDNS), having issues there since I would like to try 2 mDNS packages simultanously 

yet in plan:
* simple timers (using an alive signal in TUI)
* further lifetimes optimizations
* client/server authorization/management in a safe way (some small crypto)
* exchange of data over net
* internationalization (which is not really supported yet by Rust)
* a good and fast data collection

