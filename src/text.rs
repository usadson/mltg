use crate::*;
use std::convert::TryInto;
use windows::core::{Interface, HSTRING};
use windows::Win32::{Foundation::*, Graphics::DirectWrite::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum FontWeight {
    Thin = DWRITE_FONT_WEIGHT_THIN.0,
    UltraLight = DWRITE_FONT_WEIGHT_ULTRA_LIGHT.0,
    Light = DWRITE_FONT_WEIGHT_LIGHT.0,
    SemiLight = DWRITE_FONT_WEIGHT_SEMI_LIGHT.0,
    Regular = DWRITE_FONT_WEIGHT_REGULAR.0,
    Medium = DWRITE_FONT_WEIGHT_MEDIUM.0,
    SemiBold = DWRITE_FONT_WEIGHT_SEMI_BOLD.0,
    Bold = DWRITE_FONT_WEIGHT_BOLD.0,
    UltraBold = DWRITE_FONT_WEIGHT_ULTRA_BOLD.0,
    Heavy = DWRITE_FONT_WEIGHT_HEAVY.0,
    UltraBlack = DWRITE_FONT_WEIGHT_ULTRA_BLACK.0,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum FontStyle {
    Normal = DWRITE_FONT_STYLE_NORMAL.0,
    Oblique = DWRITE_FONT_STYLE_OBLIQUE.0,
    Italic = DWRITE_FONT_STYLE_ITALIC.0,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum FontStretch {
    Undefined = DWRITE_FONT_STRETCH_UNDEFINED.0,
    UltraCondensed = DWRITE_FONT_STRETCH_ULTRA_CONDENSED.0,
    ExtraCondensed = DWRITE_FONT_STRETCH_EXTRA_CONDENSED.0,
    Condensed = DWRITE_FONT_STRETCH_CONDENSED.0,
    SemiCondensed = DWRITE_FONT_STRETCH_SEMI_CONDENSED.0,
    Medium = DWRITE_FONT_STRETCH_MEDIUM.0,
    SemiExpanded = DWRITE_FONT_STRETCH_SEMI_EXPANDED.0,
    Expanded = DWRITE_FONT_STRETCH_EXPANDED.0,
    ExtraExpanded = DWRITE_FONT_STRETCH_EXTRA_EXPANDED.0,
    UltraExpanded = DWRITE_FONT_STRETCH_ULTRA_EXPANDED.0,
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
    Leading = DWRITE_TEXT_ALIGNMENT_LEADING.0,
    Center = DWRITE_TEXT_ALIGNMENT_CENTER.0,
    Trailing = DWRITE_TEXT_ALIGNMENT_TRAILING.0,
    Justified = DWRITE_TEXT_ALIGNMENT_JUSTIFIED.0,
}

impl std::convert::TryFrom<DWRITE_TEXT_ALIGNMENT> for TextAlignment {
    type Error = ();

    fn try_from(src: DWRITE_TEXT_ALIGNMENT) -> core::result::Result<Self, ()> {
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

#[derive(Clone)]
pub enum Font<'a, 'b> {
    System(&'a str),
    File(&'a std::path::Path, &'b str),
    Memory(&'a [u8], &'b str),
}

impl<'a, 'b> Font<'a, 'b> {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Self::System(name) => name,
            Self::File(_, name) => name,
            Self::Memory(_, name) => name,
        }
    }
}

impl<'a, 'b> std::fmt::Debug for Font<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::System(font_name) => write!(f, "Font::System({})", font_name),
            Self::File(path, font_name) => {
                write!(f, "Font::File({}, {})", path.display(), font_name)
            }
            Self::Memory(_, font_name) => write!(f, "Font::Memory({})", font_name),
        }
    }
}

#[derive(Clone)]
pub struct TextFormat {
    format: IDWriteTextFormat,
    name: String,
    size: f32,
    style: TextStyle,
}

impl TextFormat {
    #[inline]
    pub(crate) fn new(
        factory: &IDWriteFactory5,
        in_memory_loader: &IDWriteInMemoryFontFileLoader,
        font: Font,
        size: f32,
        style: Option<&TextStyle>,
    ) -> Result<Self> {
        let style = style.cloned().unwrap_or_default();
        let (font_name, font_collection): (_, Option<IDWriteFontCollection>) = match font {
            Font::System(font_name) => (font_name, None),
            Font::File(path, font_name) => unsafe {
                let set_builder: IDWriteFontSetBuilder1 = factory.CreateFontSetBuilder()?.cast()?;
                let font_file = factory.CreateFontFileReference(
                    &HSTRING::from(path.to_string_lossy().as_ref()),
                    std::ptr::null(),
                )?;
                set_builder.AddFontFile(&font_file)?;
                let font_set = set_builder.CreateFontSet()?;
                let font_collection = factory.CreateFontCollectionFromFontSet(&font_set)?;
                (font_name, Some(font_collection.into()))
            },
            Font::Memory(data, font_name) => unsafe {
                let set_builder: IDWriteFontSetBuilder1 = factory.CreateFontSetBuilder()?.cast()?;
                let font_file = in_memory_loader.CreateInMemoryFontFileReference(
                    factory,
                    data.as_ptr() as _,
                    data.len() as _,
                    None,
                )?;
                set_builder.AddFontFile(&font_file)?;
                let font_set = set_builder.CreateFontSet()?;
                let font_collection = factory.CreateFontCollectionFromFontSet(&font_set)?;
                (font_name, Some(font_collection.into()))
            },
        };
        let format = unsafe {
            factory.CreateTextFormat(
                &HSTRING::from(font_name),
                font_collection.as_ref(),
                DWRITE_FONT_WEIGHT(style.weight as _),
                DWRITE_FONT_STYLE(style.style as _),
                DWRITE_FONT_STRETCH(style.stretch as _),
                size,
                windows::w!(""),
            )?
        };
        Ok(Self {
            format,
            name: font_name.to_string(),
            size,
            style,
        })
    }

    #[inline]
    pub fn font_name(&self) -> &str {
        &self.name
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
    _typography: IDWriteTypography,
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
    ) -> Result<Self> {
        let layout = unsafe {
            let text = text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            factory.CreateTextLayout(&text, &format.format, std::f32::MAX, std::f32::MAX)?
        };
        let typography = unsafe {
            let typography = factory.CreateTypography()?;
            let feature = DWRITE_FONT_FEATURE {
                nameTag: DWRITE_FONT_FEATURE_TAG_STANDARD_LIGATURES,
                parameter: 0,
            };
            typography.AddFontFeature(feature)?;
            let range = DWRITE_TEXT_RANGE {
                startPosition: 0,
                length: text.chars().count() as _,
            };
            layout.SetTypography(&typography, range)?;
            typography
        };
        let max_size = unsafe {
            let size = size.unwrap_or_else(|| {
                let metrics = layout.GetMetrics().unwrap();
                (metrics.width, metrics.height).into()
            });
            layout
                .SetTextAlignment(DWRITE_TEXT_ALIGNMENT(alignment as _))
                .unwrap();
            layout
                .SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)
                .unwrap();
            layout.SetMaxWidth(size.width).unwrap();
            layout.SetMaxHeight(size.height).unwrap();
            size
        };
        Ok(Self {
            layout,
            _typography: typography,
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
            self.layout
                .SetTextAlignment(DWRITE_TEXT_ALIGNMENT(alignment as _))
                .unwrap();
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

    #[inline]
    pub fn text_position_to_point(&self, position: usize, trailing_hit: bool) -> Point {
        unsafe {
            let mut point = point(0.0, 0.0);
            let mut metrics = DWRITE_HIT_TEST_METRICS::default();
            self.layout
                .HitTestTextPosition(
                    position as _,
                    trailing_hit,
                    &mut point.x,
                    &mut point.y,
                    &mut metrics,
                )
                .unwrap();
            point
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
        let loader = unsafe {
            let loader = factory.CreateInMemoryFontFileLoader().unwrap();
            factory.RegisterFontFileLoader(&loader).unwrap();
            loader
        };
        TextFormat::new(
            &factory,
            &loader,
            Font::File(
                std::path::Path::new(
                    "./test_resource/Inconsolata/Inconsolata-VariableFont_wdth,wght.ttf",
                ),
                "Inconsolata",
            ),
            14.0,
            None,
        )
        .unwrap();
    }

    #[test]
    fn from_memory() {
        let factory: IDWriteFactory5 = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory5::IID)
                .unwrap()
                .cast()
                .unwrap()
        };
        let loader = unsafe {
            let loader = factory.CreateInMemoryFontFileLoader().unwrap();
            factory.RegisterFontFileLoader(&loader).unwrap();
            loader
        };
        TextFormat::new(
            &factory,
            &loader,
            Font::Memory(
                include_bytes!(
                    "../test_resource/Inconsolata/Inconsolata-VariableFont_wdth,wght.ttf"
                ),
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
        let loader = unsafe {
            let loader = factory.CreateInMemoryFontFileLoader().unwrap();
            factory.RegisterFontFileLoader(&loader).unwrap();
            loader
        };
        let format = TextFormat::new(
            &factory,
            &loader,
            Font::System("Meiryo"),
            FontPoint(14.0).0,
            None,
        )
        .unwrap();
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
