# How to contribute to `Finschia/cosmwasm`

First of all, thank you so much for taking your time to contribute!
It will be amazing if you could help us by doing any of the following:

- File an issue in [the issue tracker](https://github.com/Finschia/cosmwasm/issues) to report bugs and propose new features and
  improvements.
- Ask a question by creating a new issue in [the issue tracker](https://github.com/Finschia/cosmwasm/issues).
  - Browse [the list of previously answered questions](https://github.com/Finschia/cosmwasm/issues?q=label%3Aquestion).
- Contribute your work by sending [a pull request](https://github.com/Finschia/cosmwasm/pulls).

## Contributor license agreement

When you are sending a pull request and it's a non-trivial change beyond fixing typos, please sign
the ICLA (individual contributor license agreement). Please
[contact us](mailto:dev@finschia.org) if you need the CCLA (corporate contributor license agreement).

## Code of conduct

We expect contributors to follow [our code of conduct](CODE_OF_CONDUCT.md).

## Setting up your IDE

TBD

## Commit message and Pull Request message

- Follow [Conventional Commit](https://www.conventionalcommits.org) to release note automation.
- Don't mention or link that can't accessable from public.
- Use English only. Because this project will be published to the world-wide open-source world. But no worries. We are fully aware of that most of us are not the English-native.

## Pull Request Process

1. Ensure any install or build dependencies are removed before the end of the layer when doing a
   build.
2. Update the [README.md](README.md) with details of changes to the interface, this includes new environment
   variables, exposed ports, useful file locations and container parameters.
3. Fill out all sections of the pull request template. That makes it easier to review your PR for the reviewers.
4. You may merge the pull request in once you have the sign-off of two other developers, or if you
   do not have permission to do that, you may request the second reviewer to merge it for you.

## Releases

Release is maintained in a release branch named `vX.X.X-YYY+Z.Z.Z` where `vX.X.X` is the original CosmWasm's release version, `Z.Z.Z` is our release version and if additional information(e.g. rc, alpha) is needed, add `YYY`.  

The reason we use build metadata (`+`) instead of pre-release versioning (`-`) is because CosmWasm's release version overrides the pre-release version.
