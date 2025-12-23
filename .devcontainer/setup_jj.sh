#!/bin/bash

set -e
set -v

# Install cargo-binstall if not present
if ! command -v cargo-binstall &> /dev/null; then
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
fi

# Install jujutsu (jj) as prebuilt binary
cargo binstall --no-confirm jj-cli

# Set up jj completions for fish
mkdir -p ~/.config/fish/completions
jj util completion fish > ~/.config/fish/completions/jj.fish

# Set up jj aliases for fish
mkdir -p ~/.config/fish/conf.d
cat > ~/.config/fish/conf.d/jj_aliases.fish << 'EOF'
alias jbm 'jj bookmark move --to @-'
alias jd 'jj diff'
alias jl 'jj log'
alias jn 'jj next'
alias jp 'jj prev'
alias js 'jj st'
EOF

# Initialize jujutsu in colocated mode for the workspace
cd /workspaces/cryfs
jj git init --colocate
