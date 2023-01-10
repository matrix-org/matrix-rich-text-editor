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
cd platforms/android && ./gradlew publish closeAndReleaseRepository`
```
  
### Swift/iOS:
When a tag is uploaded the `./release_ios.sh` script will be called with the `-t <version>` (where `<version>` will be the provided tag).
This will push a branch named release_v_X.Y.Z on the [the swift package repo](https://github.com/matrix-org/matrix-wysiwyg-composer-swift) which will trigger the following [workflow](https://github.com/matrix-org/matrix-wysiwyg-composer-swift/blob/main/.github/workflows/pr_on_release.yml) to open a PR to main with the latest changes.
Remember to manually check the PR that will get opened on the [SWIFTPM Repo PR list](https://github.com/matrix-org/matrix-wysiwyg-composer-swift/pulls), and before merging it provide as commit message the name of the branch/PR.

Example: 
1. The branch and PR name will always be something like release_v_X.Y.Z
2. When clicking Merge customise the commit message to only include as title the same name release_v_X.Y.Z (substitute the "Merge pull request..." default commit message and put only the pr/branch name)
3. This will trigger on merge a [workflow](https://github.com/matrix-org/matrix-wysiwyg-composer-swift/blob/main/.github/workflows/tag_on_release.yml) that will automatically tag main with the release value of X.Y.Z

You can also manually do a revision release by just running `./release_ios.sh` on main which will not create a release PR but a revision PR that can have any commit message after the merge.

Instead for full manual release just run `./release_ios.sh -t <version>` and follow the example steps above.
