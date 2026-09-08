#!/usr/bin/env bash
set -euo pipefail

# ghcr.io/devcontainers/features/rust（公式feature）はrustup本体と既定の
# ツールチェーンのみを導入する。ExecSandboxゲスト向けのwasm32-wasip1
# ターゲットはこのスクリプトで追加導入する。

rustup target add wasm32-wasip1
rustup target list --installed
