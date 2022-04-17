workspace := "~/libzetta-rs"
ubuntu_host := "zetta-ubuntu"
freebsd13_host := "zetta-freebsd13"
rsync_exclude := "--exclude .git --exclude .idea --exclude target --exclude libzfs_core-sys/target"

set positional-arguments

test-freebsd13 args='':
    just delete-test-pool-on {{freebsd13_host}}
    just copy-code-to {{freebsd13_host}}
    -ssh {{freebsd13_host}} "sudo sh -c 'mdconfig -d -u 1; mdconfig -a -s 96m -u1'"
    ssh {{freebsd13_host}} '. "$HOME/.cargo/env";cd {{workspace}} && sudo env PATH=$PATH LD_PRELOAD=/usr/local/lib/libzfs_core.so cargo test {{args}}'

test-ubuntu args='':
    just delete-test-pool-on {{ubuntu_host}}
    just copy-code-to {{ubuntu_host}}
    ssh {{ubuntu_host}} '. "$HOME/.cargo/env";cd {{workspace}} && sudo env PATH=$PATH cargo test {{args}}'

delete-test-pool-on host:
  -ssh {{host}} "sudo sh -c 'zpool list -H -oname | grep test | xargs zpool destroy'"

copy-code-to host:
 rsync -az -e "ssh" {{rsync_exclude}} --progress ./ {{host}}:{{workspace}}


