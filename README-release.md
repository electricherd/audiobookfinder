# audiobookfinder (adbf)

For documentation look at [audiobookfinder](https://github.com/electricherd/audiobookfinder).
License file is also supplied.

## Running the binary
Just run the binary file `audiobookfinder --help` to see all options.

Examples:

command line search

`audiobookfinder <PATH_TO_YOUR_AUDIO_FILES>`

search with web ui

`audiobookfinder -w <PATH_TO_YOUR_AUDIO_FILES>`

search with web ui and client net search

`audiobookfinder -wn <PATH_TO_YOUR_AUDIO_FILES>`

command line search with client net search (background)

`audiobookfinder -n <PATH_TO_YOUR_AUDIO_FILES>`


#### Environment variables:

##### POSIX:
`ADBF_LOG`  = `console`, `system`, `file` (default is `system`)

##### Windows (with Powershell):
`$env:ADBF_LOG`  = `'console'`, `'system'`, `'file'` (default is `system`)

use along with `RUST_LOG`, see at [env-logger](https://docs.rs/env_logger/0.7.1/env_logger/#enabling-logging).
and e.g. `RUST_LOG=adbfbinlib::ctrl=trace` for logging trace of adbfbinlib ctrl module

## Licenses, etc. 3rd party (excluding Rust crates) all [MIT](https://tldrlegal.com/license/mit-license) licenses: 
* https://getbootstrap.com/docs/4.0/about/license/
* https://jquery.org/license/
* https://gist.github.com/ismasan/299789
* https://github.com/tempor1s/bktree-rs
