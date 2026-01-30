use crate::WinRTType;

pub enum Error {
    InvalidType(WinRTType, WinRTType),
    WindowsError(windows_core::Error),
}

impl From<windows::core::Error> for Error {
    fn from(value: windows::core::Error) -> Self {
        Self::WindowsError(value)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
