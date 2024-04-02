# Releasing matrix-wysiwyg

Normally this would be done by the project owners.

Here are the steps we take:

## 1. Choose a version number

We use semantic versioning.

It can't be anything that has been pushed to any package repo before.

## 2. Add a changelog entry

Currently this is stored in [CHANGELOG.md](CHANGELOG.md).

TODO: store the canonical version in the project root, and copy it to there
so the NPM packaging can find it.

## 3. Set the version number

To change the current version, run the script:

```shell
./update_version.sh VERSION_NUMBER
```

This will change the version number across the Rust, Web and Android projects.

Then:
* `make web` to update .lock files
* `git checkout -b version-X.Y.Z`
* `git commit -a -m "Version X.Y.Z"`
* `git push -u origin version-X.Y.Z`

Get the PR reviewed and merged to `main`.
A workflow will automatically add the tag on main with the version provided in the branch name.

## 4. Create the packages

### Web

This should be done automatically when a tag is uploaded, but you can also manually launch the
[github action](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/publish.yml)
which will package the code and upload it to NPM. It uses the version number
it finds in package.json, which you updated above.

### Android

This should be done automatically when a tag is uploaded, but it can also be done manually from a local development environment.

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
cd platforms/android && ./gradlew publish closeAndReleaseRepository
```
  
### Swift/iOS:
When a tag is added the tool in `platforms/ios/tools/release` will be run with the `--version <version>` (where `<version>` will be the provided tag).
This will build the binaries and create the new release on [the swift package repo](https://github.com/matrix-org/matrix-rich-text-editor-swift).

To manually make a release, first set the `SWIFT_RELEASE_TOKEN` environment variable and then run `swift run release --version <version>`.
