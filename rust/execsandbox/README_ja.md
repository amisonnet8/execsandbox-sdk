# execsandbox

[ExecSandbox](https://github.com/amisonnet8/execsandbox)のWASMゲストABI
（`send`/`recv`/`conn_write`/`max_frame`）向けのRustバインディング。
`wasm32-wasip1`ターゲット向けにビルドするゲストモジュール向け。

ポインタ・長さの受け渡しとバッファサイズ計算を、小さなRustネイティブな
APIの裏に隠す：`send`/`recv`/`conn_write`は`&[u8]`/`Vec<u8>`を扱い、
`Kind`列挙型が`recv`の返り値の種別を示し、タイムアウトは
`Option<Duration>`で受け取る。

## インストール

このクレートはまだcrates.ioに公開していない。公開までは、このリポジトリを
直接gitで参照すること：

```toml
[dependencies]
execsandbox = { git = "https://github.com/amisonnet8/execsandbox-sdk" }
```

## 使い方

このクレートはwasmターゲット向けにビルドされたゲストモジュール内でしか
意味を持たない——ホストアーキテクチャではなく`wasm32-wasip1`向けに
ビルドすること：

```
cargo build --target wasm32-wasip1 --release
```

```rust
fn main() {
    execsandbox::send(1, b"hello");

    if let Some(msg) = execsandbox::recv(None) { // メッセージが届くまでブロック
        if msg.kind == execsandbox::Kind::ConnData {
            let _ = execsandbox::conn_write(msg.conn_id, &msg.data);
        }
    }
}
```

完全に動くゲストモジュールの例は[`examples/`](examples)を、API全体は
[クレートドキュメント](https://docs.rs/execsandbox)を参照。

## ABIの互換性

このクレートはExecSandboxのWASM ABIにバインドしており、本体プロジェクトは
後方互換性のみを保証している：このクレートのABI関数が導入された時点の
バージョン以上のExecSandboxホストであれば、このクレートでビルドした
ゲストモジュールを実行できる。ABI自体はこのクレートではなくホスト
プロジェクトが定義している——
[`docs/spec/execsandbox_spec_ja.md`](https://github.com/amisonnet8/execsandbox/blob/main/docs/spec/execsandbox_spec_ja.md)
の§5を参照。

---

For the English version, see [README.md](README.md).
