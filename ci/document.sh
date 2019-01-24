#!bash
# only document on i686 and xenial
if [ $TARGET = x86_64-unknown-linux-gnu ] && [ $UBUNTU_VER = xenial ]; then
   cargo doc --no-deps \
   && echo '<meta http-equiv=refresh content=0;url=${CRATE_NAME}/index.html>' > target/doc/index.html \
   && ghp-import -n target/doc \
   && git push -qf https://${gitHubToken}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
fi
