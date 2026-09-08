# tests/e2e_*.sh から source される共通処理。単体では実行しない。
#
# execsandbox本体をソースから用意する(GitHub Releasesは使わない)。理由は
# PLAN.md「CIの決定事項」参照: 本体はSDKに依存しないが、SDKは本体に依存して
# 実機でABIの往復を検証する、という非対称な依存を許容している。
# go/tests/lib.shと同じ内容だが、rust/はgo/に依存させず自己完結させる方針
# のためあえて重複させている(PLAN.md「ディレクトリ構成の決定事項」参照)。

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

# cargo_build_example <crate_dir> <example_name> <out_wasm>
# execsandboxクレートのexamples/配下をwasm32-wasip1向けにビルドし、
# 生成された.wasmをout_wasmへコピーする。
cargo_build_example() {
  local crate_dir="$1"
  local example_name="$2"
  local out_wasm="$3"

  (cd "$crate_dir" && cargo build --target wasm32-wasip1 --example "$example_name")
  cp "$crate_dir/target/wasm32-wasip1/debug/examples/$example_name.wasm" "$out_wasm"
}
