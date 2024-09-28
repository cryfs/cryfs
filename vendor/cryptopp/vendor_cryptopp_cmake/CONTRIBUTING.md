# Contributing to this project

👍🎉 Thanks for taking the time to contribute! 🎉👍

## Table of Contents

- [Conventional Commits](#conventional-commits)
- [Conventional Changelog](#conventional-changelog)
- [New Release Process](#new-release-process)
- [Testing](#testing)
- [GitHub Actions](#github-actions)

## Conventional Commits

The project is setup for husky and [conventional
commits](https://www.conventionalcommits.org/en/v1.0.0/) to keep some standard
for the commit messages.

In order to be able to use that, have `nodejs` and `npm` installed in your
environment and run the following just one time after you clone this project:

```shell
npx husky install
npm install -g @commitlint/cli @commitlint/config-conventional
```

Commit message are linted automatically.

## Conventional Changelog

The project is also setup for [conventional
changelog](https://github.com/conventional-changelog/standard-version) to
automatically generate change logs.

In order to be able to use that, have `nodejs` and `npm` installed in your
environment and run the following just one time after you clone this project:

```shell
npm install -g standard-version
```

When the project is ready for a new release, run the following command in the
project root to update the `CHANGELOG.md` and the `CMakeLists.txt` files:

```shell
npx standard-version --skip.commit --skip.tag -r M.m.p
```

M.m.p is version number to be released:

The version number will be automatically bumped in the `CMakeLists.txt` and the
`CHANGELOG.md` file will be automatically updated. Open both of them, check the
changes, lint and format the `CHANGELOG.md` and write any additional notes, then
commit.

## Testing

Testing is integrated into the project and is automated via `ctest`.

## New release process

Create a new tag for the release by using the following command:

```shell
git tag -a CRYPTOPP_M_m_p -m "Blah blah blah..."
```

> :warning: **Pay attention to the format of the tag**: the version uses `_` and
> not `.`!
> Also note that *patch tags* will also have a sequential number suffix (e.g.
> CRYPTOPP_8_7_0_1).

Push with the following command:

```shell
git push --follow-tags
```

## GitHub Actions

The automatic GitHub actions will take care of the rest, including the
multi-platform builds, the testing, and when everything is successful, the
creation of a release and its associated artifacts.

> Here are the links where you can check the result of the actions:
>
> [GitHubActions](https://github.com/abdes/cryptopp-cmake/actions)
>
> [GitHub Releases](https://github.com/abdes/cryptopp-cmake/releases)
