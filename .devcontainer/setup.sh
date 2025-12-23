#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/setup_jj.sh"
"$SCRIPT_DIR/setup_fish.sh"
"$SCRIPT_DIR/setup_claude.sh"
