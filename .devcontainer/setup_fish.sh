#!/bin/bash

set -e
set -v

# Install jujutsu (jj) version control
cargo install --locked --bin jj jj-cli

curl -o /tmp/omf_install https://raw.githubusercontent.com/oh-my-fish/oh-my-fish/master/bin/install 
fish /tmp/omf_install --noninteractive

fish -c "omf install bobthefish"

mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
wget https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.1/Hack.tar.xz
tar -xvf Hack.tar.xz
rm Hack.tar.xz

fish -c 'set -U theme_nerd_fonts yes'

# Set up jj completions for fish
mkdir -p ~/.config/fish/completions
jj util completion fish > ~/.config/fish/completions/jj.fish

# Initialize jujutsu in colocated mode for the workspace
cd /workspaces/cryfs
jj git init --colocate
