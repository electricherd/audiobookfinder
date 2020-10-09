# audiobookfinder (adbf)

For documentation also see [audiobookfinder](https://github.com/electricherd/audiobookfinder). The audiobookfinder
is an application to search for audio(book) files, and share the found data along with (local network) connected
devices to (later) identify duplicates, different quality versions, etc.


## Running the binary
Just run the binary file `audiobookfinder --help` to see all options.

##### Recommended:

This opens up your default browser application with the web-ui running, and runs the local network search for other
clients after you re-affirm the paths given by command-line (or change them):

`audiobookfinder -wn <PATH_TO_YOUR_AUDIO_FILES>`


### Further Examples:

command line search

`audiobookfinder <PATH_TO_YOUR_AUDIO_FILES>`

search with web ui

`audiobookfinder -w <PATH_TO_YOUR_AUDIO_FILES>`

search with web ui and client net search

`audiobookfinder -wn <PATH_TO_YOUR_AUDIO_FILES>`

command line search with client net search (background)

`audiobookfinder -n <PATH_TO_YOUR_AUDIO_FILES>`


##### Environment variables:

###### POSIX:
`ADBF_LOG`  = `console`, `system`, `file` (default is `system`)

###### Windows (with Powershell):
`$env:ADBF_LOG`  = `'console'`, `'system'`, `'file'` (default is `system`)

use along with `RUST_LOG`, see at [env-logger](https://docs.rs/env_logger/0.7.1/env_logger/#enabling-logging).
and e.g. `RUST_LOG=adbfbinlib::ctrl=trace` for logging trace of adbfbinlib ctrl module


### Licenses: 
See provided license file LICENCE
