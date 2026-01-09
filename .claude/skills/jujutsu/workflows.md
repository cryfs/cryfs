# Jujutsu Workflows

Common development workflows and patterns for jj.

## Understanding the Working Copy Model

In jj, there's no staging area. The working copy IS a commit that updates automatically:

```
main
  │
  ├── feature-commit (parent)
  │     │
  │     └── @ (working copy - your current edits)
```

When you edit files, they're immediately part of the working copy commit. This is fundamentally different from git.

## Basic Development Workflow

### Starting New Work

```bash
# Make sure you're up to date
jj git fetch
jj rebase -d main@origin

# Start a new feature from main
jj new main -m "Add new feature"

# Edit files... changes are tracked automatically

# When done, describe your work
jj describe -m "Implement feature X"

# Create new commit for next task
jj new
```

### The Staging Commit Pattern (Recommended)

Work with two commits: a "staging" commit for completed work and the working copy for in-progress changes.

```bash
# Start with a staging commit
jj new main -m "Feature: user authentication"

# Create working copy on top
jj new

# Make changes... edit files

# When a piece is ready, squash into staging
jj squash

# Continue working on next piece
# (working copy is now empty again)

# Repeat: edit, squash, edit, squash...

# When feature is complete, the staging commit has all changes
# Push it
jj git push --bookmark feature-branch
```

### Quick Single Commit

For simple changes that don't need the staging pattern:

```bash
jj new main
# Make your changes
jj commit -m "Fix typo in README"
# This describes the commit AND creates a new empty one
```

## Modifying History

### Editing a Previous Commit

```bash
# Find the commit you want to edit
jj log

# Switch to editing it (use change ID, not commit ID)
jj edit <change-id>

# Make your changes... files update automatically

# Go back to where you were
jj new <where-you-were>
# Or just: jj new
```

All descendant commits are automatically rebased.

### Squashing Commits Together

```bash
# Squash current commit into parent
jj squash

# Squash specific commit into its parent
jj squash -r <change-id>

# Squash current into a specific ancestor
jj squash --into <change-id>

# Interactive squash (select what to move)
jj squash -i
```

### Splitting a Commit

```bash
# Split current commit
jj split

# Opens editor to select which changes go in first commit
# Remaining changes stay in second commit
```

### Reordering Commits

```bash
# Move current commit to be based on different parent
jj rebase -d <new-parent>

# Move a specific commit
jj rebase -r <change-id> -d <new-parent>

# Move commit and all its descendants
jj rebase -s <change-id> -d <new-parent>
```

## Working with Bookmarks (Branches)

### Creating a Feature Branch

```bash
# Start new work
jj new main

# Create bookmark pointing here
jj bookmark set feature/my-feature

# Work on your feature...
jj commit -m "Part 1"
jj commit -m "Part 2"

# Move bookmark to current commit
jj bookmark move feature/my-feature --to @-
```

### Pushing to Remote

```bash
# Create bookmark and push in one command (recommended for new bookmarks)
jj git push --named feature/my-feature=@

# Subsequent pushes
jj git push --bookmark feature/my-feature

# Or push all tracked bookmarks
jj git push
```

## Creating Pull Requests

### Full PR Workflow (Recommended)

```bash
# 1. Ensure your changes are committed
jj log  # Review your commits

# 2. Create bookmark and push in one command
jj git push --named feature/my-feature=@

# 3. Create the PR using GitHub CLI
gh pr create --title "My feature" --body "Description" --base main --head feature/my-feature
```

### Alternative: Create Bookmark First

```bash
# Create bookmark separately
jj bookmark create -r @ feature/my-feature

# Track and push (required for new bookmarks)
jj bookmark track feature/my-feature@origin
jj git push --bookmark feature/my-feature
```

### Key Points

- **--named flag**: Creates bookmark, tracks it, and pushes in one step (recommended)
- **Multiple commits**: The bookmark points to the tip; all ancestor commits up to main are included
- **Subsequent pushes**: Just use `jj git push --bookmark name`

### Updating a PR

```bash
# Make changes to your commits (edit, squash, etc.)
jj edit <change-id>
# ... make changes ...

# Push the updated bookmark
jj git push --bookmark feature/my-feature
```

### Updating from Main

```bash
# Fetch latest
jj git fetch

# Rebase your work onto updated main
jj rebase -d main@origin
```

## Handling Conflicts

Conflicts in jj are first-class citizens - they're stored in commits and don't block operations.

### When Conflicts Occur

```bash
# After a rebase that causes conflicts, jj shows them
jj status
# Shows: Conflicted files: src/main.rs

# Resolve interactively
jj resolve

# Or resolve specific file
jj resolve src/main.rs

# Accept one side entirely
jj resolve --tool=:ours
jj resolve --tool=:theirs
```

### Viewing Conflict Markers

Conflict markers in files look like:

```
<<<<<<< Conflict 1 of 1
%%%%%%% Changes from base to side #1
-old line
+modified in first branch
+++++++ Contents of side #2
modified in second branch
>>>>>>>
```

## Undo and Recovery

### Undoing Operations

```bash
# Undo the last operation
jj undo

# See operation history
jj op log

# Restore to a specific point
jj op restore <operation-id>
```

### Recovering Abandoned Commits

```bash
# Even abandoned commits are recoverable via op log
jj op log

# Find the operation before the abandon
jj op restore <operation-id>
```

## Stacked Changes / Multiple PRs

### Creating a Stack

```bash
# First change
jj new main -m "Refactor: extract helper function"
jj bookmark set refactor-helper
# Make changes...
jj new

# Second change (builds on first)
jj describe -m "Feature: use helper for new feature"
jj bookmark set feature-new
# Make changes...
jj new

# Third change
jj describe -m "Docs: document new feature"
jj bookmark set docs-feature
```

### Pushing Stacked PRs

```bash
# Push each bookmark separately
jj git push --bookmark refactor-helper --allow-new
jj git push --bookmark feature-new --allow-new
jj git push --bookmark docs-feature --allow-new
```

### Updating a Stack After Review

```bash
# Edit the first commit
jj edit <refactor-helper-change-id>

# Make changes...

# All descendants automatically rebase!
# Push updates
jj git push --bookmark refactor-helper
jj git push --bookmark feature-new
jj git push --bookmark docs-feature
```

## Colocated Repository Tips

When using jj colocated with git (`.jj` and `.git` both exist):

```bash
# Git operations still work
git status  # Shows same state
git log     # Shows commits

# But prefer jj commands for changes
jj commit   # Instead of git commit

# Sync if git and jj get out of sync
jj git import
jj git export
```

## Common Patterns Summary

| Goal | Commands |
|------|----------|
| Start feature | `jj new main -m "Feature"` |
| Save progress | `jj commit -m "WIP"` or `jj squash` |
| Fix old commit | `jj edit <id>`, fix, `jj new` |
| Update from main | `jj git fetch && jj rebase -d main@origin` |
| Push feature | `jj bookmark set X && jj git push --bookmark X` |
| Undo mistake | `jj undo` |
