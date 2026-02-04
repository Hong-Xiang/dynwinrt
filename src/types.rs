use windows_core::{GUID, IUnknown, Interface};

use crate::abi::{AbiType, AbiValue};
use crate::value::WinRTValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WinRTType {
    I32,
    I64,
    Object,
    HString,
    HResult,
    OutValue(Box<WinRTType>),
    ArrayOfIUnknown,
    IAsyncOperation(GUID)
}

impl WinRTType {
    pub fn abi_type(&self) -> AbiType {
        match self {
            WinRTType::I32 | WinRTType::HResult => AbiType::I32,
            WinRTType::I64 => AbiType::I64,
            WinRTType::Object | WinRTType::HString | WinRTType::OutValue(_) | WinRTType::IAsyncOperation(_) | WinRTType::ArrayOfIUnknown => AbiType::Ptr,
        }
    }

    pub fn default_value(&self) -> WinRTValue {
        match self {
            WinRTType::I32 => WinRTValue::I32(0),
            WinRTType::I64 => WinRTValue::I64(0),
            WinRTType::Object => WinRTValue::Object(unsafe { IUnknown::from_raw(std::ptr::null_mut()) }),
            WinRTType::HString => WinRTValue::HString(windows_core::HSTRING::new()),
            WinRTType::HResult => WinRTValue::HResult(windows_core::HRESULT(0)),
            WinRTType::OutValue(_) => WinRTValue::OutValue(std::ptr::null_mut(), self.clone()),
            WinRTType::IAsyncOperation(guid) => panic!("Cannot create default value for IAsyncOperation {:?}", guid),
            WinRTType::ArrayOfIUnknown => WinRTValue::ArrayOfIUnknown(crate::value::ArrayOfIUnknownData(windows::core::Array::new())),
        }
    }

    pub fn from_out(&self, ptr: *mut std::ffi::c_void) -> crate::result::Result<WinRTValue> {
        unsafe {
            match &self {
                WinRTType::I32 => Ok(WinRTValue::I32(*(ptr as *mut i32))),
                WinRTType::I64 => Ok(WinRTValue::I64(*(ptr as *mut i64))),
                WinRTType::Object => Ok(WinRTValue::Object(IUnknown::from_raw(ptr))),
                WinRTType::HString => Ok(WinRTValue::HString(std::mem::transmute(ptr))),
                WinRTType::HResult => Ok(WinRTValue::HResult(windows_core::HRESULT(
                    *(ptr as *mut i32),
                ))),
                WinRTType::IAsyncOperation(GUID) => {
                    Ok(WinRTValue::IAsyncOperation(IUnknown::from_raw(ptr).cast()?, *GUID))
                }
                _ => Err(crate::result::Error::InvalidTypeAbiToWinRT(
                    self.clone(),
                    AbiType::Ptr,
                )),
            }
        }
    }

    pub fn from_out_value(&self, out: &AbiValue) -> crate::result::Result<WinRTValue> {
        use crate::result::Error;
        match (self, out) {
            (WinRTType::I32, AbiValue::I32(i)) => Ok(WinRTValue::I32(*i)),
            (WinRTType::I64, AbiValue::I64(i)) => Ok(WinRTValue::I64(*i)),
            (WinRTType::Object, AbiValue::Pointer(p)) => {
                Ok(WinRTValue::Object(unsafe { IUnknown::from_raw(*p) }))
            }
            (WinRTType::HString, AbiValue::Pointer(p)) => {
                Ok(WinRTValue::HString(unsafe { core::mem::transmute(*p) }))
            }
            (WinRTType::HResult, AbiValue::I32(hr)) => {
                Ok(WinRTValue::HResult(windows_core::HRESULT(*hr)))
            }
            (WinRTType::OutValue(_), _) => Err(Error::InvalidNestedOutType(self.clone())),
            _ => Err(crate::result::Error::InvalidTypeAbiToWinRT(
                self.clone(),
                out.abi_type(),
            )),
        }
    }
}
