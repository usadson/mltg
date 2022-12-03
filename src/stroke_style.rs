use crate::*;
use windows::Win32::Graphics::Direct2D::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum CapStyle {
    Flat = D2D1_CAP_STYLE_FLAT.0,
    Square = D2D1_CAP_STYLE_SQUARE.0,
    Round = D2D1_CAP_STYLE_ROUND.0,
    Triangle = D2D1_CAP_STYLE_TRIANGLE.0,
}

#[derive(Clone, Copy, Debug)]
pub enum LineJoin {
    Miter,
    Bevel,
    Round,
    MiterOrBevel(f32),
}

#[derive(Clone, Copy, Debug)]
pub enum DashStyle<'a> {
    Solid,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
    Custom(&'a [f32]),
}

#[derive(Clone, Debug)]
pub struct Dash<'a> {
    pub cap: CapStyle,
    pub style: DashStyle<'a>,
    pub offset: f32,
}

impl<'a> Default for Dash<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            cap: CapStyle::Flat,
            style: DashStyle::Solid,
            offset: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrokeStyleProperties<'a> {
    pub start_cap: CapStyle,
    pub end_cap: CapStyle,
    pub line_join: LineJoin,
    pub dash: Option<Dash<'a>>,
}

impl<'a> Default for StrokeStyleProperties<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            start_cap: CapStyle::Flat,
            end_cap: CapStyle::Flat,
            line_join: LineJoin::Miter,
            dash: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StrokeStyle(ID2D1StrokeStyle);

impl StrokeStyle {
    pub(crate) fn new(factory: &ID2D1Factory6, props: &StrokeStyleProperties) -> Result<Self> {
        let (line_join, miter_limit) = match props.line_join {
            LineJoin::Miter => (D2D1_LINE_JOIN_MITER, 1.0),
            LineJoin::Bevel => (D2D1_LINE_JOIN_BEVEL, 1.0),
            LineJoin::Round => (D2D1_LINE_JOIN_ROUND, 1.0),
            LineJoin::MiterOrBevel(miter_limit) => (D2D1_LINE_JOIN_MITER_OR_BEVEL, miter_limit),
        };
        let (dash_cap, dash_style, dash_offset, dashes) = match props.dash.as_ref() {
            Some(dash) => {
                let cap = D2D1_CAP_STYLE(dash.cap as u32);
                let (style, dashes): (_, Option<&[f32]>) = match dash.style {
                    DashStyle::Solid => (D2D1_DASH_STYLE_SOLID, None),
                    DashStyle::Dash => (D2D1_DASH_STYLE_DASH, None),
                    DashStyle::Dot => (D2D1_DASH_STYLE_DOT, None),
                    DashStyle::DashDot => (D2D1_DASH_STYLE_DASH_DOT, None),
                    DashStyle::DashDotDot => (D2D1_DASH_STYLE_DASH_DOT_DOT, None),
                    DashStyle::Custom(dashes) => (D2D1_DASH_STYLE_CUSTOM, Some(dashes)),
                };
                (cap, style, dash.offset, dashes)
            }
            None => (D2D1_CAP_STYLE_FLAT, D2D1_DASH_STYLE_SOLID, 0.0, None),
        };
        let props = D2D1_STROKE_STYLE_PROPERTIES {
            startCap: D2D1_CAP_STYLE(props.start_cap as _),
            endCap: D2D1_CAP_STYLE(props.end_cap as _),
            dashCap: dash_cap,
            lineJoin: line_join,
            miterLimit: miter_limit,
            dashStyle: dash_style,
            dashOffset: dash_offset,
        };
        let stroke_style = unsafe { factory.CreateStrokeStyle(&props, dashes)? };
        Ok(StrokeStyle(stroke_style))
    }

    pub(crate) fn handle(&self) -> &ID2D1StrokeStyle {
        &self.0
    }
}
