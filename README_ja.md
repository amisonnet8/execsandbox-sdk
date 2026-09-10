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

TinyGo（Go）→Rustの順で実装、両方とも着手済み。言語ごとにリポジトリ直下の
ディレクトリを分ける（TinyGo版は`go/`、Rust版は`rust/`）。詳細は`PLAN.md`
参照。

- TinyGo版：[`go/execsandbox/`](go/execsandbox/)（パッケージ利用者向け
  ドキュメントは[`go/execsandbox/README_ja.md`](go/execsandbox/README_ja.md)、
  サンプルは[`go/examples/`](go/examples/)）。
  [![Go Reference](https://pkg.go.dev/badge/github.com/amisonnet8/execsandbox-sdk/go/execsandbox.svg)](https://pkg.go.dev/github.com/amisonnet8/execsandbox-sdk/go/execsandbox)
- Rust版：[`rust/execsandbox/`](rust/execsandbox/)（パッケージ利用者向け
  ドキュメントは[`rust/execsandbox/README_ja.md`](rust/execsandbox/README_ja.md)、
  サンプルは[`rust/execsandbox/examples/`](rust/execsandbox/examples/)——
  Cargoの組み込みexamples機能を使うため、Go版と違いクレート内に置く）。
  [![Crates.io](https://img.shields.io/crates/v/execsandbox.svg)](https://crates.io/crates/execsandbox)
  [![docs.rs](https://docs.rs/execsandbox/badge.svg)](https://docs.rs/execsandbox)

## ビルドしたWASMモジュールの使い方

このSDKで書けるのはWASMモジュール（ゲスト）のコードのみ。実際に動かすには、
[execsandbox-build](https://github.com/amisonnet8/execsandbox)（ビルダー）で
ExecSandbox本体へ埋め込み、単一の実行ファイルにする必要がある。

## 現在の状態

TinyGo版・Rust版とも、ABIの薄いラッパー実装・単体テスト・examples・
execsandbox本体を使ったE2Eテスト・GitHub Actions CIまで整備済み。詳細と
進捗は`PLAN.md`参照。

## テスト

TinyGo版・Rust版のいずれも、単体テスト（ホストアーキテクチャ）・examples
（wasmビルド確認）・E2Eテスト（execsandbox本体のソースチェックアウトを
使った実機疎通確認）の3段構成。

- TinyGo版：`cd go/execsandbox && go test ./...`、
  `tinygo build -target=wasip1 -o out.wasm .`（各exampleディレクトリで）、
  `EXECSANDBOX_HOST_REPO=/path/to/execsandbox go/tests/e2e_send_recv.sh`。
- Rust版：`cd rust/execsandbox && cargo test`、
  `cargo build --target wasm32-wasip1 --examples`、
  `EXECSANDBOX_HOST_REPO=/path/to/execsandbox rust/tests/e2e_send_recv.sh`。

## ライセンス

[MIT](LICENSE)

---

For the English version, see [README.md](README.md).
