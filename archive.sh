#!/usr/bin/env bash

TAG=$1
GPGHOMEDIR=$2

git archive --format=tgz "$1" > cryfs-$1.tar.gz
gpg --homedir "$GPGHOMEDIR" --armor --detach-sign cryfs-$1.tar.gz

git archive --format=tar "$1" | xz -9 > cryfs-$1.tar.xz
gpg --homedir "$GPGHOMEDIR" --armor --detach-sign cryfs-$1.tar.xz
