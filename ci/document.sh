#!bash
# only document on i686 and xenial and stable
if [ $TARGET = x86_64-unknown-linux-gnu ] && [ $UBUNTU_VER = LTS_16.04 ] && [ $TRAVIS_RUST_VERSION = stable ]; then
   echo "Deploying documentation of '${CRATE_NAME}'!!"
   cargo doc --package $PACKAGE_NAME --no-deps --document-private-items \
   && echo "<meta http-equiv=refresh content=0;url=${CRATE_NAME}/index.html>" > target/doc/index.html \
   && ghp-import -n target/doc \
   && git push -qf https://${gitHubToken}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
fi
