pub mod windows {
    pub use ::windows::Win32::Foundation::*;
}

use self::windows::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum ErrorKind {
    Ok = S_OK.0,
    False = S_FALSE.0,
    Fail = E_FAIL.0,
    Abort = E_ABORT.0,
    AccessDenied = E_ACCESSDENIED.0,
    InvalidArg = E_INVALIDARG.0,
    NoInterface = E_NOINTERFACE.0,
    NotImpl = E_NOTIMPL.0,
    OutOfMemory = E_OUTOFMEMORY.0,
    Pointer = E_POINTER.0,
    BadNumber = D2DERR_BAD_NUMBER.0,
    WrongState = D2DERR_WRONG_STATE.0,
    ZeroVector = D2DERR_ZERO_VECTOR.0,
    CyclicGraph = D2DERR_CYCLIC_GRAPH.0,
    InvalidCall = D2DERR_INVALID_CALL.0,
    WrongFactory = D2DERR_WRONG_FACTORY.0,
    InternalError = D2DERR_INTERNAL_ERROR.0,
    InvalidTarget = D2DERR_INVALID_TARGET.0,
    ScannerFailed = D2DERR_SCANNER_FAILED.0,
    NotInitialized = D2DERR_NOT_INITIALIZED.0,
    RecreateTarget = D2DERR_RECREATE_TARGET.0,
    InvalidProperty = D2DERR_INVALID_PROPERTY.0,
    NoSubproperties = D2DERR_NO_SUBPROPERTIES.0,
    PrintJobClosed = D2DERR_PRINT_JOB_CLOSED.0,
    BitmapCannotDraw = D2DERR_BITMAP_CANNOT_DRAW.0,
    NoHardwareDevice = D2DERR_NO_HARDWARE_DEVICE.0,
    InvalidGlyphImage = D2DERR_INVALID_GLYPH_IMAGE.0,
    PushPopUnbalanced = D2DERR_PUSH_POP_UNBALANCED.0,
    UnsupportedVersion = D2DERR_UNSUPPORTED_VERSION.0,
    LayerAleradyInUse = D2DERR_LAYER_ALREADY_IN_USE.0,
    ScreenAccessDenied = D2DERR_SCREEN_ACCESS_DENIED.0,
    DisplayStateInvalid = D2DERR_DISPLAY_STATE_INVALID.0,
    ShaderCompileFailed = D2DERR_SHADER_COMPILE_FAILED.0,
    UnsupportedOperation = D2DERR_UNSUPPORTED_OPERATION.0,
    WrongResourceDomain = D2DERR_WRONG_RESOURCE_DOMAIN.0,
    BitmapBoundAsTarget = D2DERR_BITMAP_BOUND_AS_TARGET.0,
    IntermediateTooLarge = D2DERR_INTERMEDIATE_TOO_LARGE.0,
    ExceedsMaxBitmapSize = D2DERR_EXCEEDS_MAX_BITMAP_SIZE.0,
    EffectIsNotRegistered = D2DERR_EFFECT_IS_NOT_REGISTERED.0,
    IncompatibleBrushTypes = D2DERR_INCOMPATIBLE_BRUSH_TYPES.0,
    TooManyShaderElements = D2DERR_TOO_MANY_SHADER_ELEMENTS.0,
    MaxTextureSizeExceeded = D2DERR_MAX_TEXTURE_SIZE_EXCEEDED.0,
    OriginalTargetNotBound = D2DERR_ORIGINAL_TARGET_NOT_BOUND.0,
    TargetNotGdiCompatbile = D2DERR_TARGET_NOT_GDI_COMPATIBLE.0,
    TextEffectIsWrongType = D2DERR_TEXT_EFFECT_IS_WRONG_TYPE.0,
    TooManyTransformInputs = D2DERR_TOO_MANY_TRANSFORM_INPUTS.0,
    PrintFormatNotSupported = D2DERR_PRINT_FORMAT_NOT_SUPPORTED.0,
    TextRendererNotReleased = D2DERR_TEXT_RENDERER_NOT_RELEASED.0,
    InvalidGraphConfiguration = D2DERR_INVALID_GRAPH_CONFIGURATION.0,
    PopCallDidNotMatchPush = D2DERR_POP_CALL_DID_NOT_MATCH_PUSH.0,
    DisplayFormatNotSupported = D2DERR_DISPLAY_FORMAT_NOT_SUPPORTED.0,
    OutstandingBitmapReferences = D2DERR_OUTSTANDING_BITMAP_REFERENCES.0,
    InsufficientDeviceCapabilities = D2DERR_INSUFFICIENT_DEVICE_CAPABILITIES.0,
    RenderTargetHasLayerOrClipRect = D2DERR_RENDER_TARGET_HAS_LAYER_OR_CLIPRECT.0,
    InvalidInternalGraphConfiguration = D2DERR_INVALID_INTERNAL_GRAPH_CONFIGURATION.0,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Error(pub ::windows::core::Error);

impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<::windows::core::Error> for Error {
    fn from(src: ::windows::core::Error) -> Self {
        Self(src)
    }
}

impl From<ErrorKind> for Error {
    fn from(src: ErrorKind) -> Self {
        Self(::windows::core::HRESULT(src as i32).into())
    }
}

impl PartialEq<ErrorKind> for Error {
    fn eq(&self, rhs: &ErrorKind) -> bool {
        self.0.code().0 == *rhs as i32
    }
}

impl PartialEq<Error> for ErrorKind {
    fn eq(&self, rhs: &Error) -> bool {
        rhs == self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_test() {
        assert!(Error::from(ErrorKind::Ok) == ErrorKind::Ok);
        assert!(ErrorKind::Ok == Error::from(ErrorKind::Ok));
        assert!(Error::from(ErrorKind::Ok) != ErrorKind::Fail);
    }
}
