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
./update-version.sh VERSION_NUMBER
```

This will change the version number across the Rust, Web and Android projects.

Then:
* `make web` to update .lock files
* (For iOS the release script uses the git tag, so nothing to do I think.)
* `git checkout -b version-X.Y.Z`
* `git commit -a -m "Version X.Y.Z"`
* `git push -u origin version-X.Y.Z`

Get the PR reviewed and merged to `main`.

* `git tag X.Y.Z`
* Now push your tag to the repo `git push X.Y.Z`.

TODO: automate tag creation when the `version-X.Y.Z` branch is merged.

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
cd platforms/android && ./gradlew publish closeAndReleaseRepository`
```
  
### Swift/iOS:
  Run `./release_ios.sh` which will open a PR against
  [the swift package repo](https://github.com/matrix-org/matrix-wysiwyg-composer-swift)
  with the latest from main

TODO: automate all of this using a single github workflow that triggers when we
create a github release.

TODO: update release_io.sh to handle tags/releases
