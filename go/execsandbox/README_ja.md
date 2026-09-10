# execsandbox

[![Go Reference](https://pkg.go.dev/badge/github.com/amisonnet8/execsandbox-sdk/go/execsandbox.svg)](https://pkg.go.dev/github.com/amisonnet8/execsandbox-sdk/go/execsandbox)

[ExecSandbox](https://github.com/amisonnet8/execsandbox)のWASMゲストABI
（`send`/`recv`/`conn_write`/`max_frame`）向けのGoバインディング。
[TinyGo](https://tinygo.org/)でコンパイルするゲストモジュール向け。

ポインタ・長さの受け渡しとバッファサイズ計算を、小さなGoネイティブな
APIの裏に隠す：`Send`/`Recv`/`ConnWrite`は`[]byte`を扱い、`Kind`列挙型が
`Recv`の返り値の種別を示し、タイムアウトは`time.Duration`で受け取る。

## インストール

```
go get github.com/amisonnet8/execsandbox-sdk/go/execsandbox
```

## 使い方

このパッケージはwasmターゲット向けにビルドされたゲストモジュール内でしか
意味を持たない——`go build`ではなくTinyGoでビルドすること：

```
tinygo build -target=wasip1 -o guest.wasm .
```

```go
package main

import "github.com/amisonnet8/execsandbox-sdk/go/execsandbox"

func main() {
	execsandbox.Send(1, []byte("hello"))

	msg, ok := execsandbox.Recv(-1) // メッセージが届くまでブロック
	if ok && msg.Kind == execsandbox.KindConnData {
		execsandbox.ConnWrite(msg.ConnID, msg.Data)
	}
}
```

完全に動くゲストモジュールの例は[`../examples`](../examples)を、API全体は
[パッケージドキュメント](https://pkg.go.dev/github.com/amisonnet8/execsandbox-sdk/go/execsandbox)
を参照。

## ABIの互換性

このパッケージはExecSandboxのWASM ABIにバインドしており、本体プロジェクトは
後方互換性のみを保証している：このパッケージのABI関数が導入された時点の
バージョン以上のExecSandboxホストであれば、このパッケージでビルドした
ゲストモジュールを実行できる。ABI自体はこのパッケージではなくホスト
プロジェクトが定義している——
[`docs/spec/execsandbox_spec_ja.md`](https://github.com/amisonnet8/execsandbox/blob/main/docs/spec/execsandbox_spec_ja.md)
の§5を参照。

---

For the English version, see [README.md](README.md).
