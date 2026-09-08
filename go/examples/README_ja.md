# サンプル

[`execsandbox`](../execsandbox)パッケージを使った最小限のExecSandbox
ゲストモジュール。ABIの機能ごとに1つずつ用意している：

| サンプル | 内容 |
|---|---|
| [`sender`](sender) | `Send` — 宛先1へ1件メッセージを送って終了 |
| [`receiver`](receiver) | `Recv` — 1件のメッセージを受信するまでブロックし、表示して終了 |
| [`echo`](echo) | `ConnWrite` — 外部接続で受信したデータをそのまま書き戻す |
| [`worker`](worker) | 正の`Recv`タイムアウトでメッセージ間の周期処理を行い、全`Kind`を明示的に分岐する |

## ビルド

各サンプルは独立したGoモジュール。TinyGoでWASIターゲット向けにビルドする：

```
cd sender
tinygo build -target=wasip1 -o sender.wasm .
```

## 実行

出力される`.wasm`はゲストモジュールであり、直接実行するものではない——
ExecSandboxのビルダー（[本体プロジェクト](https://github.com/amisonnet8/execsandbox)の
`execsandbox-build`）で実行ファイルへ埋め込んでから、他のExecSandbox
インスタンスと同様に起動する。CLIの全体像は本体プロジェクトの
`docs/usage/execsandbox.md`を、ビルド・埋め込み・実行までの完全な例は
このリポジトリの[`../tests/e2e_send_recv.sh`](../tests/e2e_send_recv.sh)と
[`../tests/e2e_conn.sh`](../tests/e2e_conn.sh)を参照。

---

For the English version, see [README.md](README.md).
