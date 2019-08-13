# audiobookfinder (adbf)

For documentation look at [audiobookfinder](https://github.com/electricherd/audiobookfinder).
License file is also supplied.

## Prequisites
On Ubuntu install via apt-get install:
* `libavahi-client-dev` or `libavahi-compat-libdnssd-dev`
* `libsodium`
* `libtag1`
* `libssl`

If `error while loading shared libraries: libsodium.so.18` occurs, on Ubuntu18 (bionic) it is possible to trick libsodium.so.18 to use installed libsodium.so (resp. libsodium.so.23), via `sudo ln /usr/lib/x86_64-linux-gnu/libsodium.so /usr/lib/x86_64-linux-gnu/libsodium.so.18` - not elegant but yet works!

## Create a key
`ssh-keygen -f ~/.adbf/client_key.priv -N 'adbf' -t ed25519`

## Running the binary
Just run the binary file `./audiobookfinder --help` to see options.
