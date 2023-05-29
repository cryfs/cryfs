git2version
===============

This crate provides a way to get the version of the package from git.


Setup
--------------

To use this, you need to setup a proxy-crate in your workspace.

1. Add this to your Cargo.toml:

```toml
[workspace]

[dependencies]
version_proxy = {path = "./version_proxy"}
```

2. Add these files to make up the proxy crate:

**version_proxy/Cargo.toml**:
```toml
[package]
name = "version_proxy"
# The version field here is ignored, no need to change it
version = "0.0.0"

[dependencies]
git2version = "*"

[build-dependencies]
git2version = "*"
```
You can also lock the version of git2version to a specific version instead of using `*`.

**version_proxy/build.rs**:
```rust
fn main() {
    git2version::init_proxy_build!();
}
```

**version_proxy/src/lib.rs**:
```rust
git2version::init_proxy_lib!();
```

Usage
--------------

The `init_proxy_lib!` macro in your proxy crate will generate something similar to the following:
```rust
pub const GITINFO: Option<git2version::GitInfo> =
    Some(git2version::GitInfo {
        tag: "v1.2.3-alpha",
        commits_since_tag: 5,
        commit_id: "a9ebd080a7",
        modified: false,
    });
``` 
This object can be `None` if the crate is not in a git repository or if there was an error looking up the version information from git.

You can use this const from your main crate, for example like this:
```rust
fn main() {
    println!("Version from git: {:?}", version_proxy::GITINFO);
}
```

Alternatives
--------------
The [git-version](https://crates.io/crates/git-version) crate provides similar functionality.

The main advantage of `git-version` over `git2version` is that it is much simpler to use. It uses a proc-macro based approach and doesn't require you to set up a proxy crate.

The advantages of `git2version` over `git-version` are as follows:
* `git2version` uses the [git2](https://crates.io/crates/git2) crate to read git information. This means it works without requiring a `git` executable in your path.
* `git2version` outputs structured information about the git version, while `git-version` only outputs a string as produced by `git describe`.
  In `git-version`, you have to parse that string yourself and it might not always contain all the information (e.g. `git describe --tags` doesn't output the commit id when you have the
  tag itself checked out). `git2version` always gives you the commit id.

Another point of note is that both crates use a different mechanism for change detection for incremental builds.
* `git2version` uses the `cargo:rerun-if-changed` mechanism of `build.rs` to re-generate the version number whenever the git repository changes (e.g. new tags being added, `git fetch` being called, ...)
  and whenever files in the working copy change. The latter is important because it could cause a change to the `-modified` flag of the reported version.
* `git-version` uses an `include_bytes!` mechanism to include bytes from your git repository data into the generated source code, which will cause cargo to detect it as a dependency and rerun the proc macro
  when the git repository data changes. This sounds hacky but might work. I have not tested how reliable or scalable that approach is.
`cargo:rerun-if-changed` is the officially supported way to do this kind of change detection, so I would expect it to be more reliable, but it only works for `build.rs` scripts, not for proc macros.

Why is the proxy crate required?
--------------------------------

The crate needs to know the directory of your git repository to read version information.
However, the `git2version` crate gets compiled independently from that and doesn't have access to your git repository.
This is why we need a proxy crate inside of your git repository that knows its location and can evaluate the version information.

You may ask why we do it in a proxy crate instead of just having your main crate evaluate the version information, after all
your main crate is also in your repository. The reason is that the `build.rs` code used to evaluate the version information
needs to run **after every single file modification** because that could influence the `modified` tag of the git version information.
If we put this into your main crate, then incremental compilations become basically useless because it needs to re-compile everything
for every change. By putting it into a proxy crate, we only need to re-compile the code in the proxy crate and link your main crate
against it. 