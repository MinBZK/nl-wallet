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
  # macos-sonoma-xcode:16
  default = "ghcr.io/cirruslabs/macos-sonoma-xcode@sha256:e530cb9f8a6db081a29395ef1f27ddfe5c2f28ecaabb88320155e12862b30a81"
}

variable "vm_name" {
  type = string
  default = "sonoma-wallet:0.1.13"
}

source "tart-cli" "tart" {
  vm_base_name = "${var.vm_base_name}"
  vm_name      = "${var.vm_name}"
  cpu_count    = 4
  memory_gb    = 8
  disk_size_gb = 100
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
      "rbenv install 3.1.6",
      "rbenv global 3.1.6"
    ]
  }

  provisioner "shell" {
    # Switch to same Flutter as CI image
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "git -C $FLUTTER_HOME fetch origin",
      "git -C $FLUTTER_HOME switch --detach 3.32.1",
      "dart --disable-analytics",
      "flutter config --no-analytics",
      "flutter doctor --android-licenses",
      "flutter precache",
      "dart pub global activate junitreport 2.0.2",
      "echo 'export PATH=\"$HOME/.pub-cache/bin:$PATH\"' >> ~/.zprofile"
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "set -eux",
      "brew install rustup",
      "rustup-init -y --default-toolchain 1.88.0 --profile minimal --component clippy,rustfmt",
    ]
  }

  provisioner "shell" {
    inline = [
      "source ~/.zprofile",
      "cargo install cargo-expand --version 1.0.113",
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
