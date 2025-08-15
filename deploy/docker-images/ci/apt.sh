#!/usr/bin/env bash
set -euxo pipefail

# Check APT signatures to cryptographically pin APT to trusted source
[[ $(ls /etc/apt/trusted.gpg.d/ | wc -l) -eq 9 ]]
sha256sum -c <<EOD
c2a9a16fde95e037bafd0fa6b7e31f41b4ff1e85851de5558f19a2a2f0e955e2  /etc/apt/trusted.gpg.d/debian-archive-bookworm-automatic.asc
74f81645b4e3156d1e9a88c8dd9259271b89c7099d64af89a2a6996b592faa1f  /etc/apt/trusted.gpg.d/debian-archive-bookworm-security-automatic.asc
521e9f6a9f9b92ee8d5ce74345e8cfd04028dae9db6f571259d584b293549824  /etc/apt/trusted.gpg.d/debian-archive-bookworm-stable.asc
0b7dc94b880f0b63e2093394b113cafd870badb86e020a35614f49b9d83beb1e  /etc/apt/trusted.gpg.d/debian-archive-bullseye-automatic.asc
716e79393c724d14ecba8be46e99ecbe1b689f67ceff3cb3cab28f6e69e8b8b8  /etc/apt/trusted.gpg.d/debian-archive-bullseye-security-automatic.asc
fb260ce8521a2faa4937d98a29a5347807e10614b97d510fbabe5480c803bda9  /etc/apt/trusted.gpg.d/debian-archive-bullseye-stable.asc
6f1d277429dd7ffedcc6f8688a7ad9a458859b1139ffa026d1eeaadcbffb0da7  /etc/apt/trusted.gpg.d/debian-archive-trixie-automatic.asc
844c07d242db37f283afab9d5531270a0550841e90f9f1a9c3bd599722b808b7  /etc/apt/trusted.gpg.d/debian-archive-trixie-security-automatic.asc
4d097bb93f83d731f475c5b92a0c2fcf108cfce1d4932792fca72d00b48d198b  /etc/apt/trusted.gpg.d/debian-archive-trixie-stable.asc
EOD
