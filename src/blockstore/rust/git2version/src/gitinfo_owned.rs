use git2::{Repository, StatusOptions, StatusShow};

use super::git_helpers;

use super::COMMIT_ID_SHORT_HASH_LENGTH;

#[derive(Clone, PartialEq, Eq)]
pub struct GitInfoOwned {
    pub tag: String,
    pub commits_since_tag: u32,
    pub commit_id: String,
    pub modified: bool,
}

// TODO Test
pub fn get_git_info(repo: &Repository) -> Result<GitInfoOwned, git2::Error> {
    let head_commit = repo.head()?.peel_to_commit()?;
    let head_commit_id_str = head_commit.id().to_string();
    let head_commit_id_str = head_commit_id_str[..COMMIT_ID_SHORT_HASH_LENGTH].to_string();

    let modified = {
        let statuses = repo.statuses(Some(
            StatusOptions::default()
                .show(StatusShow::IndexAndWorkdir)
                .include_untracked(false)
                .include_ignored(false)
                .include_unmodified(false)
                .exclude_submodules(false),
        ))?;
        statuses.iter().any(|status| {
            status.status() != git2::Status::CURRENT && status.status() != git2::Status::IGNORED
        })
    };

    // find closest ancestor tag, only looking at first parents (i.e. ignoring merge commits)
    // We do this without using `git describe` because the `git describe` format can be ambigious
    // if the version number contains dashes
    let all_tags = git_helpers::all_tags(&repo)?;
    let mut current_commit = head_commit;
    let mut commits_since_tag = 0;
    loop {
        let commit_id = current_commit.id();
        if let Some(tags) = all_tags.get(&commit_id) {
            // TODO Don't just take the first tag, but compare version numbers
            let tag = tags.first().expect(
                "tag list can't be empty, because the `all_tags` HashMap only contains entries that have at least one element",
            );
            return Ok(GitInfoOwned {
                tag: tag.to_string(),
                commits_since_tag,
                commit_id: head_commit_id_str,
                modified,
            });
        }
        match current_commit.parent(0) {
            Ok(parent) => current_commit = parent,
            Err(_) => {
                // We reached the root commit without finding a tag
                return Ok(GitInfoOwned {
                    tag: "".to_string(),
                    commits_since_tag: commits_since_tag + 1,
                    commit_id: head_commit_id_str,
                    modified,
                });
            }
        }
        commits_since_tag += 1;
    }
}
