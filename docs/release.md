# How to release.
- Please refer to the following file for the config file for release.
1. .github/changelog.yml
  - Update CHANGELOG when issuing a PR for release.
2. .github/release.yml
  - Executed when PR is merged into main.

### What we can do with this.
1. Automatically create a new version from the branch name.
2. Automatically generate and commit a changelog.
3. Upload dev contracts
4. Publish

### Release process
1. Update the version of Cargo.toml.
2. Make PR to main branch
  - The format of the branch name should be release-vx.x.x-x.x.x.
  - PR's comment becomes the body of the release notes.
3. On the PR, CI job(changelog.yml) extracts target version from branch name.
4. CI job create new tag from branch name.
5. CI job generates the changelog and appends to CHANGELOG-LINK.md
  - using [git-chglog](https://github.com/git-chglog/git-chglog)
6. Github Actions automatically commit CHANGELOG-LINK.md.
7. PR review.
8. merge -> main
9. On the main, CI job(release.yml) publish artifacts and create release notes.
