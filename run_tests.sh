#!/bin/sh
cargo check --tests && \
rsync -avz -e "ssh" --exclude .git --exclude .idea --exclude target --exclude libzfs_core-sys/target --progress ~/Dev/Heaven/libzfs-rs root@192.168.86.163:/root/ && \
exec ssh root@192.168.86.163 "set -x; zpool list -H -oname | grep test | xargs zpool destroy; mdconfig -d -u 1; mdconfig -a -s 96m -u1 ;cd /root/libzfs-rs; source ~/.cargo/env; cargo test --color=always $*; mdconfig -d -u1"
