use libffi::middle::Arg;
use windows_core::{GUID, IUnknown, Interface};
use windows_future::IAsyncInfo;

use crate::{
    WinRTType,
    call::{self, call_winrt_method_2},
    result,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WinRTValue {
    I32(i32),
    I64(i64),
    Object(IUnknown),
    HString(windows_core::HSTRING),
    HResult(windows_core::HRESULT),
    OutValue(*mut std::ffi::c_void, WinRTType),
    IAsyncOperation(IAsyncInfo, GUID),
}
unsafe impl Send for WinRTValue {}
unsafe impl Sync for WinRTValue {}

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
            WinRTValue::IAsyncOperation(op, _) => Some(op.cast().unwrap()),
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
            _ => Err(result::Error::ExpectObjectTypeError(self.get_type())),
        }
    }

    pub fn call_single_out(
        &self,
        method_index: usize,
        typ: &WinRTType,
        args: &[WinRTValue],
    ) -> result::Result<WinRTValue> {
        match self {
            WinRTValue::Object(obj) => {
                let mut result = std::ptr::null_mut();
                let hr = match args {
                    [] => call::call_winrt_method_1(method_index, obj.as_raw(), &mut result),
                    [WinRTValue::I32(n)] => {
                        call_winrt_method_2(method_index, obj.as_raw(), n, &mut result)
                    }
                    [WinRTValue::I64(n)] => {
                        call_winrt_method_2(method_index, obj.as_raw(), n, &mut result)
                    }
                    _ => panic!("Unsupported number of arguments"),
                };
                hr.ok().map_err(|e| result::Error::WindowsError(e))?;
                Ok(typ.from_out(result).unwrap())
            }
            _ => Err(result::Error::ExpectObjectTypeError(self.get_type())),
        }
    }

    pub fn get_type(&self) -> crate::WinRTType {
        match self {
            WinRTValue::I32(_) => crate::WinRTType::I32,
            WinRTValue::I64(_) => crate::WinRTType::I64,
            WinRTValue::Object(_) => crate::WinRTType::Object,
            WinRTValue::HString(_) => crate::WinRTType::HString,
            WinRTValue::HResult(_) => crate::WinRTType::HResult,
            WinRTValue::OutValue(_, typ) => crate::WinRTType::OutValue(Box::new(typ.clone())),
            WinRTValue::IAsyncOperation(_, iid) => {
                crate::WinRTType::IAsyncOperation(*iid)
            }
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
            WinRTValue::OutValue(p, _) => arg(p),
            WinRTValue::IAsyncOperation(info, _) => panic!("Not supported"),
        }
    }
}
