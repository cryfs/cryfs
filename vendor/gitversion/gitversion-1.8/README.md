# git-version [![Build Status](https://travis-ci.org/smessmer/gitversion.svg?branch=master)](https://travis-ci.org/smessmer/gitversion)
Make git version information (e.g. git tag name, git commit id, ...) available to your source files.
A simple use case scenario is to output this version information when the application is called with "--version".

This repository contains
  - A python script to generate C++ headers or python modules with this version information. You can add the python script to your build process to autogenerate the files on each build.
  - A CMake script which can be directly included into a CMake projects. It will then automatically be run on each build and you only have to #include the generated file.


Use with cmake (only C++)
================

Copy this repository into a subfolder of your project and include the cmake.cmake file in your CMakeLists.txt

    INCLUDE(gitversion/cmake.cmake)
    TARGET_GIT_VERSION_INIT(buildtarget)

Then, you can write in your source file:

    #include <gitversion/version.h>
    cout << version::VERSION_STRING << endl;
    cout << version::IS_STABLE_VERSION << endl;
    cout << version::GIT_COMMIT_ID << endl;
    cout << version::GIT_COMMITS_SINCE_TAG << endl;
    // ... (see below for more variables)

That's it already. Have fun :)

Use manually (C++ and Python)
================

Install from PyPi
----------------

To install the tool:

    pip install git-version

To generate a version.h file containing C++ version information for the git repository located in myrepositorydir:

    python -m gitversionbuilder --dir myrepositorydir --lang cpp version.h

Or to generate a module with version information for python:

    python -m gitversionbuilder --dir myrepositorydir --lang python version.py


Run script from source tree
-------------------------

If you don't want to use PyPi, you can run the script directly from the source tree.
Clone this repository and go to the src directory (or alternatively add the src directory to the PYTHONPATH environment variable), then call for example

    python -m gitversionbuilder --dir myrepositorydir --lang cpp version.h
    
If you want to build a distribution of the package to use it somewhere else, you can use the standard python [setuptools](https://pythonhosted.org/setuptools/).
A corresponding setup.py is available in the directory.


Available Information
=================

Basic Information
-----------------
The following table shows the basic variables that are always available.

<table>
  <tr>
    <th rowspan="6">VERSION_STRING</th>
    <td style="white-space: nowrap;">1.0</td>
    <td>Built from git tag "1.0".</td>
  </tr>
  <tr>
    <td style="white-space: nowrap;">v0.8alpha</td>
    <td>Built from git tag "v0.8alpha".</td>
  </tr>
  <tr>
    <td style="white-space: nowrap;">0.8.dev3+rev4fa254c
    <td>Built from 3 commits after git tag "0.8". The current git commit has commit id 4fa254c.
  </tr>
  <tr>
    <td style="white-space: nowrap;">dev2+rev4fa254c</td>
    <td>The repository doesn't have any git tags yet. There are 2 commits since the repository started and the current git commit has commit id 4fa254c.</td>
  </tr>
  <tr>
    <td>0.8-modified</td>
    <td rowspan="2">The suffix "-modified" will be used if there have been modifications since the last commit.</td>
  </tr>
  <tr>
    <td>0.8.dev3+rev4fa254c-modified</td>
  </tr>

  <tr>
    <th>GIT_TAG_NAME</th>
    <td colspan="2">The name of the last git tag. If there is no git tag, then this is the name of the git branch.</td>
  </tr>

  <tr>
    <th>GIT_COMMITS_SINCE_TAG</th>
    <td colspan="2">The number of git commits since the last git tag. If the repository doesn't have any git tags, then this is the number of git commits since the repository started</td>
  </tr>

  <tr>
    <th>GIT_COMMIT_ID</th>
    <td colspan="2">The commit id of the git commit this was built from.</td>
  </tr>

  <tr>
    <th>MODIFIED_SINCE_COMMIT</th>
    <td colspan="2">True, if there are uncommitted changes in the git working directory or index since the last commit; i.e. untracked (and not ignored) files, or modified files in the working directory or the index.</td>
  </tr>

  <tr>
    <th>IS_DEV_VERSION</th>
    <td colspan="2">True, if this is a development version; i.e. there are no tags yet or GIT_COMMITS_SINCE_TAG > 0 or MODIFIED_SINCE_COMMIT.</td>
  </tr>
</table>

Additional Information
----------------------

We will parse the git tag name and provide additional information if you use the following versioning scheme for your git tag names:


    /^v?[0-9]+(\.[0-9]+)*(-?((alpha|beta|rc|pre|m)[0-9]?|stable|final))?$/

In words, we support a set of numeric version components separated by a dot, then optionally a version tag like "alpha", "beta", "beta2", "rc2", "M3", "pre2", "stable", "final". The version tag can optionally be separated with a dash and the version number can optionally be prefixed with "v".
The version tag is matched case insensitive. It is for example also allowed to write "RC" instead of "rc".

Examples for supported version numbers:

   - 0.8.1
   - v3.0
   - 1.1-alpha
   - 1.2alpha
   - 1.4.3beta
   - 1.4.3beta2
   - 2.0-M2
   - 4-RC2
   - 3.0final
   - 2.1-stable
   - ...

If you use a version scheme supported by this, we will provide the following additional information

<table>
  <tr>
    <th>IS_STABLE_VERSION</th>
    <td>True, if built from a final tag; i.e. IS_DEV_VERSION == false and GIT_COMMITS_SINCE_TAG == 0 and VERSION_TAG in {"", "stable", "final"}</td>
  </tr>

  <tr>
    <th>VERSION_COMPONENTS</th>
    <td>An array containing the version number split at the dots. That is, git tag "1.02.3alpha" will have VERSION_COMPONENTS=["1","02","3"].</td>
  </tr>

  <tr>
    <th>VERSION_TAG</th>
    <td>The version tag ("alpha", "beta", "rc4", "M2", "stable", "final", "", ...) that follows after the version number. If the version tag is separated by a dash, the dash is not included.</td>
  </tr>
</table>


