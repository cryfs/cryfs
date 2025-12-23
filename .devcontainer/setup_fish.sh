#!/bin/bash

set -e
set -v

curl -o /tmp/omf_install https://raw.githubusercontent.com/oh-my-fish/oh-my-fish/master/bin/install
fish /tmp/omf_install --noninteractive

fish -c "omf install bobthefish"

mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
wget https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.1/Hack.tar.xz
tar -xvf Hack.tar.xz
rm Hack.tar.xz

fish -c 'set -U theme_nerd_fonts yes'
