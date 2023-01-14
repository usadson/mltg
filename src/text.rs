use crate::*;
use windows::core::{Interface, HSTRING};
use windows::Win32::{Foundation::*, Graphics::Direct2D::*, Graphics::DirectWrite::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(i32)]
pub enum FontStyle {
    Normal = DWRITE_FONT_STYLE_NORMAL.0,
    Oblique = DWRITE_FONT_STYLE_OBLIQUE.0,
    Italic = DWRITE_FONT_STYLE_ITALIC.0,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TextStyle {
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
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

impl From<DWRITE_TEXT_ALIGNMENT> for TextAlignment {
    #[inline]
    fn from(src: DWRITE_TEXT_ALIGNMENT) -> Self {
        match src {
            DWRITE_TEXT_ALIGNMENT_LEADING => TextAlignment::Leading,
            DWRITE_TEXT_ALIGNMENT_CENTER => TextAlignment::Center,
            DWRITE_TEXT_ALIGNMENT_TRAILING => TextAlignment::Trailing,
            DWRITE_TEXT_ALIGNMENT_JUSTIFIED => TextAlignment::Justified,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineSpacingMethod {
    Default,
    Uniform,
    Proportional,
}

impl From<DWRITE_LINE_SPACING_METHOD> for LineSpacingMethod {
    fn from(src: DWRITE_LINE_SPACING_METHOD) -> Self {
        match src {
            DWRITE_LINE_SPACING_METHOD_DEFAULT => LineSpacingMethod::Default,
            DWRITE_LINE_SPACING_METHOD_UNIFORM => LineSpacingMethod::Uniform,
            DWRITE_LINE_SPACING_METHOD_PROPORTIONAL => LineSpacingMethod::Proportional,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LineSpacing {
    pub method: LineSpacingMethod,
    pub height: f32,
    pub baseline: f32,
}

#[derive(Clone, Debug)]
pub enum Font<'a, 'b> {
    System(&'a str),
    File(&'a std::path::Path, &'b str),
    Memory(&'a [u8], &'b str),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HitTestResult {
    pub text_position: usize,
    pub inside: bool,
    pub trailing_hit: bool,
}

#[derive(Debug)]
pub struct TextFormat {
    format: IDWriteTextFormat,
    _not_sync: std::cell::UnsafeCell<()>, // !Sync
}

impl TextFormat {
    pub(crate) fn new(
        factory: &IDWriteFactory6,
        loader: &IDWriteInMemoryFontFileLoader,
        font: Font,
        size: f32,
        style: Option<&TextStyle>,
        locale: &str,
    ) -> Result<Self> {
        let (font_name, font_collection): (_, Option<IDWriteFontCollection>) = match font {
            Font::System(name) => (name, None),
            Font::File(path, name) => unsafe {
                let set_builder: IDWriteFontSetBuilder1 = factory.CreateFontSetBuilder()?.cast()?;
                let font_file = factory.CreateFontFileReference(
                    &HSTRING::from(path.to_string_lossy().as_ref()),
                    None,
                )?;
                set_builder.AddFontFile(&font_file)?;
                let font_set = set_builder.CreateFontSet()?;
                let font_collection = factory.CreateFontCollectionFromFontSet(&font_set)?;
                (name, Some(font_collection.into()))
            },
            Font::Memory(data, name) => unsafe {
                let set_builder: IDWriteFontSetBuilder1 = factory.CreateFontSetBuilder()?.cast()?;
                let font_file = loader.CreateInMemoryFontFileReference(
                    factory,
                    data.as_ptr() as _,
                    data.len() as _,
                    None,
                )?;
                set_builder.AddFontFile(&font_file)?;
                let font_set = set_builder.CreateFontSet()?;
                let font_collection = factory.CreateFontCollectionFromFontSet(&font_set)?;
                (name, Some(font_collection.into()))
            },
        };
        let style = style.cloned().unwrap_or_default();
        let format = unsafe {
            factory.CreateTextFormat(
                &HSTRING::from(font_name),
                font_collection.as_ref(),
                DWRITE_FONT_WEIGHT(style.weight as _),
                DWRITE_FONT_STYLE(style.style as _),
                DWRITE_FONT_STRETCH(style.stretch as _),
                size,
                &HSTRING::from(locale),
            )?
        };
        Ok(Self {
            format,
            _not_sync: std::cell::UnsafeCell::new(()),
        })
    }

    pub fn line_spacing(&self) -> Result<LineSpacing> {
        let mut method = DWRITE_LINE_SPACING_METHOD_DEFAULT;
        let mut height = 0.0;
        let mut baseline = 0.0;

        unsafe {
            self.format.GetLineSpacing(&mut method, &mut height, &mut baseline)?;
        }

        Ok(LineSpacing{
            method: method.into(),
            height,
            baseline
        })
    }
}

impl PartialEq for TextFormat {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
    }
}

impl Eq for TextFormat {}

impl Clone for TextFormat {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            format: self.format.clone(),
            _not_sync: std::cell::UnsafeCell::new(()),
        }
    }
}

#[derive(Debug)]
pub struct TextLayout {
    layout: IDWriteTextLayout,
    format: TextFormat,
    typography: IDWriteTypography,
    size: Size<f32>,
    _not_sync: std::cell::UnsafeCell<()>, // !Sync
}

impl TextLayout {
    pub(crate) fn new(
        factory: &IDWriteFactory6,
        text: &str,
        format: &TextFormat,
        alignment: TextAlignment,
        size: Option<Size<f32>>,
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
            typography,
            format: format.clone(),
            size: max_size,
            _not_sync: std::cell::UnsafeCell::new(()),
        })
    }

    #[inline]
    pub fn format(&self) -> &TextFormat {
        &self.format
    }

    #[inline]
    pub fn size(&self) -> Size<f32> {
        self.size
    }

    #[inline]
    pub fn reset_size(&mut self) {
        let size: Size<f32> = unsafe {
            let Ok(metrics) = self.layout.GetMetrics() else { return };
            (metrics.width, metrics.height).into()
        };
        self.set_size(size);
    }

    #[inline]
    pub fn set_size(&mut self, size: impl Into<Size<f32>>) {
        self.size = size.into();
        unsafe {
            self.layout.SetMaxWidth(self.size.width).unwrap_or(());
            self.layout.SetMaxHeight(self.size.height).unwrap_or(());
        }
    }

    #[inline]
    pub fn alignment(&self) -> TextAlignment {
        unsafe { self.layout.GetTextAlignment().into() }
    }

    #[inline]
    pub fn set_alignment(&self, align: TextAlignment) {
        unsafe {
            self.layout
                .SetTextAlignment(DWRITE_TEXT_ALIGNMENT(align as _))
                .unwrap_or(());
        }
    }

    #[inline]
    pub fn hit_test(&self, pt: impl Into<Point<f32>>) -> Result<HitTestResult> {
        let pt = pt.into();
        let mut trailing_hit = BOOL::default();
        let mut inside = BOOL::default();
        let mut metrics = DWRITE_HIT_TEST_METRICS::default();
        unsafe {
            self.layout
                .HitTestPoint(pt.x, pt.y, &mut trailing_hit, &mut inside, &mut metrics)?;
        }
        Ok(HitTestResult {
            text_position: metrics.textPosition as _,
            inside: inside.as_bool(),
            trailing_hit: trailing_hit.as_bool(),
        })
    }

    #[inline]
    pub fn text_position_to_point(
        &self,
        position: usize,
        trailing_hit: bool,
    ) -> Result<Point<f32>> {
        let mut point = Point::new(0.0, 0.0);
        let mut metrics = DWRITE_HIT_TEST_METRICS::default();
        unsafe {
            self.layout.HitTestTextPosition(
                position as _,
                trailing_hit,
                &mut point.x,
                &mut point.y,
                &mut metrics,
            )?;
        }
        Ok(point)
    }

    #[inline]
    pub fn position(&self, pt: impl Into<Point<f32>>) -> (&Self, Point<f32>) {
        (self, pt.into())
    }
}

impl PartialEq for TextLayout {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.layout == other.layout
    }
}

impl Eq for TextLayout {}

impl Clone for TextLayout {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            format: self.format.clone(),
            typography: self.typography.clone(),
            size: self.size,
            _not_sync: std::cell::UnsafeCell::new(()),
        }
    }
}

impl<T> Fill for (&TextLayout, T)
where
    T: Into<Point<f32>> + Clone,
{
    #[inline]
    fn fill(&self, dc: &ID2D1DeviceContext5, brush: &ID2D1Brush) {
        unsafe {
            let point = self.1.clone().into();
            dc.DrawTextLayout(
                Wrapper(point).into(),
                &self.0.layout,
                brush,
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT | D2D1_DRAW_TEXT_OPTIONS_CLIP,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_file() {
        let factory =
            unsafe { DWriteCreateFactory::<IDWriteFactory6>(DWRITE_FACTORY_TYPE_SHARED).unwrap() };
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
                    "./resources/Inconsolata/Inconsolata-VariableFont_wdth,wght.ttf",
                ),
                "Inconsolata",
            ),
            14.0,
            None,
            "",
        )
        .unwrap();
    }

    #[test]
    fn from_memory() {
        let factory =
            unsafe { DWriteCreateFactory::<IDWriteFactory6>(DWRITE_FACTORY_TYPE_SHARED).unwrap() };
        let loader = unsafe {
            let loader = factory.CreateInMemoryFontFileLoader().unwrap();
            factory.RegisterFontFileLoader(&loader).unwrap();
            loader
        };
        TextFormat::new(
            &factory,
            &loader,
            Font::Memory(
                include_bytes!("../resources/Inconsolata/Inconsolata-VariableFont_wdth,wght.ttf"),
                "Inconsolata",
            ),
            14.0,
            None,
            "",
        )
        .unwrap();
    }

    #[test]
    fn hit_test() {
        let factory =
            unsafe { DWriteCreateFactory::<IDWriteFactory6>(DWRITE_FACTORY_TYPE_SHARED).unwrap() };
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
            "",
        )
        .unwrap();
        let layout =
            TextLayout::new(&factory, "abcd", &format, TextAlignment::Leading, None).unwrap();
        let size = layout.size();
        assert!(
            layout.hit_test([0.0, 0.0]).unwrap()
                == HitTestResult {
                    text_position: 0,
                    inside: true,
                    trailing_hit: false,
                }
        );
        assert!(
            layout.hit_test([size.width - 0.1, 0.0]).unwrap()
                == HitTestResult {
                    text_position: 3,
                    inside: true,
                    trailing_hit: true,
                }
        );
        assert!(
            layout.hit_test([-100.0, 0.0]).unwrap()
                == HitTestResult {
                    text_position: 0,
                    inside: false,
                    trailing_hit: false,
                }
        );
    }
}
