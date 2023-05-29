// This needs to be a macro instead of just a function because we need the `CARGO_MANIFEST_DIR`
// of the client library, not our own.
#[macro_export]
macro_rules! init_proxy_build {
    () => {
        use $crate::GitInfo;

        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");

        fn output_none() {
            println!("cargo:rustc-env=PACKAGEVERSION_GITVERSION_IS_KNOWN=false");
            println!("cargo:rustc-env=PACKAGEVERSION_GITVERSION_TAG=",);
            println!("cargo:rustc-env=PACKAGEVERSION_GITVERSION_COMMITS_SINCE_TAG=",);
            println!("cargo:rustc-env=PACKAGEVERSION_GITVERSION_COMMIT_ID=",);
            println!("cargo:rustc-env=PACKAGEVERSION_GITVERSION_MODIFIED=",);
        }

        let repo = match $crate::git2::Repository::discover(cargo_manifest_dir) {
            Ok(repo) => Some(repo),
            Err(err) => {
                println!("cargo:warning=Error getting version info from git, didn't find git repository: {}", err);
                None
            }
        };
        let repository_version = repo.as_ref().and_then(|repo|
            match $crate::get_git_info(&repo) {
                Ok(git_info) => Some(git_info),
                Err(err) => {
                    println!("cargo:warning=Error getting version info from git: {}", err);
                    None
                }
            }
        );

        if let Some(repository_version) = repository_version {
            println!("cargo:rustc-env=PACKAGEVERSION_GITVERSION_IS_KNOWN=true");
            println!(
                "cargo:rustc-env=PACKAGEVERSION_GITVERSION_TAG={}",
                repository_version.tag
            );
            println!(
                "cargo:rustc-env=PACKAGEVERSION_GITVERSION_COMMITS_SINCE_TAG={}",
                repository_version.commits_since_tag
            );
            println!(
                "cargo:rustc-env=PACKAGEVERSION_GITVERSION_COMMIT_ID={}",
                repository_version.commit_id
            );
            println!(
                "cargo:rustc-env=PACKAGEVERSION_GITVERSION_MODIFIED={}",
                repository_version.modified
            );
        } else {
            output_none();
        }

        if let Some(repo) = repo {
            // Rerun the build script if any files changed. This is necessary to correctly update
            // the `.dirty` flag of version numbers
            println!(
                "cargo:rerun-if-changed={repo_workspace_path}",
                repo_workspace_path = repo.workdir().unwrap().display()
            );

            // Also rerun the build script if anything in the .git repository changed.
            // This is for the case where our `Cargo.toml` is in a subdirectory of the
            // main git repository. In this case, we still need to react to changes in
            // the git repository.
            println!(
                "cargo:rerun-if-changed={repo_path}",
                repo_path = repo.path().display()
            );
        } else {
            // We didn't find a git repository. Let's rerun if the directory of the `Cargo.toml`
            // changed to check if a git repository got added. Note: This won't catch cases where
            // a git repository is added as a parent directory, but probably nothing we can do
            // about that.
            println!(
                "cargo:rerun-if-changed={cargo_manifest_dir}",
            );
        }
    };
}

#[macro_export]
macro_rules! init_proxy_lib {
    () => {
        pub use $crate::*;

        pub const GITINFO: Option<$crate::GitInfo> = if $crate::konst::unwrap_ctx!(
            $crate::konst::primitive::parse_bool(env!("PACKAGEVERSION_GITVERSION_IS_KNOWN"))
        ) {
            Some($crate::GitInfo {
                tag: env!("PACKAGEVERSION_GITVERSION_TAG"),
                commits_since_tag: $crate::konst::unwrap_ctx!($crate::konst::primitive::parse_u32(
                    env!("PACKAGEVERSION_GITVERSION_COMMITS_SINCE_TAG")
                )),
                commit_id: env!("PACKAGEVERSION_GITVERSION_COMMIT_ID"),
                modified: $crate::konst::unwrap_ctx!($crate::konst::primitive::parse_bool(env!(
                    "PACKAGEVERSION_GITVERSION_MODIFIED"
                ))),
            })
        } else {
            None
        };
    };
}
