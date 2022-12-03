use windows::Win32::Foundation::D2DERR_RECREATE_TARGET;

#[derive(Clone, PartialEq, Eq, Debug, thiserror::Error)]
pub enum Error {
    #[error("RecreateTarget")]
    RecreateTarget,
    #[error("{0}")]
    Other(windows::core::Error),
}

impl From<windows::core::Error> for Error {
    fn from(src: windows::core::Error) -> Self {
        if src.code() == D2DERR_RECREATE_TARGET {
            Self::RecreateTarget
        } else {
            Self::Other(src)
        }
    }
}

impl From<windows::core::HRESULT> for Error {
    fn from(src: windows::core::HRESULT) -> Self {
        match src {
            D2DERR_RECREATE_TARGET => Self::RecreateTarget,
            _ => Self::Other(src.into()),
        }
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;
