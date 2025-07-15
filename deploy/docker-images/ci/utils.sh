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
wget --no-hsts -q -O yq.tar.gz https://github.com/mikefarah/yq/releases/download/v4.45.1/yq_linux_amd64.tar.gz
echo "290b22a62d0bd3590741557eb6391707a519893d81be975637bc13443140e057 yq.tar.gz" | sha256sum -c
tar -xf yq.tar.gz --no-same-owner ./yq_linux_amd64
mv yq_linux_amd64 /usr/local/bin/yq
rm yq.tar.gz

# Minio
# Download from: https://dl.min.io/client/mc/release/linux-amd64/
wget --no-hsts -q -O /usr/local/bin/mc https://dl.min.io/client/mc/release/linux-amd64/archive/mc.RELEASE.2024-11-21T17-21-54Z
echo "0312010a9d0aa7a52b4dfb14330f289ee4e295def80b3a3f530d4f38c1b71da0  /usr/local/bin/mc" | sha256sum -c
chmod +x /usr/local/bin/mc

# k8s same version as SP
# Get sha256 by appending .sha256)
wget --no-hsts -q -O /usr/local/bin/kubectl https://dl.k8s.io/release/v1.29.12/bin/linux/amd64/kubectl
echo "35fc028853e6f5299a53f22ab58273ea2d882c0f261ead0a2eed5b844b12dbfb  /usr/local/bin/kubectl" | sha256sum -c
chmod +x /usr/local/bin/kubectl

# Helm
# Download from https://github.com/helm/helm/releases
wget --no-hsts -q -O helm.tar.gz https://get.helm.sh/helm-v3.16.4-linux-amd64.tar.gz
echo "fc307327959aa38ed8f9f7e66d45492bb022a66c3e5da6063958254b9767d179  helm.tar.gz" | sha256sum -c
tar -xf helm.tar.gz --no-same-owner linux-amd64/helm
mv linux-amd64/helm /usr/local/bin/
rmdir linux-amd64
rm helm.tar.gz

# GitHub CLI
wget --no-hsts -q -O gh.tar.gz https://github.com/cli/cli/releases/download/v2.67.0/gh_2.67.0_linux_amd64.tar.gz
echo "d77623479bec017ef8eebadfefc785bafd4658343b3eb6d3f3e26fd5e11368d5  gh.tar.gz" | sha256sum -c
tar -xf gh.tar.gz --exclude 'LICENSE' --exclude 'share'
mv gh_*/bin/gh /usr/local/bin/
rm -rf gh_* gh.tar.gz
