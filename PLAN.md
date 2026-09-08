# PLAN.md — execsandbox-sdk 実装計画

## 現在地

**TinyGo版SDKの最初の実装（ABIの薄いラッパー）を`execsandbox/`ディレクトリに
作成した。** `Send`/`Recv`/`ConnWrite`/`MaxFrame`の4関数と、`Kind`/`Message`型を
実装済み。TinyGo 0.42.0で`-target=wasip1`ビルドが通ることを一時的なmain
パッケージ経由で確認し、生成されたwasm内に4つのABI関数名
（`send`/`recv`/`conn_write`/`max_frame`、モジュール名`execsandbox`）が
実際に埋め込まれていることも確認済み。

**execsandbox本体のビルダーでの実機疎通確認も完了した。** 本体リポジトリを
`~/reference/execsandbox`（このリポジトリの外、gitには含まれない）にcloneし、
ソースから`cmd/execsandbox-build`（ビルダー）と6環境分のベースバイナリを
ビルド。`execsandbox/`パッケージを使うだけのTinyGoゲストモジュールを3つ
（送信専用nodeA、受信専用nodeB、`--max-frame`確認用、外部接続echo用）新規に
書いてwasip1向けにビルドし、ビルダーでスタンプして実際に実行。

- `Send`→`Recv`: nodeAが送った文字列をnodeBが`Kind`・`Data`込みで正しく
  受信できることを確認（本体の`tests/e2e_basic.sh`と同じ観測手段）。
- `MaxFrame`: 既定値`1048576`（1M）、`--max-frame 2M`指定時`2097152`と、
  仕様書§7.3の解釈通りの値が返ることを確認。
- `ConnWrite`: 本体の`tests/connclient`から`-l`で待ち受けたサンドボックスへ
  TCP接続し、SDKの`ConnWrite`で書き戻したデータが正しくエコーされることを
  確認（本体の`tests/e2e_conn.sh`と同じ構成）。

**単体テストとGitHub Actions CIも整備した。** 検証用コードは使い捨てず、
以下の形で恒久化した。

- `execsandbox/execsandbox_test.go`：`hostSend`/`hostRecv`/`hostConnWrite`/
  `hostMaxFrame`をパッケージ変数化し（`execsandbox/abi.go`は`//go:build wasm`
  で隔離、wasmビルド時のみ`init()`で実ABIを束縛）、フェイクのhost関数で
  タイムアウト変換・バッファ再確保・未知kindスキップ等のロジックを
  ホストアーキテクチャ上で単体テストできるようにした。**TinyGoは
  `//go:wasmimport`関数を値として直接使えない**（`cannot use an exported
  function as value`）ため、`abi.go`の`init()`では実ABI関数を無名関数で
  ラップしてから変数へ代入している（要注意点）。
- `examples/{sender,receiver,echo}`：手元検証で使ったTinyGoゲストモジュールを
  そのまま公開サンプルとして恒久化（それぞれ`Send`単発、`Recv`単発、
  `ConnWrite`によるエコー）。
- `tests/{lib.sh,e2e_send_recv.sh,e2e_conn.sh}`：`EXECSANDBOX_HOST_REPO`
  環境変数が指す本体ソースチェックアウトから`execsandbox-build`等を都度
  ビルドし、上記examplesを実際に実行して疎通を確認する（本体の
  `tests/e2e_basic.sh`・`tests/e2e_conn.sh`と同じ構成）。手元では
  `EXECSANDBOX_HOST_REPO=~/reference/execsandbox`で実行確認済み。
- `.github/workflows/ci.yml`：`unit-test`（gofmt/vet/test）、`wasm-build`
  （TinyGoで全examplesをビルド）、`integration`（本体をタグ固定でcheckoutし
  上記e2eスクリプトを実行）の3ジョブ。詳細は次節「テスト・CIの決定事項」参照。

次は「次にやること」の4番（TinyGo版が一段落したらRustへ着手）に進む前に、
現時点の`execsandbox/`パッケージ自体をこのまま完成形とするか、他に
追加すべき使い勝手（ドキュメント等）がないか、ユーザーと相談すること。
（2026-09-08、execsandbox本体のフェーズ①〜④完了後に着手）

### ディレクトリ構成の決定事項（2026-09-08）

**言語ごとにリポジトリ直下のディレクトリを分ける。** Go/TinyGo版は
`execsandbox/`（`go.mod`のモジュール名は
`github.com/amisonnet8/execsandbox-sdk/execsandbox`、パッケージ名も
`execsandbox`でディレクトリ名と一致させ、importの別名指定が不要になるように
した）。Rust着手時は同様に`rust/`等、言語名で切る想定（着手時に改めて決定）。

このパッケージはwasmアーキテクチャ向けにしかビルドできない
（`//go:wasmimport`が非wasmターゲットでは受理されないため）。ビルドタグは
付けていない。TinyGoの`-target=wasip1`、または標準Goでも
`GOOS=wasip1 GOARCH=wasm go build`でビルド・型チェックできる。

### テスト・CIの決定事項（2026-09-08）

**「本体はSDKに依存しないが、SDKは本体に依存して実機で試験してよい」という
非対称な依存を許容する。** `.claude/rules/abi-boundary.md`の「本体のコードへ
依存しない」は**SDKの本番コード（go.mod依存・import）**の話であり、
テスト・CIが検証目的で本体のソースを利用することは妨げない（ユーザー確認
済み）。実際、`execsandbox/`パッケージ自体のgo.modには本体への依存は一切
追加していない。

- **本体はGitHub Releasesではなくソースからビルドする。** `~/reference/
  execsandbox`への手元clone、CIでは`actions/checkout`の`repository:`で別
  リポジトリとしてcheckoutし、`make cross-base && go build ./cmd/
  execsandbox-build`でビルダーを都度用意する。
- **本体のバージョンはタグで固定する**（`.github/workflows/ci.yml`の
  `EXECSANDBOX_HOST_REF`、現在`v0.1.0`）。本体のmain更新でSDK側CIが
  無関係な理由で壊れることを避けるため。本体の新しいタグを追随したく
  なったら、ここを書き換える。
- E2Eスクリプト（`tests/*.sh`）は本体の`tests/e2e_basic.sh`・
  `tests/e2e_conn.sh`と同じ構成・観測手段（プロセス終了・標準出力・
  `tests/connclient`）を踏襲する。
- **Unixソケットパス長の落とし穴**：`XDG_RUNTIME_DIR`を`mktemp -d`の既定
  パス（scratchpad配下等、長くなりがち）に置くと`bind: invalid argument`
  で失敗することがある（108バイト前後の制限）。`tests/e2e_send_recv.sh`は
  `/tmp`直下に短い一時ディレクトリを別途作ってこれに充てている。

この構成が「同じ種類の判断や落とし穴」として今後も繰り返されるようなら、
`.claude/rules/testing.md`の新設をあらためて提案する
（CLAUDE.mdの方針により、ここでは提案に留め勝手に作成しない）。

### devcontainer feature選定の決定事項（2026-09-08）

**devcontainer featureは公式（`devcontainers`組織、`ghcr.io/devcontainers/
features/*`）のみ使用する方針。** 公式featureが存在しないツールは、
devcontainer feature（有志コレクション等）に頼らず、そのツール自身の公式
配布物を`postCreateCommand`で直接導入する。

- 経緯: 当初`ghcr.io/devcontainers-community/features/tinygo`（有志運営、
  非公式）を使ってリビルドし失敗した。原因の切り分け中にこの方針を採用する
  ことにした。
- 対応: Goは公式feature（`ghcr.io/devcontainers/features/go`）で導入。
  TinyGoは公式featureが存在しないため、`.devcontainer/install-tinygo.sh`が
  TinyGo公式配布（tinygo-org/tinygoのGitHub Releasesの`.deb`）を直接導入する。
- 今後Rust着手時にfeaturesへ追記する際も、この方針（公式のみ）に従うこと。

## このプロジェクトについて

execsandbox本体が提供するWASM ABIの上に、各言語ネイティブなSDKライブラリを
提供する。詳細は`README.md`参照。ABIの定義自体は本体リポジトリ
（execsandbox）の仕様書§5にあり、このリポジトリでは持たない
（`.claude/rules/abi-boundary.md`）。

## 対応言語（決定事項）

**TinyGo（Go）を最初に実装し、その後Rustに着手する。**
（2026-09-08決定）

- godocコメント言語: **英語のみ**（2026-09-08決定）。
- rustdocコメント言語: 未決定。Rust着手時に改めて判断する。

## 保留事項

- **パッケージマネージャへの公開方法。** 各言語の実装着手時に決める
  （CI構成は上記「テスト・CIの決定事項」により決着済み）。

## 次にやること

1. ~~TinyGoのツールチェーンをdevcontainerへ追加する。~~ 完了
   （Goは公式feature`ghcr.io/devcontainers/features/go`で、TinyGoは
   `.devcontainer/install-tinygo.sh`（postCreateCommand）でTinyGo公式配布を
   直接導入する構成にした。**devcontainerのリビルドが必要**）。
2. ~~TinyGo版SDKの最初の実装（ABIの薄いラッパーから）。~~ 完了
   （`execsandbox/`に`Send`/`Recv`/`ConnWrite`/`MaxFrame`を実装。
   TinyGoでのwasip1ビルド疎通は一時的なmainパッケージで確認済み）。
3. ~~execsandbox本体のビルダーで実際にWASMモジュールをビルド・実行して疎通
   確認する。~~ 完了（GitHub Releasesではなく、`~/reference/execsandbox`に
   cloneした本体をソースからビルドして使用。詳細は「現在地」参照）。
4. TinyGo版が一段落したらRustに着手する（rustdocコメント言語をその時点で
   改めて判断）。

## 参考: execsandbox本体との役割分担

| リポジトリ | 役割 |
| :--- | :--- |
| execsandbox | ホスト本体・ビルダー・ABI仕様の正 |
| execsandbox-sdk（本リポジトリ） | ゲスト側SDKライブラリ（全言語） |
