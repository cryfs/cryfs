#!/bin/bash
#
# Configure Claude Code to auto-accept all tool executions in devcontainers.
#
# This creates .claude/settings.local.json which is gitignored, so it only
# affects devcontainer environments. Local development retains manual approval
# for security.
#

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

mkdir -p "$REPO_ROOT/.claude"
cat > "$REPO_ROOT/.claude/settings.local.json" << 'EOF'
{
  "permissions": {
    "allow": [
      "Bash(*)",
      "Edit(*)",
      "Write(*)",
      "Read(*)",
      "WebFetch(*)",
      "NotebookEdit(*)"
    ]
  }
}
EOF
