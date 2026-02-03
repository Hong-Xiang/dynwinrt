use crate::{WinRTType, abi::AbiType};

#[derive(Debug)]
pub enum Error {
    ExpectObjectTypeError(WinRTType),
    InvalidType(WinRTType, WinRTType),
    InvalidNestedOutType(WinRTType),
    InvalidTypeAbiToWinRT(WinRTType, AbiType),
    WindowsError(windows_core::Error),
}

impl Error {
    pub fn expect_object_type(actual: WinRTType) -> Self {
        Error::ExpectObjectTypeError(actual)
    }
}

impl From<windows::core::Error> for Error {
    fn from(value: windows::core::Error) -> Self {
        Self::WindowsError(value)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
