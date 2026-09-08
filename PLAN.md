# PLAN.md — execsandbox-sdk 実装計画

## 現在地

**TinyGo版SDKは実装・単体テスト・実機疎通・CI・ドキュメントまで一区切り
ついた状態。** 詳細は以下。

- **実装**：`go/execsandbox/`に`Send`/`Recv`/`ConnWrite`/`MaxFrame`と
  `Kind`/`Message`型。TinyGoの`//go:wasmimport`はwasm以外のターゲットでは
  受理されないため`go/execsandbox/abi.go`を`//go:build wasm`で隔離し、
  `hostSend`等をパッケージ変数化して`init()`で実ABIへ束縛する構造（詳細は
  「テスト・CIの決定事項」）。**TinyGoは`//go:wasmimport`関数を値として
  直接使えない**（`cannot use an exported function as value`）ため、
  `init()`では無名関数でラップしてから変数へ代入している（要注意点）。
- **単体テスト**：`go/execsandbox/execsandbox_test.go`。フェイクのhost関数で
  タイムアウト変換・バッファ再確保・未知kindスキップ等をホスト
  アーキテクチャ上で検証。`go/execsandbox/example_test.go`に
  `ExampleSend`等（pkg.go.dev向け、`Output:`無しで実行はされない）。
- **実機疎通・E2E**：`go/examples/{sender,receiver,echo}`（公開サンプル
  兼E2E用フィクスチャ）と`go/tests/{lib.sh,e2e_send_recv.sh,e2e_conn.sh}`。
  `EXECSANDBOX_HOST_REPO`環境変数が指す本体ソースチェックアウトから
  `execsandbox-build`等を都度ビルドし、本体の`tests/e2e_basic.sh`・
  `tests/e2e_conn.sh`と同じ構成・観測手段でSend/Recv/ConnWrite/MaxFrameの
  実機疎通を確認する（`~/reference/execsandbox`で手元確認済み）。
- **CI**：`.github/workflows/ci.yml`の`unit-test`（gofmt/vet/test）、
  `wasm-build`（TinyGoで全examplesをビルド）、`integration`（本体をタグ
  固定でcheckoutし上記e2eスクリプトを実行）の3ジョブ。詳細は「テスト・CIの
  決定事項」参照。初回コミット（`8e12bab`）をpushしGitHub Actionsで3ジョブ
  とも成功（green）を確認済み。
- **ドキュメント**：`go/execsandbox/`はそれ自体が独立したGoモジュールで
  あり、pkg.go.devはリポジトリ直下ではなく**モジュール直下**のREADMEを
  表示するため、`go/execsandbox/README.md`（英語、理由は「対応言語」節）を
  用意。`go/examples/README.md`にサンプルの一覧とビルド・実行方法。

次は「次にやること」の4番（TinyGo版が一段落したらRustへ着手）へ進める
段階。（2026-09-08、execsandbox本体のフェーズ①〜④完了後に着手）

### ディレクトリ構成の決定事項（2026-09-08）

**言語ごとにリポジトリ直下のディレクトリを分ける。** Go/TinyGo版は
`go/`直下に、パッケージ本体（`go/execsandbox/`）・サンプル
（`go/examples/`）・E2Eテスト（`go/tests/`）をまとめる。Rust着手時は同様に
`rust/`（`rust/execsandbox/`・`rust/examples/`・`rust/tests/`）を追加する
想定。

- **経緯**：当初はGo/TinyGo版だけが対象という理由で`execsandbox/`・
  `examples/`・`tests/`をリポジトリ直下に直接置いていたが、Rust追加時に
  `execsandbox/`（Go、直下）と`rust/`（Rust、直下にネスト）が非対称になる
  点をユーザーに指摘され、`go/`配下へ移動した（2026-09-08）。
- Goモジュール名は`github.com/amisonnet8/execsandbox-sdk/go/execsandbox`。
  パッケージ名`execsandbox`とディレクトリ名を一致させ、importの別名指定が
  不要になるようにしている（ディレクトリ名を`go`一段のみにしないのは、
  Goの予約語`go`はパッケージ名にできないため）。

`go/execsandbox/`はwasmアーキテクチャ向けにしかビルドできない
（`//go:wasmimport`が非wasmターゲットでは受理されないため）。ビルドタグは
付けていない。TinyGoの`-target=wasip1`、または標準Goでも
`GOOS=wasip1 GOARCH=wasm go build`でビルド・型チェックできる。

### テスト・CIの決定事項（2026-09-08）

**「本体はSDKに依存しないが、SDKは本体に依存して実機で試験してよい」という
非対称な依存を許容する。** `.claude/rules/abi-boundary.md`の「本体のコードへ
依存しない」は**SDKの本番コード（go.mod依存・import）**の話であり、
テスト・CIが検証目的で本体のソースを利用することは妨げない（ユーザー確認
済み）。実際、`go/execsandbox/`パッケージ自体のgo.modには本体への依存は
一切追加していない。

- **本体はGitHub Releasesではなくソースからビルドする。** `~/reference/
  execsandbox`への手元clone、CIでは`actions/checkout`の`repository:`で別
  リポジトリとしてcheckoutし、`make cross-base && go build ./cmd/
  execsandbox-build`でビルダーを都度用意する。
- **本体のバージョンはタグで固定する**（`.github/workflows/ci.yml`の
  `EXECSANDBOX_HOST_REF`、現在`v0.1.0`）。本体のmain更新でSDK側CIが
  無関係な理由で壊れることを避けるため。本体の新しいタグを追随したく
  なったら、ここを書き換える。
- E2Eスクリプト（`go/tests/*.sh`）は本体の`tests/e2e_basic.sh`・
  `tests/e2e_conn.sh`と同じ構成・観測手段（プロセス終了・標準出力・
  `tests/connclient`）を踏襲する。
- **Unixソケットパス長の落とし穴**：`XDG_RUNTIME_DIR`を`mktemp -d`の既定
  パス（scratchpad配下等、長くなりがち）に置くと`bind: invalid argument`
  で失敗することがある（108バイト前後の制限）。`go/tests/e2e_send_recv.sh`は
  `/tmp`直下に短い一時ディレクトリを別途作ってこれに充てている。
- **`actions/setup-go`のキャッシュは無効化する**（`cache: false`）。
  `go/execsandbox/`・`go/examples/*`はいずれも外部依存を持たず`go.sum`が
  存在しないため、既定の依存キャッシュは`go.sum`を探せず警告を出す
  （2026-09-08、CI初回実行のwarningで発覚）。`actions/checkout`・
  `actions/setup-go`のメジャーバージョンは、Node.jsランタイムが新しい
  ものに追随する（現在`checkout@v7`・`setup-go@v7`、両方`node24`）。

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
   （`go/execsandbox/`に`Send`/`Recv`/`ConnWrite`/`MaxFrame`を実装。
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
