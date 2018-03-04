#!bash
# only document on i686
if [ $TARGET = i686-unknown-linux-gnu ]; then
   cargo doc --no-deps \
   && echo '<meta http-equiv=refresh content=0;url=${CRATE_NAME}/index.html>' > target/doc/index.html \
   &&  ghp-import -n target/doc \
   &&  git push -qf https://${gitHubToken}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
fi
