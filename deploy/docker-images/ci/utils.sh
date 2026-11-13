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
sed -i -E '/^Defaults\s+(env_reset|mail_badpass|secure_path)/d' /etc/sudoers

# yq
# Download from: https://github.com/mikefarah/yq/releases/
wget --no-hsts -q -O /usr/local/bin/yq https://github.com/mikefarah/yq/releases/download/v4.54.3/yq_linux_amd64
echo "129534601871538efe0ba4c5e62a34fb1d5f469bcb975c4fddeb467d82974973  /usr/local/bin/yq" | sha256sum -c
chmod +x /usr/local/bin/yq

# Minio
# MinIO archived mc and dl.min.io no longer serves its binaries; the GitHub releases still do.
wget --no-hsts -q -O /usr/local/bin/mc https://github.com/minio/mc/releases/download/RELEASE.2025-08-13T08-35-41Z/mc.linux-amd64.RELEASE.2025-08-13T08-35-41Z
echo "01f866e9c5f9b87c2b09116fa5d7c06695b106242d829a8bb32990c00312e891  /usr/local/bin/mc" | sha256sum -c
chmod +x /usr/local/bin/mc

# k8s same version as SP
# Get sha256 by appending .sha256)
wget --no-hsts -q -O /usr/local/bin/kubectl https://dl.k8s.io/release/v1.32.13/bin/linux/amd64/kubectl
echo "db2ae479a63f3665d7f704ab18c0d4d4050144237980763221835b7305703c4c  /usr/local/bin/kubectl" | sha256sum -c
chmod +x /usr/local/bin/kubectl

# Helm
# Download from: https://github.com/helm/helm/releases
wget --no-hsts -q -O helm.tar.gz https://get.helm.sh/helm-v3.22.1-linux-amd64.tar.gz
echo "896cf63684a42a723d61c980369d6e588330927deb13676ca0d85006dd6da0ea  helm.tar.gz" | sha256sum -c
tar -xf helm.tar.gz --no-same-owner linux-amd64/helm
mv linux-amd64/helm /usr/local/bin/
rmdir linux-amd64
rm helm.tar.gz

# GitHub CLI
# Download from: https://github.com/cli/cli/releases
wget --no-hsts -q -O gh.tar.gz https://github.com/cli/cli/releases/download/v2.103.3/gh_2.103.3_linux_amd64.tar.gz
echo "4c806978ffce1cec70853302a56d70ac6da4f19bae7adca4e1a8ab00633dd91d  gh.tar.gz" | sha256sum -c
tar -xf gh.tar.gz --exclude 'LICENSE' --exclude 'share'
mv gh_*/bin/gh /usr/local/bin/
rm -rf gh_* gh.tar.gz
gh config set telemetry disabled

# GitLab CLI
# Download from: https://gitlab.com/gitlab-org/cli/-/releases
wget --no-hsts -q -O glab.deb https://gitlab.com/gitlab-org/cli/-/releases/v1.130.0/downloads/glab_1.130.0_linux_amd64.deb
echo "caf19cb4a842da7dbe4ecf146929f44e037fc733ab3f11945a1a48798497ed51  glab.deb" | sha256sum -c
dpkg -i glab.deb
rm glab.deb
