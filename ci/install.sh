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
      x86_64-unknown-linux-gnu)
         sudo apt-get install -qq libavahi-compat-libdnssd-dev -y \
         && sudo add-apt-repository ppa:james-page/0mq -y \
         && sudo apt-get update -qq \
         && sudo apt-get install libsodium-dev -y
#         docker build -t electricherd/adbfimage:0.1.13 ci/docker/x86_64-unknown-linux-gnu
      ;;
      i686-unknown-linux-gnu)
       rustup target install i686-unknown-linux-gnu \
         && sudo apt-get install -qq libavahi-compat-libdnssd-dev -y \
         && sudo add-apt-repository ppa:james-page/0mq -y \
         && sudo apt-get update -qq \
         && sudo apt-get install libsodium-dev -y
      ;;
      armv7-unknown-linux-gnueabihf)
#       rustup target install armv7-unknown-linux-gnueabihf \
#         && sudo apt-get install -qq arm-linux-gnueabihf-gcc \
#         && sudo apt-get install -qq libavahi-compat-libdnssd-dev -y \
#         && sudo add-apt-repository ppa:segf4ult/zeromq -y \
#         && sudo apt-get update -qq \
#         && sudo apt-get install libsodium-dev -y
        docker build -t electricherd/adbfimage:0.1.13 ci/docker/armv7-unknown-linux-gnueabihf
      ;;
      arm-unknown-linux-gnueabi)
       sudo apt-get install -qq libavahi-compat-libdnssd-dev -y
      ;;
      aarch64-unknown-linux-gnu)
       sudo apt-get install -qq libavahi-compat-libdnssd-dev -y \
        && sudo add-apt-repository ppa:chris-lea/libsodium -y \
        && sudo apt-get update -qq \
        && sudo apt-get install libsodium-dev -y
      ;;
      x86_64-apple-darwin)
       ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)" < /dev/null 2> /dev/null \
        && brew install libsodium
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
}
main
