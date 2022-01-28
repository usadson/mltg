use crate::*;
use std::convert::TryInto;
use windows::core::Interface;
use windows::Win32::{Foundation::*, Graphics::DirectWrite::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum FontWeight {
    Thin = DWRITE_FONT_WEIGHT_THIN,
    UltraLight = DWRITE_FONT_WEIGHT_ULTRA_LIGHT,
    Light = DWRITE_FONT_WEIGHT_LIGHT,
    SemiLight = DWRITE_FONT_WEIGHT_SEMI_LIGHT,
    Regular = DWRITE_FONT_WEIGHT_REGULAR,
    Medium = DWRITE_FONT_WEIGHT_MEDIUM,
    SemiBold = DWRITE_FONT_WEIGHT_SEMI_BOLD,
    Bold = DWRITE_FONT_WEIGHT_BOLD,
    UltraBold = DWRITE_FONT_WEIGHT_ULTRA_BOLD,
    Heavy = DWRITE_FONT_WEIGHT_HEAVY,
    UltraBlack = DWRITE_FONT_WEIGHT_ULTRA_BLACK,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum FontStyle {
    Normal = DWRITE_FONT_STYLE_NORMAL,
    Oblique = DWRITE_FONT_STYLE_OBLIQUE,
    Italic = DWRITE_FONT_STYLE_ITALIC,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum FontStretch {
    Undefined = DWRITE_FONT_STRETCH_UNDEFINED,
    UltraCondensed = DWRITE_FONT_STRETCH_ULTRA_CONDENSED,
    ExtraCondensed = DWRITE_FONT_STRETCH_EXTRA_CONDENSED,
    Condensed = DWRITE_FONT_STRETCH_CONDENSED,
    SemiCondensed = DWRITE_FONT_STRETCH_SEMI_CONDENSED,
    Medium = DWRITE_FONT_STRETCH_MEDIUM,
    SemiExpanded = DWRITE_FONT_STRETCH_SEMI_EXPANDED,
    Expanded = DWRITE_FONT_STRETCH_EXPANDED,
    ExtraExpanded = DWRITE_FONT_STRETCH_EXTRA_EXPANDED,
    UltraExpanded = DWRITE_FONT_STRETCH_ULTRA_EXPANDED,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct FontPoint(pub f32);

impl From<FontPoint> for f32 {
    #[inline]
    fn from(src: FontPoint) -> f32 {
        src.0 * 96.0 / 72.0
    }
}

#[inline]
pub fn font_point(value: f32) -> FontPoint {
    FontPoint(value)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextStyle {
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,
}

impl Default for TextStyle {
    #[inline]
    fn default() -> Self {
        Self {
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Medium,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum TextAlignment {
    Leading = DWRITE_TEXT_ALIGNMENT_LEADING,
    Center = DWRITE_TEXT_ALIGNMENT_CENTER,
    Trailing = DWRITE_TEXT_ALIGNMENT_TRAILING,
    Justified = DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
}

impl std::convert::TryFrom<DWRITE_TEXT_ALIGNMENT> for TextAlignment {
    type Error = ();

    fn try_from(src: DWRITE_TEXT_ALIGNMENT) -> Result<Self, ()> {
        let dest = match src {
            DWRITE_TEXT_ALIGNMENT_LEADING => TextAlignment::Leading,
            DWRITE_TEXT_ALIGNMENT_CENTER => TextAlignment::Center,
            DWRITE_TEXT_ALIGNMENT_TRAILING => TextAlignment::Trailing,
            DWRITE_TEXT_ALIGNMENT_JUSTIFIED => TextAlignment::Justified,
            _ => return Err(()),
        };
        Ok(dest)
    }
}

#[derive(Clone, Debug)]
pub enum Font {
    System(String),
    File(std::path::PathBuf, String),
}

impl Font {
    #[inline]
    pub fn system(name: impl AsRef<str>) -> Self {
        Self::System(name.as_ref().to_string())
    }

    #[inline]
    pub fn file(path: impl AsRef<std::path::Path>, name: impl AsRef<str>) -> Self {
        Self::File(path.as_ref().to_path_buf(), name.as_ref().to_string())
    }

    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Self::System(name) => name.as_str(),
            Self::File(_, name) => name.as_str(),
        }
    }
}

#[derive(Clone)]
pub struct TextFormat {
    format: IDWriteTextFormat,
    font: Font,
    size: f32,
    style: TextStyle,
}

impl TextFormat {
    #[inline]
    pub(crate) fn new(
        factory: &IDWriteFactory5,
        font: &Font,
        size: f32,
        style: Option<&TextStyle>,
    ) -> windows::core::Result<Self> {
        let style = style.cloned().unwrap_or_default();
        let (font_name, font_collection): (_, Option<IDWriteFontCollection>) = match font {
            Font::System(font_name) => (font_name, None),
            Font::File(path, font_name) => unsafe {
                let set_builder: IDWriteFontSetBuilder1 =
                    { factory.CreateFontSetBuilder()?.cast()? };
                let font_file = {
                    factory.CreateFontFileReference(
                        path.as_path().to_string_lossy().as_ref(),
                        std::ptr::null(),
                    )?
                };
                set_builder.AddFontFile(&font_file)?;
                let font_set = { set_builder.CreateFontSet()? };
                let font_collection = { factory.CreateFontCollectionFromFontSet(&font_set)? };
                (font_name, Some(font_collection.into()))
            },
        };
        let format = unsafe {
            factory.CreateTextFormat(
                font_name.as_str(),
                font_collection,
                style.weight as _,
                style.style as _,
                style.stretch as _,
                size,
                "",
            )?
        };
        Ok(Self {
            format,
            font: font.clone(),
            size,
            style,
        })
    }

    #[inline]
    pub fn font(&self) -> &Font {
        &self.font
    }

    #[inline]
    pub fn font_size(&self) -> f32 {
        self.size
    }

    #[inline]
    pub fn style(&self) -> &TextStyle {
        &self.style
    }
}

impl PartialEq for TextFormat {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
    }
}

impl Eq for TextFormat {}

unsafe impl Send for TextFormat {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HitTestResult {
    pub text_position: usize,
    pub inside: bool,
    pub trailing_hit: bool,
}

#[derive(Clone)]
pub struct TextLayout {
    layout: IDWriteTextLayout,
    format: TextFormat,
    text: String,
    size: Size,
}

impl TextLayout {
    #[inline]
    pub(crate) fn new(
        factory: &IDWriteFactory5,
        text: &str,
        format: &TextFormat,
        alignment: TextAlignment,
        size: Option<Size>,
    ) -> windows::core::Result<Self> {
        let (layout, max_size) = unsafe {
            let text = text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            let layout = factory.CreateTextLayout(
                PWSTR(text.as_ptr() as _),
                text.len() as _,
                &format.format,
                std::f32::MAX,
                std::f32::MAX,
            )?;
            let size = size.unwrap_or_else(|| {
                let metrics = layout.GetMetrics().unwrap();
                (metrics.width, metrics.height).into()
            });
            layout.SetMaxWidth(size.width).unwrap();
            layout.SetMaxHeight(size.height).unwrap();
            layout.SetTextAlignment(alignment as _).unwrap();
            layout
                .SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)
                .unwrap();
            (layout, size)
        };
        Ok(Self {
            layout,
            format: format.clone(),
            text: text.into(),
            size: max_size,
        })
    }

    #[inline]
    pub(crate) fn draw(&self, dc: &ID2D1DeviceContext, brush: &Brush, origin: Point) {
        unsafe {
            let origin: D2D_POINT_2F = Inner(origin).into();
            dc.DrawTextLayout(
                origin,
                &self.layout,
                &brush.handle(),
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT | D2D1_DRAW_TEXT_OPTIONS_CLIP,
            );
        }
    }

    #[inline]
    pub fn format(&self) -> &TextFormat {
        &self.format
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    pub fn alignment(&self) -> TextAlignment {
        unsafe { self.layout.GetTextAlignment().try_into().unwrap() }
    }

    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }

    #[inline]
    pub fn set_alignment(&self, alignment: TextAlignment) {
        unsafe {
            self.layout.SetTextAlignment(alignment as _).unwrap();
        }
    }

    #[inline]
    pub fn reset_size(&self) {
        let size: Size = unsafe {
            let metrics = self.layout.GetMetrics().unwrap();
            (metrics.width, metrics.height).into()
        };
        self.set_size(size);
    }

    #[inline]
    pub fn set_size(&self, size: impl Into<Size>) {
        unsafe {
            let size = size.into();
            self.layout.SetMaxWidth(size.width).unwrap();
            self.layout.SetMaxHeight(size.height).unwrap();
        }
    }

    #[inline]
    pub fn hit_test(&self, pt: impl Into<Point>) -> HitTestResult {
        unsafe {
            let pt = pt.into();
            let mut trailing_hit = BOOL(0);
            let mut inside = BOOL(0);
            let mut matrics = DWRITE_HIT_TEST_METRICS::default();
            self.layout
                .HitTestPoint(pt.x, pt.y, &mut trailing_hit, &mut inside, &mut matrics)
                .unwrap();
            HitTestResult {
                text_position: matrics.textPosition as _,
                inside: inside.as_bool(),
                trailing_hit: trailing_hit.as_bool(),
            }
        }
    }
}

impl PartialEq for TextLayout {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.layout == other.layout
    }
}

impl Eq for TextLayout {}

unsafe impl Send for TextLayout {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_file() {
        let factory: IDWriteFactory5 = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory5::IID)
                .unwrap()
                .cast()
                .unwrap()
        };
        TextFormat::new(
            &factory,
            &Font::file(
                "./test_resource/Inconsolata/Inconsolata-VariableFont_wdth,wght.ttf",
                "Inconsolata",
            ),
            14.0,
            None,
        )
        .unwrap();
    }

    #[test]
    fn hit_test() {
        let factory: IDWriteFactory5 = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory5::IID)
                .unwrap()
                .cast()
                .unwrap()
        };
        let format =
            TextFormat::new(&factory, &Font::system("Meiryo"), FontPoint(14.0).0, None).unwrap();
        let layout =
            TextLayout::new(&factory, "abcd", &format, TextAlignment::Leading, None).unwrap();
        let size = layout.size();
        assert!(
            layout.hit_test([0.0, 0.0])
                == HitTestResult {
                    text_position: 0,
                    inside: true,
                    trailing_hit: false,
                }
        );
        assert!(
            layout.hit_test([size.width - 0.1, 0.0])
                == HitTestResult {
                    text_position: 3,
                    inside: true,
                    trailing_hit: true,
                }
        );
        assert!(
            layout.hit_test([-100.0, 0.0])
                == HitTestResult {
                    text_position: 0,
                    inside: false,
                    trailing_hit: false,
                }
        );
    }
}
