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

    pub fn abi_type(&self) -> AbiType {
        match self {
            AbiValue::I32(_) => AbiType::I32,
            AbiValue::I64(_) => AbiType::I64,
            AbiValue::Pointer(_) => AbiType::Ptr,
        }
    }
}
