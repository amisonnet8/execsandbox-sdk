#!/usr/bin/env bash
set -euo pipefail

# devcontainer feature（ghcr.io/devcontainers-community/features/tinygo）は
# 公式（devcontainers組織）ではない有志コレクションのため使用しない方針
# （2026-09-08決定、PLAN.md参照）。代わりにTinyGo公式配布（tinygo-org/tinygo
# のGitHub Releasesにある.deb）を直接導入する。バージョン更新時はここを
# 書き換えること。
TINYGO_VERSION="0.42.0"

case "$(dpkg --print-architecture)" in
  amd64) ARCH="amd64" ;;
  arm64) ARCH="arm64" ;;
  armhf) ARCH="armhf" ;;
  *)
    echo "install-tinygo.sh: unsupported architecture: $(dpkg --print-architecture)" >&2
    exit 1
    ;;
esac

DEB="tinygo_${TINYGO_VERSION}_${ARCH}.deb"
URL="https://github.com/tinygo-org/tinygo/releases/download/v${TINYGO_VERSION}/${DEB}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

curl -fsSL -o "${TMP_DIR}/${DEB}" "${URL}"
sudo apt-get update
sudo apt-get install -y "${TMP_DIR}/${DEB}"

tinygo version
