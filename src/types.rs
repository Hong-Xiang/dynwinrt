use windows_core::{IUnknown, Interface};

use crate::value::{AbiValue, WinRTValue};

#[derive(Debug, Clone, Copy)]
pub enum WinRTType {
    I32,
    I64,
    Object,
    HString,
    HResult,
    Pointer,
}


impl WinRTType {
    pub fn abi_type(&self) -> AbiType {
        match self {
            WinRTType::I32 | WinRTType::HResult => AbiType::I32,
            WinRTType::I64 => AbiType::I64,
            WinRTType::Object | WinRTType::HString | WinRTType::Pointer => AbiType::Ptr,
        }
    }

    pub fn from_out_value(&self, out: &AbiValue) -> WinRTValue {
        match (self, out) {
            (WinRTType::I32, AbiValue::I32(i)) => WinRTValue::I32(*i),
            (WinRTType::Object, AbiValue::Pointer(p)) => {
                WinRTValue::Object(unsafe { IUnknown::from_raw(*p) })
            }
            (WinRTType::HString, AbiValue::Pointer(p)) => {
                WinRTValue::HString(unsafe { core::mem::transmute(*p) })
            }
            (WinRTType::HResult, AbiValue::I32(hr)) => {
                WinRTValue::HResult(windows_core::HRESULT(*hr))
            }
            (WinRTType::Pointer, AbiValue::Pointer(p)) => WinRTValue::Pointer(*p),
            (WinRTType::I64, AbiValue::I64(i)) => WinRTValue::I64(*i),
            _ => panic!("Mismatched out value type"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AbiType {
    I32,
    I64,
    Ptr,
}

impl AbiType {
    pub fn default_value(&self) -> AbiValue {
        match self {
            AbiType::I32 => AbiValue::I32(0),
            AbiType::I64 => AbiValue::I64(0),
            AbiType::Ptr => AbiValue::Pointer(std::ptr::null_mut()),
        }
    }
    pub fn libffi_type(&self) -> libffi::middle::Type {
        match self {
            AbiType::I32 => libffi::middle::Type::i32(),
            AbiType::I64 => libffi::middle::Type::i64(),
            AbiType::Ptr => libffi::middle::Type::pointer(),
        }
    }
}
