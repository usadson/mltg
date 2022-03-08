# Hello, world!

mltgの`examples`にある`hello.rs`を例にmltgの使い方を見ていきましょう。
`hello.rs`は単純に青い背景にHello, world!と表示するだけのプログラムとなります。

[ソースコード](https://github.com/LNSEAB/mltg/blob/master/mltg/examples/hello.rs)

`hello.rs`では以下のクレートを使っています。

* [anyhow](https://crates.io/crates/anyhow)
* [wita](https://crates.io/crates/wita)

## Applicationの定義

描画に必要な変数を`Application`のフィールドに定義しています。

```rust,ignore
struct Application {
    context: mltg::Context<mltg::Direct2D>,
    back_buffer: Vec<mltg::d2d::RenderTarget>,
    white: mltg::Brush,
    text_layout: mltg::TextLayout,
}
```

`mltg::Context`の型引数に`mltg::Direct2D`、`mltg::Direct3D11`、`mltg::Direct3D12`のいずれかを
指定する必要があります。ここではDirect2Dだけを使うので`mltg::Direct2D`にしています。
また、`back_buffer`は描画先となるので持っておく必要があり、
`white`が色を表す`mltg::Brush`で`text_layout`が字を描画するための`mltg::TextLayout`となっています。


## Apllication::new

`Application::new`の中でウィンドウやコンテキストを作っていきます。

### ウィンドウを作る

mltgにはウィンドウを生成する機能はないので他のクレートを利用してください。
ここでは`wita`を使います (自分で作ったクレートの宣伝)。

```rust,ignore
let window = wita::WindowBuilder::new().title("mltg hello").build()?;
let window_size = window.inner_size();
```

ウィンドウの生成とウィンドウのクライアント領域のサイズを取得しています。

### バックエンドを作る

`mltg::Direct2D`を作ります。

```rust,ignore
let backend = mltg::Direct2D::new(
    window.raw_handle(),
    (window_size.width, window_size.height)
)?;
```

ウィンドウのハンドルと`mltg::Direct2D`の中に作られるバックバッファのサイズを引数として渡すと
`mltg::Direct2D`を作ることができます。

### コンテキストとバックバッファを作る

`mltg::Context`と`Vec<mltg::d3d::RenderTarget>`を作ります。

```rust,ignore
let context = mltg::Context::new(backend)?;
let back_buffer = context.create_back_buffers()?;
```

`mltg::Direct2D`である`backend`を渡すだけで`mltg::Context`を作れます。
そして今回はバックバッファに直接描画するために`context.create_back_buffers`を使って
描画先となるオブジェクトを作ります。`mltg::Direct2D`の`swap_chain`を渡す必要があるのですが、
`mltg::Context`の中に取り込まれるので`mltg::Context`の`backend`で`mltg::Direct2D`を借用して
`swap_chain`を呼び出しています。

> `mltg::Direct3D11`や`mltg::Direct3D12`をバックエンドにした場合は
> バックエンドの中でスワップチェーンを作らないので、ユーザ側でスワップチェーンを作って`create_back_buffers`に渡してもらうことになります。

### ブラシと文字フォーマットを作る

`mltg::Brush`と`mltg::TextFormat`を作ります。

```rust,ignore
let white = context.create_solid_color_brush([1.0, 1.0, 1.0, 1.0])?;
let text_format = context.create_text_format(
    &mltg::Font::system("Yu Gothic UI"),
    mltg::font_point(28.0),
    None,
)?;
```

Direct2DとDirectWriteの関数とほぼ一緒です。
`context.create_text_format`の`mltg::font_point`はフォントのサイズをポイントで指定するための構造体です。
`mltg::font_point`を使わず直接`f32`の値を指定すると論理ピクセル(DIP)でサイズをすることになります。

## イベント

`Application::new`の処理が終わるとウィンドウに描画します。
そしてウィンドウのサイズが変更された場合は対応することとします。

### 描画

`Application`が実装する`wita::EventHandler`の`draw`の中で描画します。

```rust,ignore
fn draw(&mut self, _window: &wita::Window) {
    self.context.draw(&self.back_buffer[0], |cmd| {
        cmd.clear([0.0, 0.0, 0.3, 0.0]);
        cmd.draw_text("Hello, world!", &self.text_format, &self.white, (0.0, 0.0));
    });
}
```

`self.context.draw`で描画します。ここで描画先として`self.back_buffer`を渡します。

mltgでは`BeginDraw`と`EndDraw`で囲む代わりにクロージャの中に処理を書くようにしています。
クロージャの第1引数は`mltg::DrawCommand`で描画するためのコマンドをまとめたオブジェクトです。

ここでは`cmd.clear`で青い背景になるように塗りつぶして`cmd.draw_text`で文字列を描画しています。

### ウィンドウのサイズが変更されている場合

`Application`が実装する`wita::EventHandler`の`resizing`の中で描画します。

ちなみに`resizing`はウィンドウのサイズ変更中に呼ばれます。

```rust,ignore
fn resizing(&mut self, window: &wita::Window, size: wita::PhysicalSize<u32>) {
    self.back_buffer.clear();
    self.context.resize((size.width, size.height));
    self.back_buffer = self
        .context
        .create_back_buffers()
        .unwrap();
    window.redraw();
}
```

一旦バックバッファに関連付けられた`mltg::d2d::RenderTarget`を全て破棄して
バックバッファのサイズを変更して作り直して再描画しています。

> `mltg::Direct3D11`と`mltg::Direct3D12`ではユーザ側でスワップチェーンを作ってもらうことになるため、
> スワップチェーンの`ResizeBuffers`を呼び出す前に`mltg::d3d11::RenderTarget`や`mltg::d3d12::RenderTarget`を
> 破棄する必要があります。