#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/setup_jj.sh"
"$SCRIPT_DIR/setup_fish.sh"
# Claude setup runs via postStartCommand (every container start, not just creation)
