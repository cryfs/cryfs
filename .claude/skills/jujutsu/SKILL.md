---
name: jujutsu
description: Guide to Jujutsu (jj) version control system. Use when working with commits, branches, version control, rebasing, or when the user mentions jj, git, or version control operations.
---

# Jujutsu (jj) Version Control Guide

Jujutsu is a modern, Git-compatible version control system. This project uses jj colocated with git.

## Key Differences from Git

| Concept | Git | Jujutsu |
|---------|-----|---------|
| Staging area | Explicit `git add` | None - working copy IS a commit |
| Branches | Named refs | Bookmarks (auto-follow rewrites) |
| Stash | Separate stash stack | Not needed - just use commits |
| Amend | `git commit --amend` | Just edit files, or use `jj squash` |
| Identity | Commit ID only | Change ID (stable) + Commit ID |

## Essential Commands

| Task | Command |
|------|---------|
| Status | `jj status` or `jj st` |
| Diff | `jj diff` |
| Log | `jj log` |
| Commit & continue | `jj commit -m "message"` |
| Update message | `jj describe -m "message"` |
| New empty commit | `jj new` |
| Squash into parent | `jj squash` |
| Undo last operation | `jj undo` |
| Fetch from remote | `jj git fetch` |
| Push to remote | `jj git push` |

## Working Copy Model

The working copy (`@`) is always a commit. File changes are automatically tracked - no staging required.

```
parent commit
    ↓
@ (working copy) ← your edits go here automatically
```

## Quick Git-to-Jujutsu Translation

| Git | Jujutsu |
|-----|---------|
| `git status` | `jj st` |
| `git diff` | `jj diff` |
| `git log` | `jj log` |
| `git add . && git commit -m "msg"` | `jj commit -m "msg"` |
| `git push` | `jj git push` |
| `git pull` | `jj git fetch` then `jj rebase -d main@origin` |
| `git checkout -b branch` | `jj new main` then `jj bookmark set branch` |
| `git branch` | `jj bookmark list` |
| `git stash` | `jj new` (just start new commit) |
| `git blame` | `jj file annotate` |

## Additional References

- [commands-reference.md](commands-reference.md) - Complete command reference
- [workflows.md](workflows.md) - Common development workflows
- [revsets.md](revsets.md) - Revision selection syntax
