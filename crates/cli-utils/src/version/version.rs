use cryfs_version::VersionInfo;

use crate::env::Environment;

// TODO Tests for different WARNING messages
pub fn show_version(env: &Environment, name: &str, version_info: VersionInfo) {
    // TODO If this happens due to the user specifying --version, we should print to stdout instead of stderr.
    eprintln!("{name} {version_info}");
    if let Some(gitinfo) = version_info.gitinfo() {
        if let Some(tag_info) = gitinfo.tag_info {
            if tag_info.commits_since_tag > 0 {
                eprintln!(
                    "WARNING! This is a development version based on git commit {}. Please don't use in production.",
                    gitinfo.commit_id,
                );
            }
        }
        if gitinfo.modified {
            eprintln!("WARNING! There were uncommitted changes in the repository when building this version.");
        }
    }
    if version_info.version().prerelease.is_some() {
        eprintln!("WARNING! This is a prerelease version. Please backup your data frequently!");
    }

    #[cfg(debug_assertions)]
    eprintln!("WARNING! This is a debug build. Performance might be slow.");

    #[cfg(feature = "check_for_updates")]
    _check_for_updates(env);
}

// TODO Tests
#[cfg(feature = "check_for_updates")]
fn _check_for_updates(env: &Environment) {
    if env.no_update_check {
        eprintln!("Automatic checking for security vulnerabilities and updates is disabled.");
    } else if env.is_noninteractive {
        eprintln!("Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode.");
    } else {
        // TODO
        // todo!()
    }
}
