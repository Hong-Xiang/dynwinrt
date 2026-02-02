use libffi::middle::Arg;
use windows_core::{GUID, IUnknown, Interface};

use crate::result;

#[derive(Debug, PartialEq, Eq)]
pub enum WinRTValue {
    I32(i32),
    I64(i64),
    Object(IUnknown),
    HString(windows_core::HSTRING),
    HResult(windows_core::HRESULT),
    Pointer(*mut std::ffi::c_void),
}

impl WinRTValue {
    pub fn as_hstring(&self) -> Option<windows_core::HSTRING> {
        match self {
            WinRTValue::HString(hstr) => Some((*hstr).clone()),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            WinRTValue::I32(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<IUnknown> {
        match self {
            WinRTValue::Object(obj) => Some(obj.clone()),
            _ => None,
        }
    }

    pub fn cast(&self, iid: &GUID) -> result::Result<WinRTValue> {
        match self {
            WinRTValue::Object(obj) => {
                let mut result = std::ptr::null_mut();
                unsafe { obj.query(iid, &mut result) }.ok()?;
                Ok(WinRTValue::Object(unsafe { IUnknown::from_raw(result) }))
            }
            _ => Err(result::Error::expect_object_type(self.get_type())),
        }
    }

    pub fn get_type(&self) -> crate::WinRTType {
        match self {
            WinRTValue::I32(_) => crate::WinRTType::I32,
            WinRTValue::I64(_) => crate::WinRTType::I64,
            WinRTValue::Object(_) => crate::WinRTType::Object,
            WinRTValue::HString(_) => crate::WinRTType::HString,
            WinRTValue::HResult(_) => crate::WinRTType::HResult,
            WinRTValue::Pointer(_) => crate::WinRTType::Pointer,
        }
    }

    pub fn libffi_arg(&self) -> Arg<'_> {
        use libffi::middle::arg;
        match &self {
            WinRTValue::Object(p) => arg(p),
            WinRTValue::HString(hstr) => arg(hstr),
            WinRTValue::HResult(hr) => arg(hr),
            WinRTValue::I32(i) => arg(i),
            WinRTValue::I64(i) => arg(i),
            WinRTValue::Pointer(p) => arg(p),
        }
    }
}
