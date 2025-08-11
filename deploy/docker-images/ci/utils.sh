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
wget --no-hsts -q -O /usr/local/bin/yq https://github.com/mikefarah/yq/releases/download/v4.47.1/yq_linux_amd64
echo "0fb28c6680193c41b364193d0c0fc4a03177aecde51cfc04d506b1517158c2fb  /usr/local/bin/yq" | sha256sum -c
chmod +x /usr/local/bin/yq

# Minio
# Download from: https://dl.min.io/client/mc/release/linux-amd64/
wget --no-hsts -q -O /usr/local/bin/mc https://dl.min.io/client/mc/release/linux-amd64/archive/mc.RELEASE.2025-07-21T05-28-08Z
echo "ea4a453be116071ab1ccbd24eb8755bf0579649f41a7b94ab9e68571bb9f4a1e  /usr/local/bin/mc" | sha256sum -c
chmod +x /usr/local/bin/mc

# k8s same version as SP
# Get sha256 by appending .sha256)
wget --no-hsts -q -O /usr/local/bin/kubectl https://dl.k8s.io/release/v1.30.14/bin/linux/amd64/kubectl
echo "7ccac981ece0098284d8961973295f5124d78eab7b89ba5023f35591baa16271  /usr/local/bin/kubectl" | sha256sum -c
chmod +x /usr/local/bin/kubectl

# Helm
# Download from https://github.com/helm/helm/releases
wget --no-hsts -q -O helm.tar.gz https://get.helm.sh/helm-v3.17.4-linux-amd64.tar.gz
echo "c91e3d7293849eff3b4dc4ea7994c338bcc92f914864d38b5789bab18a1d775d  helm.tar.gz" | sha256sum -c
tar -xf helm.tar.gz --no-same-owner linux-amd64/helm
mv linux-amd64/helm /usr/local/bin/
rmdir linux-amd64
rm helm.tar.gz

# GitHub CLI
wget --no-hsts -q -O gh.tar.gz https://github.com/cli/cli/releases/download/v2.76.2/gh_2.76.2_linux_amd64.tar.gz
echo "62544b0f3759bbf1155c0ac3d75838b5fe23d66dfb75cf8368f84fff8f82b93e  gh.tar.gz" | sha256sum -c
tar -xf gh.tar.gz --exclude 'LICENSE' --exclude 'share'
mv gh_*/bin/gh /usr/local/bin/
rm -rf gh_* gh.tar.gz
