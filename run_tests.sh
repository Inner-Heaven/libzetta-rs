#!/bin/sh

cargo check --tests && \
rsync -avz -e "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null" --exclude target --exclude libzfs_core-sys/target --progress ~/Dev/Heaven/libzfs-rs root@192.168.86.34:/root/ && \
exec ssh root@192.168.86.34 "zpool list -H -oname | grep test | xargs zpool destroy; cd /root/libzfs-rs; env RUST_BACKTRACE=full cargo test $*"

