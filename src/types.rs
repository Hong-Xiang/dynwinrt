use windows_core::{IUnknown, Interface};

use crate::value::{OutValue, WinRTValue};

#[derive(Debug)]
pub enum WinRTType {
    I32,
    Object,
    HString,
    HResult,
    Pointer,
}

impl WinRTType {
    pub fn new_out_value(&self) -> OutValue {
        match self {
            WinRTType::I32 => OutValue::I32(0),
            WinRTType::Object => OutValue::Pointer(std::ptr::null_mut()),
            WinRTType::HString => OutValue::Pointer(std::ptr::null_mut()),
            WinRTType::HResult => OutValue::I32(0),
            WinRTType::Pointer => OutValue::Pointer(std::ptr::null_mut()),
        }
    }

    fn abi_type(&self) -> AbiType {
        match self {
            WinRTType::I32 | WinRTType::HResult => AbiType::I32,
            WinRTType::Object | WinRTType::HString | WinRTType::Pointer => AbiType::Ptr,
        }
    }

    pub fn libffi_type(&self) -> libffi::middle::Type {
        match self {
            WinRTType::I32 => libffi::middle::Type::i32(),
            WinRTType::Object => libffi::middle::Type::pointer(),
            WinRTType::HString => libffi::middle::Type::pointer(),
            WinRTType::HResult => libffi::middle::Type::i32(),
            WinRTType::Pointer => libffi::middle::Type::pointer(),
        }
    }

    pub fn from_out_value(&self, out: &OutValue) -> WinRTValue {
        match (self, out) {
            (WinRTType::I32, OutValue::I32(i)) => WinRTValue::I32(*i),
            (WinRTType::Object, OutValue::Pointer(p)) => {
                WinRTValue::Object(unsafe { IUnknown::from_raw(*p) })
            }
            (WinRTType::HString, OutValue::Pointer(p)) => {
                WinRTValue::HString(unsafe { core::mem::transmute(*p) })
            }
            (WinRTType::HResult, OutValue::I32(hr)) => {
                WinRTValue::HResult(windows_core::HRESULT(*hr))
            }
            (WinRTType::Pointer, OutValue::Pointer(p)) => WinRTValue::Pointer(*p),
            _ => panic!("Mismatched out value type"),
        }
    }
}

#[derive(Debug)]
enum AbiType {
    I32,
    Ptr,
}

impl AbiType {
    pub fn libffi_type(&self) -> libffi::middle::Type {
        match self {
            AbiType::I32 => libffi::middle::Type::i32(),
            AbiType::Ptr => libffi::middle::Type::pointer(),
        }
    }
}
