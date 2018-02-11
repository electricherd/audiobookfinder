# audiobookfinder
An example program to learn [Rust](https://www.rust-lang.org/) and meet its concepts. Find my audiobooks on different machines.
## Why Rust?
Rust is an awesome but difficult to learn programming language using different approaches and concepts to solve the current main software development issues for system programming, such as parallism (what else to do with this multi-core, we are not getting much faster any more), safety and security (let the computer/compiler do what it can do better than a programmer, more quality but also IOT ... I want safe products at home, that cannot be turned into zombie devices by buffer overflow), and more high-level approaches which lets you implement more functionality in less code.

As a C++ developer, I know now why I do use and know some of the C++11/14/17 enhancements but some don't convince me at all, hey, look here about the "costs" you have and what it looks like in Simon Brand's ["How Rust gets polymorphism right"](https://www.youtube.com/watch?v=VSlBhAOLtFA).

## My first program in Rust
Actually I plan to do something even useful. The program would get all information about (yet) audio books on different devices/clients, collect it and then do some actions, like stats, find duplicates, aggregate everything at some place.

The primary goal actually is to learn Rust and to cover various aspects of the language, of which some of I already put inside, such as:
* borrowing
* shared-data (not yet lifetime optimized)
* multi-threading ([Rayon](https://github.com/rayon-rs/rayon))
* an optional graphical interface that even runs on console only machines (the [Cursive](https://github.com/gyscos/Cursive) TUI)
* architecture (modules), did some rework with file structure but it is not yet perfect in Rust, really. Now the code is better hidden inside a library... this gives some more opportunities
* high-level functionality of different crates
* including/using different crates (I don't want to reinvent the wheel)
* in-code documentation (well, small yet)
* easy command-line (always was looking for that, nice: [clap](https://github.com/kbknapp/clap-rs))
* channel/thread communication
* high level networking (mDNS), having issues there since I would like to try 2 mDNS packages simultanously
* use the test feature of Rust (simple case)
* trait example (a simple drop with print message)
* simple timers, alive signal in TUI (yet a sleep thread for each timer, not perfect)
* easy cross compile (and test) for raspberry (v1 and v2, v3)... ok the tui update needs adjustment
* still looking for the right IDE, sublime is good and fast, I am now trying [ATOM](https://atom.io/)




yet in plan:
* nicer timer (until I got tokio timers), at least threadpool
* mDNS network fun
* more traits (serialization, iterator, etc. when it makes sense and fun)
* further lifetimes optimizations
* client/server authorization/management in a safe way (some small crypto with [thrussh](https://pijul.org/thrussh/))
* exchange of data over net (probably searialization using [serde](https://docs.serde.rs/serde/))
* internationalization (which is not really supported yet by Rust)
* a good and fast data collection
* [futures](https://tokio.rs/docs/getting-started/futures/) and ([tokio](https://tokio.rs/)) for async behavior and for networking
