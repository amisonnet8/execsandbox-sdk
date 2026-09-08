#!/usr/bin/env bash
# examples/sender・examples/receiverが、SDKの Send/Recv を介して実際の
# execsandbox本体と正しく疎通することを確認するE2E。
#
# 本体のtests/e2e_basic.shと同じ構成・観測手段を踏襲する: senderは1回だけ
# 送って終了する送信専用サンドボックスとして起動し、receiverが受信できるまで
# senderをリトライ起動する(宛先未起動時のsendは仕様上無言で捨てられるため)。
# receiverはメッセージを受信すると標準出力に書いて終了するので、その出力を
# 検証する(本体のhost_probeと違い、-s outで中身まで確認する)。
#
# 前提: EXECSANDBOX_HOST_REPO 環境変数がexecsandbox本体リポジトリのソース
# チェックアウトを指していること。go・tinygoがPATH上にあること。
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
# shellcheck source=tests/lib.sh
source "$SCRIPT_DIR/lib.sh"

: "${EXECSANDBOX_HOST_REPO:?EXECSANDBOX_HOST_REPO must point to a checkout of the execsandbox host repository}"

WORKDIR=$(mktemp -d)
# Unixソケットのパス長制限(108バイト前後)に収まるよう、待ち受けソケットは
# 別途短い一時ディレクトリに置く(mktemp -dの既定パスは長すぎることがある)。
SHORT_XDG=$(mktemp -d /tmp/execsandbox-sdk-e2e.XXXXXX)
PIDS=()
cleanup() {
  if [ "${#PIDS[@]}" -gt 0 ]; then
    for pid in "${PIDS[@]}"; do
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    done
  fi
  rm -rf "$WORKDIR" "$SHORT_XDG"
}
trap cleanup EXIT

export XDG_RUNTIME_DIR="$SHORT_XDG"

build_host_tools "$EXECSANDBOX_HOST_REPO" "$WORKDIR"

echo "tests/e2e_send_recv: building sender/receiver with TinyGo..."
tinygo_build_example "$REPO_ROOT/examples/sender" "$WORKDIR/sender.wasm"
tinygo_build_example "$REPO_ROOT/examples/receiver" "$WORKDIR/receiver.wasm"

echo "tests/e2e_send_recv: stamping with execsandbox-build..."
"$WORKDIR/execsandbox-build" -o "$WORKDIR/nodeA" "$WORKDIR/sender.wasm"
"$WORKDIR/execsandbox-build" -o "$WORKDIR/nodeB" "$WORKDIR/receiver.wasm"
chmod +x "$WORKDIR/nodeA" "$WORKDIR/nodeB"

"$WORKDIR/nodeB" -n nodeB -s out >"$WORKDIR/nodeb.out" 2>"$WORKDIR/nodeb.err" &
pid_b=$!
PIDS+=("$pid_b")

echo "tests/e2e_send_recv: waiting for sender -> receiver delivery..."
delivered=0
for _ in $(seq 1 30); do
  if ! kill -0 "$pid_b" >/dev/null 2>&1; then
    delivered=1
    break
  fi
  "$WORKDIR/nodeA" -d 1=nodeB || true
  sleep 0.2
done

if [ "$delivered" -ne 1 ]; then
  echo "tests/e2e_send_recv: FAILED - receiver did not receive within the timeout" >&2
  echo "--- receiver stderr ---" >&2
  cat "$WORKDIR/nodeb.err" >&2
  exit 1
fi
wait "$pid_b" 2>/dev/null || true

want="kind=0 data=hello from execsandbox-sdk"
got=$(cat "$WORKDIR/nodeb.out")
if [ "$got" != "$want" ]; then
  echo "tests/e2e_send_recv: FAILED - receiver printed %q, want %q" >&2
  printf 'got:  %s\n' "$got" >&2
  printf 'want: %s\n' "$want" >&2
  exit 1
fi

echo "tests/e2e_send_recv: OK - Send/Recv round-tripped through the real host"
