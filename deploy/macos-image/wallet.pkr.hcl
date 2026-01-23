packer {
  required_plugins {
    tart = {
      version = ">= 1.2.0"
      source  = "github.com/cirruslabs/tart"
    }
  }
}

variable "vm_base_name" {
  type = string
  # macos-tahoe-xcode:26.2
  default = "ghcr.io/cirruslabs/macos-tahoe-xcode@sha256:1f7b38002dcfc2927a2e33a405a44867e3d140d92fa295f9e89c018edfe4214a"
}

variable "vm_name" {
  type = string
  default = "tahoe-wallet:0.3.0"
}

source "tart-cli" "tart" {
  vm_base_name = "${var.vm_base_name}"
  vm_name      = "${var.vm_name}"
  cpu_count    = 4
  memory_gb    = 8
  disk_size_gb = 120
  headless     = true
  ssh_password = "admin"
  ssh_username = "admin"
  ssh_timeout  = "120s"
}

build {
  sources = ["source.tart-cli.tart"]

  provisioner "file" {
    source = pathexpand("~/.ssh/id_tart.pub")
    destination = "/Users/admin/.ssh/authorized_keys"
  }

  provisioner "shell" {
    # Switch to same Ruby as CI image
    inline = [
      "source ~/.zprofile",
      "rbenv version | grep -v system | xargs -n1 rbenv uninstall -f",
      "rbenv install 3.3.8",
      "rbenv global 3.3.8"
    ]
  }

  provisioner "shell" {
    # Switch to same Flutter as CI image
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "git -C $FLUTTER_HOME fetch origin",
      "git -C $FLUTTER_HOME switch --detach 3.38.1",
      "dart --disable-analytics",
      "flutter config --no-analytics",
      "yes | sdkmanager --licenses",
      "flutter doctor --android-licenses",
      "flutter precache --ios",
      "dart pub global activate junitreport 2.0.2",
      "echo 'export PATH=\"$HOME/.pub-cache/bin:$PATH\"' >> ~/.zprofile"
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "brew install rustup",
      "rustup-init -y --default-toolchain 1.92.0 --profile minimal --component clippy,rustfmt",
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "cargo install cargo-expand --locked --version 1.0.118",
      "cargo install lcov2xml --locked --version 1.0.6",
      "rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios",
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "brew install kubernetes-cli",
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "brew install minio-mc",
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "brew doctor || true", # brew doctor warns about formulae in taps with same name
      "flutter doctor",
    ]
  }
}
