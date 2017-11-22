#!/bin/sh
rsync -avz -e "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null" --exclude target --exclude libzfs_core-sys/target --progress ~/Dev/Heaven/libzfs-rs root@192.168.86.34:/root/
exec ssh root@192.168.86.34 'cd /root/libzfs-rs; cargo test'
