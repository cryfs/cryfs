# Jujutsu Revsets

Revsets are expressions for selecting revisions. Used with `-r` flag in most commands.

## Basic Selectors

| Revset | Description |
|--------|-------------|
| `@` | Working copy (current commit) |
| `@-` | Parent of working copy |
| `@--` | Grandparent of working copy |
| `@-n` | N-th ancestor of working copy |
| `root()` | The root commit (empty, virtual) |

## Identifying Commits

| Revset | Description |
|--------|-------------|
| `abc123` | Commit or change ID prefix |
| `main` | Bookmark name |
| `main@origin` | Remote bookmark |
| `tags()` | All tagged commits |
| `v1.0` | Specific tag |

### Change ID vs Commit ID

- **Change ID**: Stable across rewrites (e.g., `kmqrlpok`)
- **Commit ID**: Changes when commit is modified (e.g., `a1b2c3d4`)

Use change IDs when referring to commits you might edit.

## Ancestry Operators

| Revset | Description |
|--------|-------------|
| `x-` | Parent of x |
| `x+` | Children of x |
| `x--` | Grandparent of x |
| `x++` | Grandchildren of x |
| `::x` | Ancestors of x (inclusive) |
| `x::` | Descendants of x (inclusive) |
| `x::y` | x to y (DAG range) |
| `x..y` | Commits reachable from y but not x |

## Set Operations

| Revset | Description |
|--------|-------------|
| `x & y` | Intersection (commits in both) |
| `x | y` | Union (commits in either) |
| `x ~ y` | Difference (in x but not y) |
| `~x` | Negation (all commits except x) |

## Functions

### Ancestry Functions

| Function | Description |
|----------|-------------|
| `parents(x)` | Direct parents of x |
| `children(x)` | Direct children of x |
| `ancestors(x)` | All ancestors (same as `::x`) |
| `descendants(x)` | All descendants (same as `x::`) |
| `heads(x)` | Commits in x with no descendants in x |
| `roots(x)` | Commits in x with no ancestors in x |

### Bookmark Functions

| Function | Description |
|----------|-------------|
| `bookmarks()` | All local bookmarks |
| `bookmarks(pattern)` | Bookmarks matching glob pattern |
| `remote_bookmarks()` | All remote bookmarks |
| `remote_bookmarks(pattern)` | Remote bookmarks matching pattern |
| `tracked_remote_bookmarks()` | Tracked remote bookmarks |

### Special Sets

| Function | Description |
|----------|-------------|
| `trunk()` | Main branch (main, master, or trunk) |
| `tags()` | All tagged commits |
| `git_refs()` | All git references |
| `git_head()` | Git's HEAD |
| `visible_heads()` | All visible head commits |
| `all()` | All commits |
| `none()` | Empty set |

### State Functions

| Function | Description |
|----------|-------------|
| `empty()` | Commits with no changes |
| `conflict()` | Commits with conflicts |
| `description(pattern)` | Commits whose description matches |
| `author(pattern)` | Commits by matching author |
| `committer(pattern)` | Commits by matching committer |
| `file(path)` | Commits touching file/directory |
| `diff_contains(pattern)` | Commits whose diff contains text |

### Immutability

| Function | Description |
|----------|-------------|
| `immutable()` | Commits that can't be modified |
| `mutable()` | Commits that can be modified |
| `immutable_heads()` | Boundary between mutable/immutable |

## Common Examples

### View Recent Work

```bash
# Your commits not yet on main
jj log -r "main..@"

# All commits since diverging from origin
jj log -r "trunk()..@"
```

### Find Specific Commits

```bash
# Commits touching a file
jj log -r "file('src/main.rs')"

# Commits by you
jj log -r "author('your-email')"

# Commits with 'fix' in message
jj log -r "description('fix')"

# Commits with conflicts
jj log -r "conflict()"
```

### Branch Operations

```bash
# All feature branches
jj bookmark list -r "bookmarks('feature/*')"

# Commits on feature branch not on main
jj log -r "main..feature/x"
```

### Working with Stacks

```bash
# My full stack from main
jj log -r "main::@"

# Just my commits (not main itself)
jj log -r "main..@"

# Children of current commit
jj log -r "@+"
```

### Rebasing Selections

```bash
# Rebase everything since main
jj rebase -s "roots(main..@)" -d main@origin

# Rebase specific range
jj rebase -s <start> -d <dest>
```

## Revset in Practice

```bash
# Log with revset
jj log -r "trunk()..@ | @+"

# Diff range
jj diff --from main --to @

# Operate on multiple commits
jj describe -r "empty() & mine()" -m "WIP"

# Squash range
jj squash --from "@--" --into "@-"
```

## Tips

1. Use `jj log -r "all()"` sparingly - can be huge
2. Prefer change IDs over commit IDs for refs that might change
3. `trunk()` auto-detects main/master/trunk
4. Combine with `-p` to see patches: `jj log -r "file('x')" -p`
5. Test revsets with `jj log -r "<revset>"` before using in other commands
