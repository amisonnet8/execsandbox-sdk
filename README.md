# ExecSandbox SDK

[ExecSandbox](https://github.com/amisonnet8/execsandbox) 用のWASMゲスト
モジュールを書くための、各言語向けSDKライブラリ集。

## ExecSandboxとは

WASM実行ランタイムとWASMモジュールを1つの実行ファイルに封じ込める、環境構築
不要のポータブルなサンドボックス実行ツール（Go製、中核は
[wazero](https://github.com/tetratelabs/wazero)）。複数のExecSandbox
インスタンスをメールボックス方式のメッセージングで繋ぎ合わせてシステムを
構成する。本体の詳細は
[execsandboxリポジトリ](https://github.com/amisonnet8/execsandbox)の
`docs/spec/execsandbox_spec_ja.md` を参照。

## このリポジトリの役割

ExecSandboxのゲストモジュール（WASM側）は、ホストが提供する少数のホスト関数
（`send`/`recv`/`conn_write`/`max_frame`）を`import`することで、他の
ExecSandboxインスタンスとメッセージをやり取りする。この呼び出し規約（ABI）は
低レベルの契約であり、素で叩くとポインタ・長さ・バッファ確保を利用者が
直接扱う必要がある。

**このリポジトリは、そのABIの上に各言語ネイティブな皮を被せたSDKライブラリを
提供する。** バッファ確保を隠す、メッセージ種別を列挙型にする、タイムアウトを
言語ネイティブな型（`time.Duration`等）で受ける、といった使い勝手の改善は
ここで行う。

## ABIとの関係

- **ABIの定義そのものはこのリポジトリにはない。** 正は
  [execsandboxリポジトリ](https://github.com/amisonnet8/execsandbox)の
  仕様書§5であり、このリポジトリは参照するだけでコピーしない。
- ABIは後方互換のみ保証される。本体とSDKのバージョンがズレていても、本体が
  新しければ壊れない（本体が古いまま、SDKが新しいホスト関数を使い始めた
  場合のみ、利用者は本体側を先にアップデートする必要がある）。
- 本体のコードへの依存はゼロ。SDKは各言語の外部関数宣言（Goの
  `//go:wasmimport`、Rustの`extern "C"`等）でABIを直接叩くだけで、
  execsandbox本体のパッケージをimportしない。

## 対応言語

TinyGo（Go）→Rustの順で実装予定。TinyGo版は`execsandbox/`ディレクトリに
着手済み（詳細は`PLAN.md`参照）。

## ビルドしたWASMモジュールの使い方

このSDKで書けるのはWASMモジュール（ゲスト）のコードのみ。実際に動かすには、
[execsandbox-build](https://github.com/amisonnet8/execsandbox)（ビルダー）で
ExecSandbox本体へ埋め込み、単一の実行ファイルにする必要がある。

## 現在の状態

TinyGo版SDKの最初の実装（`execsandbox/`、ABIの薄いラッパー）に着手済み。
単体テスト・examples・execsandbox本体を使ったE2Eテスト・GitHub Actions CIも
整備済み。詳細と進捗は`PLAN.md`参照。

## テスト

- `cd execsandbox && go test ./...`：ホストアーキテクチャで動く単体テスト。
- `examples/`：`Send`/`Recv`/`ConnWrite`の最小サンプル。TinyGoで
  `tinygo build -target=wasip1 -o out.wasm .`とビルドできる。
- `tests/e2e_*.sh`：execsandbox本体のソースチェックアウトを使った実機
  疎通テスト。`EXECSANDBOX_HOST_REPO`環境変数で本体リポジトリのパスを
  指定して実行する（例:
  `EXECSANDBOX_HOST_REPO=/path/to/execsandbox tests/e2e_send_recv.sh`）。

## ライセンス

[MIT](LICENSE)
