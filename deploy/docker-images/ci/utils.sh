#!/usr/bin/env bash
set -euxo pipefail

DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  ca-certificates \
  curl \
  wget \
  xz-utils \
  zip \
  unzip \
  git \
  jq \
  xxd \
  imagemagick \
  gnupg2 \
  sudo \
  openssh-client \
  rsync \
  procps \
  gettext-base # envsubst
# Unsafe sudo for easy switching to wallet user
sed -i'' -E '/^Defaults\s+(env_reset|mail_badpass|secure_path)/d' /etc/sudoers

# yq
# Download from: https://github.com/mikefarah/yq/releases/
wget --no-hsts -q -O /usr/local/bin/yq https://github.com/mikefarah/yq/releases/download/v4.52.4/yq_linux_amd64
echo "0c4d965ea944b64b8fddaf7f27779ee3034e5693263786506ccd1c120f184e8c  /usr/local/bin/yq" | sha256sum -c
chmod +x /usr/local/bin/yq

# Minio
# Download from: https://dl.min.io/client/mc/release/linux-amd64/
wget --no-hsts -q -O /usr/local/bin/mc https://dl.min.io/client/mc/release/linux-amd64/archive/mc.RELEASE.2025-08-13T08-35-41Z
echo "01f866e9c5f9b87c2b09116fa5d7c06695b106242d829a8bb32990c00312e891  /usr/local/bin/mc" | sha256sum -c
chmod +x /usr/local/bin/mc

# k8s same version as SP
# Get sha256 by appending .sha256)
wget --no-hsts -q -O /usr/local/bin/kubectl https://dl.k8s.io/release/v1.32.13/bin/linux/amd64/kubectl
echo "db2ae479a63f3665d7f704ab18c0d4d4050144237980763221835b7305703c4c  /usr/local/bin/kubectl" | sha256sum -c
chmod +x /usr/local/bin/kubectl

# Helm
# Download from: https://github.com/helm/helm/releases
wget --no-hsts -q -O helm.tar.gz https://get.helm.sh/helm-v3.20.1-linux-amd64.tar.gz
echo "0165ee4a2db012cc657381001e593e981f42aa5707acdd50658326790c9d0dc3  helm.tar.gz" | sha256sum -c
tar -xf helm.tar.gz --no-same-owner linux-amd64/helm
mv linux-amd64/helm /usr/local/bin/
rmdir linux-amd64
rm helm.tar.gz

# GitHub CLI
# Download from: https://github.com/cli/cli/releases
wget --no-hsts -q -O gh.tar.gz https://github.com/cli/cli/releases/download/v2.88.1/gh_2.88.1_linux_amd64.tar.gz
echo "36352a993b97e9758793cdb87f9ba674bd6d88c914488e122be78a1962203803  gh.tar.gz" | sha256sum -c
tar -xf gh.tar.gz --exclude 'LICENSE' --exclude 'share'
mv gh_*/bin/gh /usr/local/bin/
rm -rf gh_* gh.tar.gz
