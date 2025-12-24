#!/bin/bash
#
# Configure Claude Code to auto-accept all tool executions in devcontainers.
#
# This script runs on every container start (via postStartCommand) to ensure
# permissions are always configured correctly.
#

set -e

PERMISSIONS='{
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
}'

# User-level settings (always loaded regardless of working directory)
mkdir -p ~/.claude
echo "$PERMISSIONS" > ~/.claude/settings.json

# Clear any cached permission denials from previous sessions
rm -f ~/.claude.json
