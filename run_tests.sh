#!/bin/sh


run_bsd() {
    rsync -avz -e "ssh" --exclude .git --exclude .idea --exclude target --exclude libzfs_core-sys/target --progress /home/andoriyu/dev/github.com/Inner-Heaven/libzetta-rs root@192.168.86.25:/root/ && \
    exec ssh root@192.168.86.25 "set -x; zpool list -H -oname | grep test | xargs zpool destroy; mdconfig -d -u 1; mdconfig -a -s 96m -u1 ;cd /root/libzetta; source ~/.cargo/env; cargo test --color=always $*; mdconfig -d -u1"
}

run_linux() {
    rsync -avz -e "ssh" --exclude .git --exclude .idea --exclude target --exclude libzfs_core-sys/target --progress /home/andoriyu/dev/github.com/Inner-Heaven/libzetta-rs root@192.168.86.171:/root/ && \
    exec ssh root@192.168.86.171 "set -x; zpool list -H -oname | grep test | xargs zpool destroy;cd /root/libzetta; source ~/.cargo/env; cargo test --color=always $*"
}

#cargo check --tests || exit 1
if [ $1 == "linux" ]; then
    shift
    run_linux $*
else
    run_bsd $*
fi

