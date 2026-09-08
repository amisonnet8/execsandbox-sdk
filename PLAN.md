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

**Rust版SDKも実装・単体テスト・実機疎通・CI・ドキュメントまで一区切り
ついた（2026-09-08）。** TinyGo版と対になる構成・検証を行った。

- **環境構築**：devcontainerに公式feature`ghcr.io/devcontainers/
  features/rust:1`を追加。wasm32-wasip1ターゲットはfeatureの範囲外の
  ため`.devcontainer/install-rust-wasm-target.sh`
  （`rustup target add wasm32-wasip1`）をpostCreateCommandに追加した
  （**devcontainerのリビルドが必要**）。このセッションでは
  リビルド前にrustup公式インストーラ（sh.rustup.rs）で直接導入して
  作業した。
- **実装**：`rust/execsandbox/src/lib.rs`に`send`/`recv`/`conn_write`/
  `max_frame`と`Kind`/`Message`型。TinyGoと違い、Rustの`extern "C"`関数は
  値として扱える制約がないため、Go版の「パッケージ変数＋`init()`束縛」の
  代わりに`Host`トレイト＋`RealHost`（`#[cfg(target_arch = "wasm32")]`で
  実ABIへ、それ以外は`unreachable!()`）で実装した。ABI宣言自体
  （`#[link(wasm_import_module = "execsandbox")]`）は`src/abi.rs`に分離し
  `#[cfg(target_arch = "wasm32")]`で隔離（TinyGoの`abi.go`と同じ理由）。
  `#[link(wasm_import_module = ...)]`はstable Rust（1.98.1で確認）で
  問題なく使え、nightly機能は不要だった。
  - **`Duration`は負にできない点がGoとの設計差**：Goは`time.Duration`が
    符号付きで「負値＝無期限待ち」と直接表現できたが、Rustの
    `std::time::Duration`は非負のみ。そのため`recv`のtimeoutは
    `Option<Duration>`（`None`＝無期限、`Some(Duration::ZERO)`＝即時、
    `Some(d)`＝`d`まで待つ）とした。ABIの`timeout_ms`の意味（負値＝無期限、
    `0`＝即時、正値＝その時間まで）自体はGo版と変えていない。
  - **エラーの表現方法もGoとの意図的な設計差**：`conn_write`の失敗は
    Goが標準の`error`インターフェース、Rustは専用の`UnknownConnection`型を
    返す`Result<(), UnknownConnection>`とした。ABI上の失敗パターンは1種類
    （未知の`conn_id`）のみで両言語とも変わらないが、表現は各言語の慣習に
    合わせた（Rustは失敗が１パターンしかない場合、専用エラー型＋`Result`が
    慣用的）。`send`が失敗を一切報告しない（ABI仕様上の設計、spec 3.4）点も
    両言語で共通のドキュメント方針（doc comment上部に明記）を取っている。
- **単体テスト**：`Host`トレイトをテスト用の`MockHost`（クロージャを
  `RefCell<Option<Box<dyn FnMut...>>>`で保持）に差し替え、
  `recv_with`/`send_with`/`conn_write_with`等の非公開ヘルパーを直接叩いて
  ホストアーキテクチャ上で検証（`cargo test`、9件）。グローバル変数越しの
  DIではなく関数引数での注入にしたのは、`cargo test`が既定でテストを
  並列実行するため（Goの`go test`は既定で同一パッケージ内は逐次実行）。
  rustdoc doctest（`no_run`、Go版の`Example*`相当、pkg.go.dev同様
  docs.rsのAPIドキュメントに表示される）も`send`/`recv`/`conn_write`/
  `max_frame`に追加（4件、コンパイルのみ確認され実行はされない）。
  `cargo clippy --all-targets -- -D warnings`・`cargo fmt --check`も
  クリーン。
- **実機疎通・E2E**：`rust/execsandbox/examples/{sender,receiver,echo}.rs`
  （Cargoの組み込みexamples機能を使うため、Go版と違いクレート内に置く。
  「ディレクトリ構成の決定事項」参照）と
  `rust/tests/{lib.sh,e2e_send_recv.sh,e2e_conn.sh}`。構成はGo版の
  `go/tests/*`と同一（`build_host_tools`はあえて重複させている。理由は
  「ディレクトリ構成の決定事項」）。手元で`~/reference/execsandbox`に対し
  Send/Recv/ConnWrite/MaxFrameいずれも実機疎通を確認済み。
- **CI**：`.github/workflows/ci.yml`に`unit-test-rust`（fmt/clippy/test）・
  `wasm-build-rust`（wasm32-wasip1でexamplesビルド）・`integration-rust`
  （本体をタグ固定でcheckoutしe2eスクリプトを実行）を追加。既存のGo向け
  3ジョブも`unit-test-go`/`wasm-build-go`/`integration-go`に改名した
  （2言語になったための明確化）。pushしてGitHub Actionsで6ジョブとも成功
  （green）を確認済み。
- **ドキュメント**：`rust/execsandbox/README.md`（crates.io/docs.rs向け、
  英語。rustdoc言語決定は「対応言語（決定事項）」参照）。

**devcontainerにVS Code拡張`github.vscode-github-actions`も追加した**
（`.github/workflows/ci.yml`をエディタ上で見やすくするため）。

**使い勝手・ドキュメントの見直しも一巡した（2026-09-08）。** 「次にやること」
5番の一環として、両言語のREADME・パッケージメタデータ・サンプルを点検し、
以下を対応した。

- `rust/execsandbox/README.md`の`cargo add`/`docs.rs`記述を、crates.io未公開の
  現状に合わせて修正（`git`依存での参照方法を明記。実際に`cargo build`で
  解決・ビルドできることを確認済み——サブディレクトリ配置・非ワークスペース
  構成でもCargoは単一のcrateを自動検出する）。
- `go/execsandbox/`・`rust/execsandbox/`それぞれにLICENSEファイルを追加
  （ルートの`LICENSE`と同内容。pkg.go.dev/crates.ioはパッケージ/クレート
  直下のLICENSE検出を期待するため）。
- Go版のpackage doc commentは`doc.go`に既に存在していることを確認
  （当初「無い」と誤認していた。`execsandbox.go`への重複追加は取り消し済み）。
- `conn_write`のエラー表現がGo（`error`）とRust（`UnknownConnection`＋
  `Result`）で異なる点は意図的な設計差であることをPLAN.mdへ明記
  （Rust実装の「経緯」節）。
- 新example `worker`をGo（`go/examples/worker/`）・Rust
  （`rust/execsandbox/examples/worker.rs`）双方に追加。既存のsender/
  receiver/echoが単機能のデモなのに対し、`worker`は正のtimeoutで
  待ち時間中に周期処理（ハートビート）を行い、全`Kind`を明示的に
  分岐するという「よくある使い方パターン」を示す。両言語ともCIの
  wasm-buildジョブはディレクトリを走査する構成のため、追加のCI変更は
  不要（`go/examples/*/`のループ、`cargo build --examples`）。

次は「次にやること」の5番のうち、パッケージマネージャへの公開方法の検討が
残っている。
（2026-09-08、execsandbox本体のフェーズ①〜④完了後に着手）

### ディレクトリ構成の決定事項（2026-09-08）

**言語ごとにリポジトリ直下のディレクトリを分ける。** Go/TinyGo版は
`go/`、Rust版は`rust/`。両言語ともパッケージ本体・サンプル・E2Eテストを
その配下にまとめる点は共通だが、**examplesの置き場所だけ言語ごとに異なる**
（下記）。この非対称は意図的：トップレベル（`go/` vs `rust/`が並ぶ）での
対称性は保ちつつ、各言語の内部構造はその言語のツールの慣習
（GoのモジュールシステムvsCargoの組み込みexamples機能）に従わせている。

| | Go/TinyGo版 | Rust版 |
| :--- | :--- | :--- |
| パッケージ/クレート本体 | `go/execsandbox/` | `rust/execsandbox/` |
| サンプル | `go/examples/`（**独立ディレクトリ**。各サンプルが個別の`go.mod`を持ち、`replace`でローカルの`execsandbox/`を参照） | `rust/execsandbox/examples/`（**クレート内**。Cargoの組み込みexamples機能で`cargo build --example <name>`が使える） |
| E2Eテスト | `go/tests/` | `rust/tests/` |

- **経緯**：当初はGo/TinyGo版だけが対象という理由で`execsandbox/`・
  `examples/`・`tests/`をリポジトリ直下に直接置いていたが、Rust追加時に
  `execsandbox/`（Go、直下）と`rust/`（Rust、直下にネスト）が非対称になる
  点をユーザーに指摘され、`go/`配下へ移動した（2026-09-08）。Rust着手時に
  実際に`rust/`を追加する段になり、examplesの置き場所は各言語のツールの
  慣習に従うのが自然と判断した（Cargoは`<crate>/examples/*.rs`を標準の
  サンプル置き場として認識するため、そこから外すほうがかえって不自然）。
- Goモジュール名は`github.com/amisonnet8/execsandbox-sdk/go/execsandbox`。
  パッケージ名`execsandbox`とディレクトリ名を一致させ、importの別名指定が
  不要になるようにしている（ディレクトリ名を`go`一段のみにしないのは、
  Goの予約語`go`はパッケージ名にできないため）。Rustのクレート名は
  `execsandbox`（Cargoはディレクトリ名とcrate名の一致を要求しないため、
  この制約自体がないが、揃えてある）。
- `go/tests/lib.sh`と`rust/tests/lib.sh`は同じ`build_host_tools`関数を
  あえて重複させている。`go/`・`rust/`をそれぞれ自己完結させ、一方が
  他方の内部ファイルに依存しない構成を優先した（DRYよりディレクトリ間の
  独立性を優先する判断）。

`go/execsandbox/`はwasmアーキテクチャ向けにしかビルドできない
（`//go:wasmimport`が非wasmターゲットでは受理されないため）。ビルドタグは
付けていない。TinyGoの`-target=wasip1`、または標準Goでも
`GOOS=wasip1 GOARCH=wasm go build`でビルド・型チェックできる。

`rust/execsandbox/`は全ターゲットでビルド・テストできる（`extern "C"`の
ABI宣言は`src/abi.rs`を`#[cfg(target_arch = "wasm32")]`で隔離しており、
それ以外のターゲットでは`Host`トレイトの別実装——呼ばれたら
`unreachable!()`——にフォールバックするため）。単体テストは
`cargo test`（ホストアーキテクチャ）、実際のゲストモジュールとしての
ビルドは`cargo build --target wasm32-wasip1 --examples`。

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
  Rustも公式feature（`ghcr.io/devcontainers/features/rust`）が存在するため
  それを使う。wasm32-wasip1ターゲットはfeatureの範囲外
  （rustupのtarget追加が必要）なため、TinyGoと同様
  `.devcontainer/install-rust-wasm-target.sh`
  （`rustup target add wasm32-wasip1`）をpostCreateCommandで追加実行する
  （2026-09-08）。

## このプロジェクトについて

execsandbox本体が提供するWASM ABIの上に、各言語ネイティブなSDKライブラリを
提供する。詳細は`README.md`参照。ABIの定義自体は本体リポジトリ
（execsandbox）の仕様書§5にあり、このリポジトリでは持たない
（`.claude/rules/abi-boundary.md`）。

## 対応言語（決定事項）

**TinyGo（Go）を最初に実装し、その後Rustに着手する。**
（2026-09-08決定、両方とも着手済み）

- godocコメント言語: **英語のみ**（2026-09-08決定）。
- rustdocコメント言語: **英語のみ**（2026-09-08決定。godocと同じ理由——
  crates.io/docs.rsの読者も本体より広く英語話者を含む——で一貫させた）。

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
4. ~~TinyGo版が一段落したらRustに着手する（rustdocコメント言語をその時点で
   改めて判断）。~~ 完了（`rust/execsandbox/`に実装・単体テスト・
   examples・E2E・CI・ドキュメント。rustdocは英語に決定。CIもpush済みで
   6ジョブとも成功（green）確認済み。詳細は「現在地」参照）。
5. 両言語とも実装・CI確認まで一区切りついたので、次に何をするかユーザーと
   相談する（候補: パッケージマネージャへの公開方法の検討（保留事項）、
   他に追加すべき使い勝手やドキュメント等）。

## 参考: execsandbox本体との役割分担

| リポジトリ | 役割 |
| :--- | :--- |
| execsandbox | ホスト本体・ビルダー・ABI仕様の正 |
| execsandbox-sdk（本リポジトリ） | ゲスト側SDKライブラリ（全言語） |
