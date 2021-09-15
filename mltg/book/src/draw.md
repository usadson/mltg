# 描画

`mltg::DrawCommand`に描画するための関数があるので紹介していきます。

## stroke -- 輪郭を描く

`mltg::DrawCommand::stroke`は図形の輪郭を描くための関数です。

```rust,ignore
pub fn stroke(
    &self,
    object: &impl Stroke,
    brush: &Brush,
    width: f32,
    style: Option<&StrokeStyle>
)
```

`mltg::Stroke`を実装する型の値を`object`に渡すと渡した図形の輪郭を描くことができます。

以下の型が`mltg::Stroke`を実装しています。
* `mltg::Ellipse`
* `mltg::Line`
* `mltg::Path`
* `mltg::RoundedRect`
* `mltg::Circle`
* `mltg::Rect`

## fill -- 塗りつぶし

`mltg::DrawCommand::fill`は塗りつぶした図形を描くための関数です。

```rust,ignore
pub fn fill(
    &self,
    object: &impl Fill,
    brush: &Brush
)
```

`mltg::Fill`を実装する型の値を`object`に渡すと塗りつぶした図形を描くことができます。

以下の型が`mltg::Fill`を実装しています。
* `mltg::Ellipse`
* `mltg::Path`
* `mltg::RoundedRect`
* `mltg::Circle`
* `mltg::Rect`

## draw_image -- 画像

`mltg::DrawCommand::draw_image`は`mltg::Context::create_image`で作った画像を描画するための関数です。

```rust,ignore
pub fn draw_image(
    &self,
    image: &Image,
    dest_rect: impl Into<Rect>,
    src_rect: Option<Rect>,
    interpolation: Interpolation
)
```

## draw_text -- 文字列の描画

`mltg::DrawCommand::draw_text`は文字列を描画するための関数です。

```rust,ignore
pub fn draw_text(
    &self,
    text: &str,
    format: &TextFormat,
    brush: &Brush,
    origin: impl Into<Point>
)
```

`mltg::DrawCommand::draw_text`は呼び出すたびに内部で`mltg::TextLayout`を作って描画します。

## draw_text_layout -- レイアウト文字列の描画

`mltg::DrawCommand::draw_text_layout`は`mltg::Context::create_text_layout`で作った文字列を描画するための関数です。

```rust,ignore
pub fn draw_text_layout(
    &self,
    layout: &TextLayout,
    brush: &Brush,
    origin: impl Into<Point>
)
```

## clear -- ターゲットのクリア

```rust,ignore
pub fn clear(
    &self,
    color: impl Into<Rgba>
)
```

`mltg::DrawCommand::clear`は指定された色でターゲット全体を塗りつぶします。

Direct3D11や12においてバックバッファに直接描画する場合は`mltg::DrawCommand::clear`を呼び出すよりも
Direct3D側でバックバッファのクリアを行うことをおすすめします。
