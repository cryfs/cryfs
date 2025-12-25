# Jujutsu Command Reference

Complete reference for jj commands organized by category.

## Repository Setup

| Command | Description |
|---------|-------------|
| `jj git init --colocate` | Initialize jj in existing git repo (creates both .jj and .git) |
| `jj git clone <url>` | Clone a git repository |
| `jj git clone --colocate <url>` | Clone with colocated git repo |
| `jj config set --user user.name "Name"` | Set global username |
| `jj config set --user user.email "email"` | Set global email |

## Viewing State

| Command | Description |
|---------|-------------|
| `jj status` / `jj st` | Show working copy status |
| `jj diff` | Show changes in current commit |
| `jj diff -r <rev>` | Show changes in specific revision |
| `jj diff --from <a> --to <b>` | Compare two revisions |
| `jj log` | Show commit graph |
| `jj log -r <revset>` | Show specific revisions |
| `jj log -p` | Show log with patches |
| `jj show <rev>` | Show commit details and diff |
| `jj file list` | List tracked files |
| `jj file show <path>` | Show file contents at revision |
| `jj file annotate <path>` | Show blame/annotate (like git blame) |

## Creating and Modifying Commits

| Command | Description |
|---------|-------------|
| `jj new` | Create new empty commit on top of current |
| `jj new <rev>` | Create new commit on top of specified revision |
| `jj new -m "message"` | Create new commit with description |
| `jj new -A <rev>` | Create new commit after specified revision |
| `jj new -B <rev>` | Create new commit before specified revision |
| `jj commit -m "message"` | Describe current commit and create new empty one |
| `jj describe -m "message"` | Update current commit's description |
| `jj describe -r <rev> -m "msg"` | Update specific commit's description |

## Navigating Commits

| Command | Description |
|---------|-------------|
| `jj edit <rev>` | Switch to editing a specific commit |
| `jj prev` | Move to parent commit |
| `jj next` | Move to child commit |
| `jj next --edit` | Move to child and start editing it |

## Moving and Combining Changes

| Command | Description |
|---------|-------------|
| `jj squash` | Move changes from current commit into parent |
| `jj squash -r <rev>` | Squash specific revision into its parent |
| `jj squash --into <rev>` | Squash current commit into specified revision |
| `jj squash -i` | Interactively select what to squash |
| `jj split` | Split current commit into multiple commits |
| `jj split -r <rev>` | Split specific revision |
| `jj absorb` | Auto-distribute changes to appropriate prior commits |
| `jj diffedit` | Interactively edit the current commit's changes |
| `jj diffedit -r <rev>` | Interactively edit specific revision |

## Rebasing

| Command | Description |
|---------|-------------|
| `jj rebase -d <dest>` | Rebase current commit onto destination |
| `jj rebase -r <rev> -d <dest>` | Rebase specific revision onto destination |
| `jj rebase -s <source> -d <dest>` | Rebase source and descendants onto destination |
| `jj rebase -b <branch> -d <dest>` | Rebase branch and ancestors onto destination |

## Bookmarks (Branches)

| Command | Description |
|---------|-------------|
| `jj bookmark list` | List all bookmarks |
| `jj bookmark create <name>` | Create bookmark at current commit (fails if exists) |
| `jj bookmark set <name>` | Create or update bookmark at current commit |
| `jj bookmark set <name> -r <rev>` | Set bookmark at specific revision |
| `jj bookmark move <name> -r <rev>` | Move existing bookmark to revision |
| `jj bookmark delete <name>` | Delete a bookmark |
| `jj bookmark track <name>@<remote>` | Track a remote bookmark |
| `jj bookmark untrack <name>@<remote>` | Stop tracking remote bookmark |

## Remote Operations

| Command | Description |
|---------|-------------|
| `jj git fetch` | Fetch from all remotes |
| `jj git fetch --remote <name>` | Fetch from specific remote |
| `jj git push` | Push all tracked bookmarks |
| `jj git push --bookmark <name>` | Push specific bookmark |
| `jj git push --bookmark <name> --allow-new` | Push new bookmark (first time) |
| `jj git push --change <rev>` | Push a specific change |
| `jj git push --all` | Push all bookmarks |
| `jj git remote add <name> <url>` | Add a remote |
| `jj git remote list` | List remotes |

## Conflict Resolution

| Command | Description |
|---------|-------------|
| `jj resolve` | Interactively resolve conflicts |
| `jj resolve --list` | List conflicted files |
| `jj resolve <path>` | Resolve specific file |
| `jj resolve --tool=:ours` | Accept "our" side for all conflicts |
| `jj resolve --tool=:theirs` | Accept "their" side for all conflicts |

## Undo and History

| Command | Description |
|---------|-------------|
| `jj undo` | Undo the last jj operation |
| `jj redo` | Redo after an undo |
| `jj op log` | Show operation history |
| `jj op restore <id>` | Restore repo to previous operation state |
| `jj evolog` | Show evolution history of current change |
| `jj evolog -r <rev>` | Show evolution history of specific change |

## Discarding Changes

| Command | Description |
|---------|-------------|
| `jj restore` | Restore working copy to parent state |
| `jj restore <path>` | Restore specific file |
| `jj restore --from <rev>` | Restore from specific revision |
| `jj abandon` | Abandon current commit (descendants remain) |
| `jj abandon <rev>` | Abandon specific revision |

## File Operations

| Command | Description |
|---------|-------------|
| `jj file track <path>` | Explicitly track a file |
| `jj file untrack <path>` | Stop tracking a file |
| `jj file chmod +x <path>` | Make file executable |
| `jj file chmod -x <path>` | Remove executable bit |

## Advanced Commands

| Command | Description |
|---------|-------------|
| `jj duplicate <rev>` | Create copy of commit with new change ID |
| `jj parallelize <revset>` | Convert linear stack to parallel siblings |
| `jj fix` | Run configured formatters/linters on files |
| `jj bisect run '<cmd>'` | Binary search to find bug-introducing commit |
| `jj workspace add <path>` | Create additional working copy |
| `jj sparse set --add <path>` | Sparse checkout - add path |

## Useful Flags

| Flag | Description |
|------|-------------|
| `-r <rev>` | Specify revision (most commands) |
| `-m "message"` | Provide commit message |
| `-p` | Show patch/diff |
| `-s` | Short/summary output |
| `--no-edit` | Skip editor |
| `--at-op <op>` | Run command at specific operation |
