# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --bin $CRATE_NAME --target $TARGET
    cross build --bin $CRATE_NAME --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --bin $CRATE_NAME --target $TARGET
    cross test --bin $CRATE_NAME --target $TARGET --release

    cross run --target $TARGET -- testaudio
    cross run --target $TARGET --release -- testaudio
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
