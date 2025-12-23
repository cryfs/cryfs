#!/bin/bash

set -e
set -v

# Remove the broken git-lfs repository that causes apt-get update to fail
# See: https://github.com/git-lfs/git-lfs/issues/5893
sudo rm -f /etc/apt/sources.list.d/github_git-lfs.list

# Add fish shell PPA and install fish
sudo apt-add-repository -y ppa:fish-shell/release-3
sudo apt-get update
sudo apt-get install -y fish fuse3 libfuse3-dev

curl -o /tmp/omf_install https://raw.githubusercontent.com/oh-my-fish/oh-my-fish/master/bin/install 
fish /tmp/omf_install --noninteractive

fish -c "omf install bobthefish"

mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
wget https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.1/Hack.tar.xz
tar -xvf Hack.tar.xz
rm Hack.tar.xz

fish -c 'set -U theme_nerd_fonts yes'
