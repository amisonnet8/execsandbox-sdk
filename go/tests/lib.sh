# tests/e2e_*.sh から source される共通処理。単体では実行しない。
#
# execsandbox本体をソースから用意する(GitHub Releasesは使わない)。理由は
# PLAN.md「CIの決定事項」参照: 本体はSDKに依存しないが、SDKは本体に依存して
# 実機でABIの往復を検証する、という非対称な依存を許容している。

# build_host_tools <host_repo> <out_dir>
# host_repoにあるexecsandbox本体のソースから、実機疎通確認に必要な
# execsandbox-build(ビルダー)をout_dirへビルドする。
build_host_tools() {
  local host_repo="$1"
  local out_dir="$2"

  echo "tests/lib.sh: building base binaries embedded by execsandbox-build..."
  make -C "$host_repo" cross-base

  echo "tests/lib.sh: building execsandbox-build..."
  (cd "$host_repo" && go build -o "$out_dir/execsandbox-build" ./cmd/execsandbox-build)
}

# tinygo_build_example <example_dir> <out_wasm>
# examples/配下のTinyGoゲストモジュールをwasip1向けにビルドする。
tinygo_build_example() {
  local example_dir="$1"
  local out_wasm="$2"

  (cd "$example_dir" && tinygo build -target=wasip1 -o "$out_wasm" .)
}
