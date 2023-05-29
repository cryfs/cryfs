use git2::{Commit, Repository};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

use git2version::{GitInfo, COMMIT_ID_SHORT_HASH_LENGTH};

const FILENAME: &str = "some_file";

fn create_initial_commit(repo: &Repository) {
    create_change(repo);
    add_all_changes_to_index(repo);
    commit(repo, &[], "Initial commit");
}

fn create_change(repo: &Repository) -> String {
    let content = rand::random::<u64>().to_string();
    std::fs::write(repo.workdir().unwrap().join(&FILENAME), &content).unwrap();
    content
}

fn add_all_changes_to_index(repo: &Repository) {
    let mut index = repo.index().unwrap();
    index
        .add_all(&["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
}

fn commit(repo: &Repository, parents: &[&Commit], description: &str) -> git2::Oid {
    let sig = repo.signature().unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, description, &tree, parents)
        .unwrap()
}

fn create_change_and_commit(repo: &Repository) -> git2::Oid {
    let content = create_change(repo);

    add_all_changes_to_index(repo);
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
    commit(
        repo,
        &[&head_commit],
        &format!("Commit {FILENAME}: {content}"),
    )
}

fn create_tag(repo: &Repository, tag: &str) {
    let head_commit = repo.head().unwrap().peel(git2::ObjectType::Commit).unwrap();
    repo.tag_lightweight(tag, &head_commit, true).unwrap();
}

fn create_some_commits_but_no_tags(repo: &Repository) {
    create_initial_commit(&repo);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
}

fn create_some_commits_and_a_tag(repo: &Repository, tag: &str) {
    create_initial_commit(&repo);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
    create_tag(&repo, &tag);
}

fn create_some_commits_a_tag_and_some_more_commits(repo: &Repository, tag: &str) {
    create_initial_commit(&repo);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
    create_tag(&repo, &tag);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
    create_change_and_commit(&repo);
}

#[test]
fn no_git() {
    let project_dir = make_version_test_project();
    run_version_test_project(project_dir.path(), None);
}

#[test]
fn empty_git() {
    let project_dir = make_version_test_project();
    Repository::init(&project_dir).unwrap();
    run_version_test_project(project_dir.path(), None);
}

#[test]
fn with_initial_commit_notmodified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_initial_commit(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "",
            commits_since_tag: 1,
            commit_id: &head_commit_id(&repo),
            modified: false,
        }),
    );
}

#[test]
fn with_initial_commit_modified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_initial_commit(&repo);
    create_change(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "",
            commits_since_tag: 1,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn with_initial_commit_modified_staged() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_initial_commit(&repo);
    create_change(&repo);
    add_all_changes_to_index(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "",
            commits_since_tag: 1,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn with_some_commits_but_no_tags_notmodified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_but_no_tags(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "",
            commits_since_tag: 3,
            commit_id: &head_commit_id(&repo),
            modified: false,
        }),
    );
}

#[test]
fn with_some_commits_but_no_tags_modified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_but_no_tags(&repo);
    create_change(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "",
            commits_since_tag: 3,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn with_some_commits_but_no_tags_modified_staged() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_but_no_tags(&repo);
    create_change(&repo);
    add_all_changes_to_index(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "",
            commits_since_tag: 3,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn on_tag_notmodified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_and_a_tag(&repo, "v1.2.3-alpha");
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "v1.2.3-alpha",
            commits_since_tag: 0,
            commit_id: &head_commit_id(&repo),
            modified: false,
        }),
    );
}

#[test]
fn on_tag_modified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_and_a_tag(&repo, "v1.2.3-alpha");
    create_change(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "v1.2.3-alpha",
            commits_since_tag: 0,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn on_tag_modified_staged() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_and_a_tag(&repo, "v1.2.3-alpha");
    create_change(&repo);
    add_all_changes_to_index(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "v1.2.3-alpha",
            commits_since_tag: 0,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn after_tag_notmodified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_a_tag_and_some_more_commits(&repo, "v1.2.3-alpha");
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "v1.2.3-alpha",
            commits_since_tag: 5,
            commit_id: &head_commit_id(&repo),
            modified: false,
        }),
    );
}

#[test]
fn after_tag_modified() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_a_tag_and_some_more_commits(&repo, "v1.2.3-alpha");
    create_change(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "v1.2.3-alpha",
            commits_since_tag: 5,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

#[test]
fn after_tag_modified_staged() {
    let project_dir = make_version_test_project();
    let repo = Repository::init(&project_dir).unwrap();
    create_some_commits_a_tag_and_some_more_commits(&repo, "v1.2.3-alpha");
    create_change(&repo);
    add_all_changes_to_index(&repo);
    run_version_test_project(
        project_dir.path(),
        Some(GitInfo {
            tag: "v1.2.3-alpha",
            commits_since_tag: 5,
            commit_id: &head_commit_id(&repo),
            modified: true,
        }),
    );
}

// TODO Test that incremental compiles pick up changes, both changes in the git repo (e.g. create tag) and in the source (e.g. .dirty)

fn head_commit_id(repo: &Repository) -> String {
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    let commit_id = head.id().to_string();
    commit_id[..COMMIT_ID_SHORT_HASH_LENGTH].to_string()
}

fn make_version_test_project() -> TempDir {
    let dir = TempDir::new("package-version-test").unwrap();
    let dir_path = dir.path();
    let path_to_git2version_crate = env!("CARGO_MANIFEST_DIR");

    create_file(
        &dir_path.join("Cargo.toml"),
        &format!(
            r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2021"
name = "package-version-test"
version = "0.1.0"

[workspace]

[dependencies]
version-proxy = {{path = "./version-proxy"}}
serde_json = "^1.0.96"
        "#
        ),
    );

    create_file(
        &dir_path.join("src/main.rs"),
        r#"
fn main() {
    println!("{}", serde_json::to_string(&version_proxy::GITINFO).unwrap());
}
        "#,
    );

    create_file(
        &dir_path.join("version-proxy/Cargo.toml"),
        &format!(
            r#"
[package]
name = "version-proxy"
# The version field here is ignored, no need to change it
version = "0.0.0"

[dependencies]
git2version = {{path = "{path_to_git2version_crate}"}}

[build-dependencies]
git2version = {{path = "{path_to_git2version_crate}", features=["build"]}}
        "#
        ),
    );

    create_file(
        &dir_path.join("version-proxy/build.rs"),
        &format!(
            r#"
fn main() {{
    git2version::init_proxy_build!();
}}
        "#
        ),
    );

    create_file(
        &dir_path.join("version-proxy/src/lib.rs"),
        &format!(
            r#"
            git2version::init_proxy_lib!();
        "#
        ),
    );

    dir
}

fn create_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    File::create(path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
}

fn run_version_test_project(project_dir: &Path, expected_version: Option<GitInfo>) {
    let output = _run_process(
        Command::new(env!("CARGO"))
            .arg("run")
            .current_dir(project_dir),
    );

    let actual_version: Option<GitInfo> = serde_json::from_str(&output).unwrap();
    assert_eq!(expected_version, actual_version);
}

fn _run_process(cmd: &mut Command) -> String {
    let output = cmd.output().unwrap();
    if !output.status.success() {
        panic!(
            "Command {:?} failed with status {:?} and stdin:\n{}\n\nstderr:\n{}",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    String::from_utf8_lossy(&output.stdout).to_string()
}
