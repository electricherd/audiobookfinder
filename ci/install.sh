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

    # install additional libraries
    #
    # libavahi-compat-libdnssd-dev
    #   - debian: is not in standard installation but in deb repo
    #   - macos:  not foundable
    # libsodium-dev
    #   - debian:
    #      - i686: it is only after trusty in repo, but PPA exists
    #      - armhf: is not in trusty, but PPA exists
    #   - macos:  need to be tested/see below
    #
    #
    # is jessie libnss-mdns-dev
    case $TARGET in
      armv7-unknown-linux-gnueabihf)
         sudo apt-get update -qq \
         && docker pull ragnaroek/rust-raspberry:1.43.1
      ;;
    esac

    # Builds for iOS are done on OSX, but require the specific target to be
    # installed.
    case $TARGET in
        aarch64-apple-ios)
            rustup target install aarch64-apple-ios
            ;;
        armv7-apple-ios)
            rustup target install armv7-apple-ios
            ;;
        armv7s-apple-ios)
            rustup target install armv7s-apple-ios
            ;;
        i386-apple-ios)
            rustup target install i386-apple-ios
            ;;
        x86_64-apple-ios)
            rustup target install x86_64-apple-ios
            ;;
    esac

    if [ $TRAVIS_RUST_VERSION = nightly ]; then
       echo "needed for xargo: rustup component add rust-src"
       # needed for xargo on nightly
       rustup component add rust-src
    fi

    # install xargo before / later inside ci/install.sh
    # force should be replaced by a check but travis is mysterious
    case $TARGET in
      x86_64-unknown-linux-gnu|i686-unknown-linux-gnu|arm-unknown-linux-gnueabi|aarch64-unknown-linux-gnu|x86_64-apple-darwin)
        # This fetches latest stable release
        local tag=$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross \
                           | cut -d/ -f3 \
                           | grep -E '^v[0.1.0-9.]+$' \
                           | $sort --version-sort \
                           | tail -n1)
        curl -LSfs https://japaric.github.io/trust/install.sh | \
            sh -s -- \
               --force \
               --git japaric/cross \
               --tag $tag \
               --target $target

        cargo install xargo --force
      ;;
      armv7-unknown-linux-gnueabihf)
        # don't do anything
      ;;
    esac
}
main
