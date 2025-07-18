#!/usr/bin/env bash
set -euxo pipefail

# Check APT signatures to cryptographically pin APT to trusted source
[[ $(ls /etc/apt/trusted.gpg.d/ | wc -l) -eq 9 ]]
sha256sum <<EOD
c2a9a16fde95e037bafd0fa6b7e31f41b4ff1e85851de5558f19a2a2f0e955e2  /etc/apt/trusted.gpg.d/debian-archive-bookworm-automatic.asc
74f81645b4e3156d1e9a88c8dd9259271b89c7099d64af89a2a6996b592faa1f  /etc/apt/trusted.gpg.d/debian-archive-bookworm-security-automatic.asc
521e9f6a9f9b92ee8d5ce74345e8cfd04028dae9db6f571259d584b293549824  /etc/apt/trusted.gpg.d/debian-archive-bookworm-stable.asc
0b7dc94b880f0b63e2093394b113cafd870badb86e020a35614f49b9d83beb1e  /etc/apt/trusted.gpg.d/debian-archive-bullseye-automatic.asc
716e79393c724d14ecba8be46e99ecbe1b689f67ceff3cb3cab28f6e69e8b8b8  /etc/apt/trusted.gpg.d/debian-archive-bullseye-security-automatic.asc
fb260ce8521a2faa4937d98a29a5347807e10614b97d510fbabe5480c803bda9  /etc/apt/trusted.gpg.d/debian-archive-bullseye-stable.asc
9c854992fc6c423efe8622c3c326a66e73268995ecbe8f685129063206a18043  /etc/apt/trusted.gpg.d/debian-archive-buster-automatic.asc
4cf886d6df0fc1c185ce9fb085d1cd8d678bc460e6267d80a833d7ea507a0fbd  /etc/apt/trusted.gpg.d/debian-archive-buster-security-automatic.asc
ca9bd1a0b3743495ae45693c6d4e54abadcffb242d72df15eda5b28e4ff385fa  /etc/apt/trusted.gpg.d/debian-archive-buster-stable.asc
EOD
