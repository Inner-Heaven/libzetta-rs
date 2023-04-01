# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.define "zetta-ubuntu" do |c|
    c.vm.box = "generic/ubuntu2004"
    c.vm.hostname = "zetta-ubuntu"
    c.vm.provision "shell", inline: <<-SHELL
      apt-get update
      apt-get install -y libblkid-dev libattr1-dev libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc binutils-dev libiberty-dev zlib1g-dev libssl-dev curl rsync
      apt-get update
      apt-get install -y zfsutils-linux libnvpair1linux libzfslinux-dev pkg-config
    SHELL
  end

  config.vm.define "zetta-freebsd13" do |c|
    c.vm.box = "generic/freebsd13"
    c.vm.hostname = "zetta-freebsd13"
    c.vm.provision "shell", inline: <<-SHELL
      env ASSUME_ALWAYS_YES=YES pkg install curl pkgconf rsync openzfs
    SHELL
  end

  config.vm.define "zetta-freebsd14" do |c|
    c.vm.box = "generic/freebsd14"
    c.vm.hostname = "zetta-freebsd14"
    c.vm.provision "shell", inline: <<-SHELL
      env ASSUME_ALWAYS_YES=YES pkg install curl pkgconf rsync
    SHELL
  end

  config.vm.box_check_update = false

  config.vm.provision "shell", privileged: false, inline: <<-SHELL
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    . "$HOME/.cargo/env"

    rustup install stable
  SHELL
end
