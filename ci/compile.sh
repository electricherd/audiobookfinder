set -ex

main() {
    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-musl
        sort=sort
    else
        target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    # compile
    case $TARGET in
      x86_64-unknown-linux-gnu|i686-unknown-linux-gnu|arm-unknown-linux-gnueabi|aarch64-unknown-linux-gnu|x86_64-apple-darwin)
         cargo build --bin audiobookfinder --target $TARGET --release
      ;;
      armv7-unknown-linux-gnueabihf)
         docker run --volume "$PWD:/home/cross/project" ragnaroek/rust-raspberry:1.43.1
      ;;
    esac
}
main