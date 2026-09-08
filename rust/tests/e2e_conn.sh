#!/usr/bin/env bash
# execsandboxクレートのexamples/echoが、SDKのrecv/conn_writeを介して実際の
# 外部接続を正しくエコーバックすることを確認するE2E。本体のtests/e2e_conn.sh
# と同じ構成(本体のtests/connclientでTCP接続してエコーを確かめる)を踏襲する
# (go/tests/e2e_conn.shと同じ、言語がRustというだけの違い)。
#
# 前提: EXECSANDBOX_HOST_REPO 環境変数がexecsandbox本体リポジトリのソース
# チェックアウトを指していること。go・cargo（wasm32-wasip1ターゲット導入
# 済み）がPATH上にあること。
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
# shellcheck source=rust/tests/lib.sh
source "$SCRIPT_DIR/lib.sh"

: "${EXECSANDBOX_HOST_REPO:?EXECSANDBOX_HOST_REPO must point to a checkout of the execsandbox host repository}"

WORKDIR=$(mktemp -d)
PIDS=()
cleanup() {
  if [ "${#PIDS[@]}" -gt 0 ]; then
    for pid in "${PIDS[@]}"; do
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    done
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

PORT=18921

build_host_tools "$EXECSANDBOX_HOST_REPO" "$WORKDIR"

echo "tests/e2e_conn: building connclient..."
(cd "$EXECSANDBOX_HOST_REPO" && go build -o "$WORKDIR/connclient" ./tests/connclient)

echo "tests/e2e_conn: building echo with cargo (wasm32-wasip1)..."
cargo_build_example "$REPO_ROOT/execsandbox" echo "$WORKDIR/echo.wasm"

echo "tests/e2e_conn: stamping with execsandbox-build..."
"$WORKDIR/execsandbox-build" -o "$WORKDIR/echo" "$WORKDIR/echo.wasm"
chmod +x "$WORKDIR/echo"

# -tで期限を切っておくと、connclientとの疎通に万一失敗してもサンドボックス
# 自体は期限内に終了する(孤児プロセス化しにくくする安全網)。
"$WORKDIR/echo" -l "$PORT" -t 20s &
pid_echo=$!
PIDS+=("$pid_echo")

echo "tests/e2e_conn: waiting for the listener and checking the echo..."
ok=0
for _ in $(seq 1 30); do
  if "$WORKDIR/connclient" tcp "127.0.0.1:$PORT" "hello from execsandbox-sdk via conn_write"; then
    ok=1
    break
  fi
  sleep 0.2
done

kill "$pid_echo" >/dev/null 2>&1 || true
wait "$pid_echo" 2>/dev/null || true

if [ "$ok" -ne 1 ]; then
  echo "tests/e2e_conn: FAILED - echo did not respond correctly within the timeout" >&2
  exit 1
fi

echo "tests/e2e_conn: OK - conn_write echoed data back over the real external connection"
