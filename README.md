# mltg

[![mltg at crates.io](https://img.shields.io/crates/v/mltg.svg)](https://crates.io/crates/mltg)
[![mltg at docs.rs](https://docs.rs/mltg/badge.svg)](https://docs.rs/mltg)

Direct2D wrapper library

* [examples](https://docs.rs/crate/mltg/latest/source/examples/)

## Usage overview

### 1. Create a `mltg::Context` and a `mltg::Factory`
```rust
let ctx = mltg::Context(mltg::Direct2D()?)?;
let factory = ctx.create_factory();
```

### 2. Create a render target
In the case of using winit.
```rust
let window_size = window.inner_size().to_logical::<f32>(window.scale_factor());
let mut render_target = ctx.create_render_target(
    window.raw_window_handle(), // winit window
    (window_size.width, window_size.height),
)?;
```

### 3. Create resources

For example, create the solid color brush.
```rust
let brush = factory.create_solid_color_brush((1.0, 1.0, 1.0, 1.0))?;
```

### 4. Draw

For example, draw the rectangle of filled color.
```rust
ctx.set_scale_factor(window.scale_factor() as _);
ctx.draw(&render_target, |cmd| {
    cmd.clear((0.0, 0.0, 0.3, 0.0));
    cmd.fill(
        &mltg::Rect::from_points((10.0, 10.0), (100.0, 100.0)),
        &brush,
    );
});
```

--------------------------------------------------------------------
Copyright (c) 2021 LNSEAB
