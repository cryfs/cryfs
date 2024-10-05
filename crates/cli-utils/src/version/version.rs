use std::io::Write;

use crate::env::Environment;
use cryfs_version::{Version, VersionInfo};

#[cfg(feature = "check_for_updates")]
use super::http_client::HttpClient;

pub fn show_version(
    env: &Environment,
    name: &str,
    #[cfg(feature = "check_for_updates")] http_client: impl HttpClient,
    version_info: VersionInfo,
) {
    _show_version(
        env,
        name,
        #[cfg(feature = "check_for_updates")]
        http_client,
        version_info,
        &mut std::io::stderr(),
    );
}

fn _show_version(
    env: &Environment,
    name: &str,
    #[cfg(feature = "check_for_updates")] http_client: impl HttpClient,
    version_info: VersionInfo,
    stderr: &mut (impl Write + ?Sized),
) {
    // TODO If this happens due to the user specifying --version, we should print to stdout instead of stderr.
    write!(stderr, "{name} {version_info}\n").unwrap();
    if let Some(gitinfo) = version_info.gitinfo() {
        if let Some(tag_info) = gitinfo.tag_info {
            if tag_info.commits_since_tag > 0 {
                write!(stderr,
                    "WARNING! This is a development version based on git commit {}. Please don't use in production.\n",
                    gitinfo.commit_id,
                ).unwrap();
            }
        }
        if gitinfo.modified {
            write!(stderr, "WARNING! There were uncommitted changes in the repository when building this version.\n").unwrap();
        }
    }
    if version_info.version().prerelease.is_some() {
        write!(
            stderr,
            "WARNING! This is a prerelease version. Please backup your data frequently!\n"
        )
        .unwrap();
    }

    #[cfg(debug_assertions)]
    write!(
        stderr,
        "WARNING! This is a debug build. Performance might be slow.\n"
    )
    .unwrap();

    #[cfg(feature = "check_for_updates")]
    _maybe_check_for_updates(env, http_client, version_info.version(), stderr);
}

#[cfg(feature = "check_for_updates")]
fn _maybe_check_for_updates<'a>(
    env: &Environment,
    http_client: impl HttpClient,
    version: Version<'a>,
    stderr: &mut (impl Write + ?Sized),
) {
    use super::update_checker::UpdateCheckResult;

    if env.no_update_check {
        write!(
            stderr,
            "Automatic checking for security vulnerabilities and updates is disabled.\n"
        )
        .unwrap();
    } else if env.is_noninteractive {
        write!(stderr, "Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode.\n").unwrap();
    } else {
        match super::update_checker::check_for_updates(http_client, version) {
            Ok(UpdateCheckResult {
                released_newer_version,
                security_warning,
            }) => {
                if let Some(released_newer_version) = released_newer_version {
                    write!(
                        stderr,
                        "A newer version of CryFS is available: {}. You are using version {}.\n",
                        released_newer_version, version
                    )
                    .unwrap();
                }
                if let Some(security_warning) = security_warning {
                    write!(stderr, "{security_warning}").unwrap();
                }
            }
            Err(e) => {
                write!(stderr, "Failed to check for updates: {e}\n").unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cryfs_version::{GitInfo, TagInfo};
    use std::io::BufWriter;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(feature = "check_for_updates")]
    use crate::version::FakeHttpClient;

    use super::*;

    fn show_version_capture_output(
        env: &Environment,
        name: &str,
        #[cfg(feature = "check_for_updates")] http_client: impl HttpClient,
        version_info: VersionInfo,
    ) -> String {
        let mut output = BufWriter::new(Vec::new());
        _show_version(
            env,
            name,
            #[cfg(feature = "check_for_updates")]
            http_client,
            version_info,
            &mut output,
        );
        String::from_utf8(output.into_inner().unwrap()).unwrap()
    }

    #[cfg(feature = "check_for_updates")]
    fn fake_http_client<'a>(newest_version: Version<'a>) -> FakeHttpClient {
        let mut http_client = FakeHttpClient::new();
        http_client.add_website(
            "https://www.cryfs.org/version_info.json".to_string(),
            format!("{{\"version_info\":{{\"current\":\"{newest_version}\"}}}}"),
        );
        http_client
    }

    #[cfg(feature = "check_for_updates")]
    fn fake_http_client_invalid<'a>() -> FakeHttpClient {
        FakeHttpClient::new()
    }

    #[cfg(feature = "check_for_updates")]
    fn fake_http_client_with_security_warning<'a>(
        newest_version: Version<'a>,
        security_warning_version: &str,
        security_warning: &str,
    ) -> FakeHttpClient {
        let mut http_client = FakeHttpClient::new();
        http_client.add_website(
            "https://www.cryfs.org/version_info.json".to_string(),
            format!("{{\"version_info\":{{\"current\":\"{newest_version}\"}}, \"warnings\":{{\"{security_warning_version}\":\"{security_warning}\"}}}}"),
        );
        http_client
    }

    fn environment() -> Environment {
        Environment {
            is_noninteractive: false,
            #[cfg(feature = "check_for_updates")]
            no_update_check: false,
            local_state_dir: std::path::PathBuf::new(),
        }
    }

    mod version_number_and_warnings {
        use super::*;

        #[track_caller]
        fn assert_shows_version_number(output: &str, version: &str) {
            assert!(
                output.starts_with(&format!("AppName {version}\n")),
                "Expected `AppName {version}` found `{output}`"
            );
        }

        const DEVELOPMENT_VERSION_WARNING: &str =
            "WARNING! This is a development version based on git commit";

        #[track_caller]
        fn assert_contains_development_version_warning(output: &str, commit_id: &str) {
            assert!(output.contains(&format!(
                "{DEVELOPMENT_VERSION_WARNING} {commit_id}. Please don't use in production."
            )));
        }

        #[track_caller]
        fn assert_doesnt_contain_development_version_warning(output: &str) {
            assert!(!output.contains(DEVELOPMENT_VERSION_WARNING));
        }

        const UNCOMMITTED_CHANGES_WARNING: &str =
            "WARNING! There were uncommitted changes in the repository when building this version.";

        #[track_caller]
        fn assert_contains_uncommitted_changes_warning(output: &str) {
            assert!(output.contains(UNCOMMITTED_CHANGES_WARNING));
        }

        #[track_caller]
        fn assert_doesnt_contain_uncommitted_changes_warning(output: &str) {
            assert!(!output.contains(UNCOMMITTED_CHANGES_WARNING));
        }

        const PRERELEASE_WARNING: &str =
            "WARNING! This is a prerelease version. Please backup your data frequently!";

        #[track_caller]
        fn assert_contains_prerelease_warning(output: &str) {
            assert!(output.contains(PRERELEASE_WARNING));
        }

        #[track_caller]
        fn assert_doesnt_contain_prerelease_warning(output: &str) {
            assert!(!output.contains(PRERELEASE_WARNING));
        }

        const DEBUG_BUILD_WARNING: &str =
            "WARNING! This is a debug build. Performance might be slow.";

        #[track_caller]
        fn assert_debug_warning_shown_if_debug_build(output: &str) {
            #[cfg(debug_assertions)]
            assert!(output.contains(DEBUG_BUILD_WARNING));
            #[cfg(not(debug_assertions))]
            assert!(!output.contains(DEBUG_BUILD_WARNING));
        }

        #[test]
        fn release_version() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3",
                        commits_since_tag: 0,
                    }),
                    modified: false,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3");
            assert_doesnt_contain_development_version_warning(&output);
            assert_doesnt_contain_uncommitted_changes_warning(&output);
            assert_doesnt_contain_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn release_version_no_gitinfo() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3");
            assert_doesnt_contain_development_version_warning(&output);
            assert_doesnt_contain_uncommitted_changes_warning(&output);
            assert_doesnt_contain_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn release_version_uncommitted_changes() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3",
                        commits_since_tag: 0,
                    }),
                    modified: true,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3+modified");
            assert_doesnt_contain_development_version_warning(&output);
            assert_contains_uncommitted_changes_warning(&output);
            assert_doesnt_contain_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn release_version_extra_commits() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3",
                        commits_since_tag: 1,
                    }),
                    modified: false,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3+1.gabcdef");
            assert_contains_development_version_warning(&output, "abcdef");
            assert_doesnt_contain_uncommitted_changes_warning(&output);
            assert_doesnt_contain_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn release_version_extra_commits_uncommitted_changes() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3",
                        commits_since_tag: 1,
                    }),
                    modified: true,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3+1.gabcdef.modified");
            assert_contains_development_version_warning(&output, "abcdef");
            assert_contains_uncommitted_changes_warning(&output);
            assert_doesnt_contain_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn prerelease_version() {
            let version = Version::parse_const("1.2.3-rc1").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3-rc1",
                        commits_since_tag: 0,
                    }),
                    modified: false,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3-rc1");
            assert_doesnt_contain_development_version_warning(&output);
            assert_doesnt_contain_uncommitted_changes_warning(&output);
            assert_contains_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn prerelease_version_no_gitinfo() {
            let version = Version::parse_const("1.2.3-rc1").unwrap();
            let version_info = VersionInfo::new(version, None);
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3-rc1");
            assert_doesnt_contain_development_version_warning(&output);
            assert_doesnt_contain_uncommitted_changes_warning(&output);
            assert_contains_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn prerelease_version_uncommitted_changes() {
            let version = Version::parse_const("1.2.3-rc1").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3-rc1",
                        commits_since_tag: 0,
                    }),
                    modified: true,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3-rc1+modified");
            assert_doesnt_contain_development_version_warning(&output);
            assert_contains_uncommitted_changes_warning(&output);
            assert_contains_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn prerelease_version_extra_commits() {
            let version = Version::parse_const("1.2.3-rc1").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3-rc1",
                        commits_since_tag: 1,
                    }),
                    modified: false,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3-rc1+1.gabcdef");
            assert_contains_development_version_warning(&output, "abcdef");
            assert_doesnt_contain_uncommitted_changes_warning(&output);
            assert_contains_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }

        #[test]
        fn prerelease_version_extra_commits_uncommitted_changes() {
            let version = Version::parse_const("1.2.3-rc1").unwrap();
            let version_info = VersionInfo::new(
                version,
                Some(GitInfo {
                    commit_id: "abcdef",
                    tag_info: Some(TagInfo {
                        tag: "1.2.3-rc1",
                        commits_since_tag: 1,
                    }),
                    modified: true,
                }),
            );
            let output = show_version_capture_output(
                &environment(),
                "AppName",
                #[cfg(feature = "check_for_updates")]
                fake_http_client(version),
                version_info,
            );
            assert_shows_version_number(&output, "1.2.3-rc1+1.gabcdef.modified");
            assert_contains_development_version_warning(&output, "abcdef");
            assert_contains_uncommitted_changes_warning(&output);
            assert_contains_prerelease_warning(&output);
            assert_debug_warning_shown_if_debug_build(&output);
        }
    }

    mod update_checks {
        use super::*;

        fn assert_didnt_do_update_check(
            output: &str,
            #[cfg(feature = "check_for_updates")] http_request_counter: &AtomicUsize,
        ) {
            assert!(!output.contains("A newer version of CryFS is available"));
            assert!(!output.contains("Failed to check for updates"));

            #[cfg(feature = "check_for_updates")]
            assert_eq!(0, http_request_counter.load(Ordering::SeqCst));
        }

        fn assert_did_do_update_check(
            output: &str,
            http_request_counter: &AtomicUsize,
            current_version: &str,
            newest_version: &str,
        ) {
            if current_version != newest_version {
                assert!(output.contains(&format!("A newer version of CryFS is available: {newest_version}. You are using version {current_version}.")), "Expected `A newer version of CryFS is available: {newest_version}. You are using version {current_version}.` found `{output}`");
            } else {
                assert!(!output.contains(&format!("A newer version of CryFS is available")));
            }
            assert_eq!(1, http_request_counter.load(Ordering::SeqCst));
        }

        fn assert_did_do_update_check_and_failed(output: &str, http_request_counter: &AtomicUsize) {
            assert!(output.contains("Failed to check for updates"));
            assert_eq!(1, http_request_counter.load(Ordering::SeqCst));
        }

        #[cfg(not(feature = "check_for_updates"))]
        #[test]
        fn update_check_disabled_by_compile_flag() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let env = Environment {
                is_noninteractive: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", version_info);
            assert_didnt_do_update_check(&output);
        }

        #[cfg(not(feature = "check_for_updates"))]
        #[test]
        fn update_check_disabled_by_compile_flag_and_noninteractive_mode() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let env = Environment {
                is_noninteractive: true,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", version_info);
            assert_didnt_do_update_check(&output);
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn update_check_disabled_by_environment() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client(version);
            let http_request_counter = http_client.request_counter();
            let env = Environment {
                is_noninteractive: false,
                no_update_check: true,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert!(output.contains(
                "Automatic checking for security vulnerabilities and updates is disabled."
            ));
            assert_didnt_do_update_check(&output, &*http_request_counter);
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn update_check_disabled_in_noninteractive_mode() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client(version);
            let http_request_counter = http_client.request_counter();
            let env = Environment {
                is_noninteractive: true,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert!(output.contains("Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode."));
            assert_didnt_do_update_check(&output, &*http_request_counter);
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn update_check_enabled_new_version_available() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client(Version::parse_const("1.2.4").unwrap());
            let http_request_counter = http_client.request_counter();
            let env = Environment {
                is_noninteractive: false,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert_did_do_update_check(&output, &*http_request_counter, "1.2.3", "1.2.4");
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn update_check_enabled_new_version_unavailable() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client(Version::parse_const("1.2.3").unwrap());
            let http_request_counter = http_client.request_counter();
            let env = Environment {
                is_noninteractive: false,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert_did_do_update_check(&output, &*http_request_counter, "1.2.3", "1.2.3");
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn update_check_enabled_new_version_invalid() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client_invalid();
            let http_request_counter = http_client.request_counter();
            let env = Environment {
                is_noninteractive: false,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert_did_do_update_check_and_failed(&output, &*http_request_counter);
        }
    }

    mod security_warnings {
        use super::*;

        const SECURITY_WARNING: &str = "my security warning";

        #[track_caller]
        fn assert_contains_security_warning(output: &str, security_warning: &str) {
            assert!(
                output.contains(security_warning),
                "Expected `{security_warning}` found `{output}`"
            );
        }

        #[track_caller]
        fn assert_doesnt_contain_security_warning(output: &str, security_warning: &str) {
            assert!(!output.contains(security_warning));
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn no_security_warning() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client(Version::parse_const("1.2.3").unwrap());
            let env = Environment {
                is_noninteractive: false,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert_doesnt_contain_security_warning(&output, SECURITY_WARNING);
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn security_warning() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client_with_security_warning(
                Version::parse_const("1.2.3").unwrap(),
                "1.2.3",
                SECURITY_WARNING,
            );
            let env = Environment {
                is_noninteractive: false,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert_contains_security_warning(&output, SECURITY_WARNING);
        }

        #[cfg(feature = "check_for_updates")]
        #[test]
        fn security_warning_for_different_version() {
            let version = Version::parse_const("1.2.3").unwrap();
            let version_info = VersionInfo::new(version, None);
            let http_client = fake_http_client_with_security_warning(
                Version::parse_const("1.2.3").unwrap(),
                "1.2.2",
                SECURITY_WARNING,
            );
            let env = Environment {
                is_noninteractive: false,
                no_update_check: false,
                local_state_dir: std::path::PathBuf::new(),
            };
            let output = show_version_capture_output(&env, "AppName", http_client, version_info);
            assert_doesnt_contain_security_warning(&output, SECURITY_WARNING);
        }
    }
}
