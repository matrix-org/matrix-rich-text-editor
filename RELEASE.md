# Releasing matrix-wysiwyg

Normally this would be done by the project owners.

Here are the steps we take:

## 1. Choose a version number

We use semantic versioning.

It can't be anything that has been pushed to any package repo before.

## 2. Add a changelog entry

Currently this is stored in platforms/web/CHANGELOG.md.

TODO: store the canonical version in the project root, and copy it to there
so the NPM packaging can find it.

## 3. Set the version number

Currently this is manual:

* Edit platforms/web/package.json
* Edit bindings/wysiwyg-ffi/Cargo.toml
* Edit bindings/wysiwyg-wasm/Cargo.toml
* Edit crates/wysiwyg/Cargo.toml
* Edit bindings/wysiwyg-wasm/package.json
* Edit platforms/android/gradle.properties
* `make web` to update .lock files
* (For iOS the release script uses the git tag, so nothing to do I think.)
* `git checkout -b version-0.1.0`
* `git commit -a -m "Version 0.1.0"`
* `git tag 0.1.0 && git push --tags`
* Now push your branch and make a PR, and get it merged.

TODO: make a script that sets the git tag and pushes it, and updates the
various files containing the version number. And checks that the changelog
entry has been created.

## 4. Create the packages

### Web
Manually launch the
[github action](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/publish.yml)
which will package the code and upload it to NPM. It uses the version number
it finds in package.json, which you updated above.

### Android
Currently this must be done manually from a local development environment.

1. Copy the GPG secret keyring file to your machine
2. Add the following Maven credentials to `~/.gradle/gradle.properties`

```
mavenCentralUsername=xxx
mavenCentralPassword=xxx

signing.keyId=xxx
signing.password=xxx
signing.secretKeyRingFile=<path-to-keyring>
```

3. Build and publish the artifact

```
cd platforms/android && ./gradlew publish closeAndReleaseRepository`
```
  
### Swift/iOS:
  Run `./release_ios.sh` which will open a PR against
  [the swift package repo](https://github.com/matrix-org/matrix-wysiwyg-composer-swift)
  with the latest from main

TODO: automate all of this using a single github workflow that triggers when we
create a github release.

TODO: update release_io.sh to handle tags/releases
