# Releasing matrix-wysiwyg

Normally this would be done by the project owners.

Here are the steps we take:

1. Choose a version number

We use semantic versioning.

It can't be anything that has been pushed to any package repo before.

2. Add a changelog entry

Currently this is stored in platforms/web/CHANGELOG.md.

TODO: store the canonical version in the project root, and copy it to there
so the NPM packaging can find it.

3. Set the version number

Currently this is manual:

* Edit platforms/web/package.json
* Edit bindings/wysiwyg-ffi/Cargo.toml
* Edit bindings/wysiwyg-wasm/Cargo.toml
* Edit crates/wysiwyg/Cargo.toml
* Edit bindings/wysiwyg-wasm/package.json
* Edit platforms/android/gradle.properties
* `git commit`
* `git tag 0.1.0 && git push --tags`
* (For iOS the release script uses the git tag, so nothing to do I think.)

TODO: make a script that sets the git tag and pushes it, and updates the
various files containing the version number. And checks that the changelog
entry has been created.

4. Create the packages

* Web:

    ```bash
    make web
    cd platforms/web
    yarn build
    npm publish --access public
    ```

* TODO: Web: manually launch the
  [github action](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/publish.yml)
  which will package the code and upload it to NPM. It uses the version number
  it finds in package.json, which you updated above.

* Android: TODO

* Swift/iOS:
  Run `./release_ios.sh` which will open a PR against
  [the swift package repo](https://github.com/matrix-org/matrix-wysiwyg-composer-swift)
  with the latest from main

TODO: automate all of this using a github workflow that triggers when we
create a github release.

TODO: update release_io.sh to handle tags/releases
