use std::mem::{MaybeUninit, zeroed};

use libffi::middle::Arg;
use windows::Win32::System::WinRT;
use windows_core::{ComObject, ComObjectInner, GUID, IInspectable, IUnknown, Interface};

use crate::WinRTType;

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

    pub fn cast(&self, iid: &GUID) -> windows::core::Result<WinRTValue> {
        match self {
            WinRTValue::Object(obj) => {
                let mut result = std::ptr::null_mut();
                unsafe { obj.query(iid, &mut result) }.ok()?;
                Ok(WinRTValue::Object(unsafe { IUnknown::from_raw(result) }))
            }
            _ => panic!("Can only cast WinRTValue::Object"),
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

#[derive(Debug)]
pub enum AbiValue {
    I32(i32),
    I64(i64),
    Pointer(*mut std::ffi::c_void),
}

impl AbiValue {
    pub fn as_out_ptr(&self) -> *const std::ffi::c_void {
        match self {
            AbiValue::I32(i) => std::ptr::from_ref(i).cast(),
            AbiValue::I64(i) => std::ptr::from_ref(i).cast(),
            AbiValue::Pointer(p) => std::ptr::from_ref(p).cast(),
        }
    }
}
