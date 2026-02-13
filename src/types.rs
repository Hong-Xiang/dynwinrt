use windows_core::{GUID, IUnknown, Interface};
use windows_future::IAsyncOperation;

use crate::abi::{AbiType, AbiValue};
use crate::call::get_sig;
use crate::signature;
use crate::value::WinRTValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WinRTType {
    I32,
    I64,
    Object,
    Interface(GUID),
    HString,
    HResult,
    OutValue(Box<WinRTType>),
    ArrayOfIUnknown,
    IAsyncOperation(GUID),
    // Interface(GUID),
    // IGeneric(GUID, Box<WinRTType>),
}

// pub enum WType {
//     I32,
//     I64,
//     Interface(GUID), // non-generic interface
//     Generic {
//         piid: GUID,
//         arity: u32, // IAsyncOperation<T> 1, IAsyncOperationWithProgress<T1, T2> 2, IVector
//     }, // pure-generic one, like IAsyncOperation<>
//     ParameterizedConcrete(
//         Box<Self>, // need to be a generic type
//         Vec<Self>, // type arguments
//     ), // pinterface(self.0.Signature(), T1::Guid, T2::Guid, ...)
//     // IAsyncOperation(Box<Self>),
//     // ...
// }

// impl WType {

//     pub fn usage_demo(){
//         // Usage
//         let PickFileResult = WType::Interface(PickFileResult::IID); // leaf type, GUID <- winmd
//         let IAsyncOperationOfPickFileResult = WType::IAsyncOperation(Box::new(PickFileResult)); // IAsyncOpeartion<PickFileResult>
//         IAsyncOperation<int>


//         // IAsyncOperation<IAsyncOperation<PickFileResult>>
//         // let t = WType::ParameterizedConcrete(AsyncOperation, vec![PickerApiResult])
//     }

//     pub fn parameterized(&self, type_arguments: &[WType]) -> WType {
//         match self {
//             WType::Generic { piid, arity } => {
//                 if *arity as usize != type_arguments.len() {
//                     panic!(
//                         "Generic type arity {} does not match type arguments length {}",
//                         arity,
//                         type_arguments.len()
//                     );
//                 }
//                 WType::ParameterizedConcrete(Box::new(self.clone()), type_arguments.to_vec())
//             }
//             _ => panic!("Only generic types can be parameterized"),
//         }
//     }

//     fn get_signature<T: windows::core::RuntimeType>() -> windows::core::imp::ConstBuffer {
//         return T::SIGNATURE;
//     }

//     pub fn signature(&self) -> windows::core::imp::ConstBuffer {
//         match self {
//             WType::I32 => get_sig::<i32>(),
//             WType::I64 => get_sig::<i64>(),
//             WType::Interface(guid) => windows::core::imp::ConstBuffer::new(),
//             WType::Generic { piid, arity } => windows::core::imp::ConstBuffer::new()
//                 .append_guid(piid),
//             WType::ParameterizedConcrete(generic, type_arguments) => {
//                 let sig = windows::core::imp::ConstBuffer::new()
//                     .append_slice(b"pinterface(")
//                     .push_other( signature(generic) )
//                     .
//                     .append_guid(&match **generic {
//                         WType::Generic { piid, .. } => piid,
//                         _ => panic!("Expected generic type"),
//                     })
//                     .append_slice(b", ");
//                 sig.push_str(&generic.signature());
//                 for arg in type_arguments {
//                     sig.push_str(", ");
//                     sig.push_str(&arg.signature());
//                 }
//                 sig
//             }
//         }
//     }
// }

// incomplete type IList<>
// complete type IList<StorageFile> , i32



// let resultType = WinRTType::IAsyncOperation(WinRTType::Interface("...."))
// let resultType2 = WinRTType::IAsyncOperation(resultType) // IAsyncOperation<IAsyncOperation<PickFileResult>>

// IAsyncOperation<PickFileResult>
// IAsyncOperation<StorageFile>
// IAsyncOperation<_> - guid  pinterface(...., T::Guid)

impl WinRTType {
    pub fn abi_type(&self) -> AbiType {
        match self {
            WinRTType::I32 | WinRTType::HResult => AbiType::I32,
            WinRTType::I64 => AbiType::I64,
            WinRTType::Object
            | WinRTType::HString
            | WinRTType::OutValue(_)
            | WinRTType::IAsyncOperation(_)
            | WinRTType::ArrayOfIUnknown
            | WinRTType::Interface(_) => AbiType::Ptr,
        }
    }

    pub fn IUnknown() -> Self {
        WinRTType::Interface(windows::core::IUnknown::IID)
    }

    pub fn IInspectable() -> Self {
        WinRTType::Interface(windows::core::IInspectable::IID)
    }

    pub fn default_value(&self) -> WinRTValue {
        match self {
            WinRTType::I32 => WinRTValue::I32(0),
            WinRTType::I64 => WinRTValue::I64(0),
            WinRTType::Object => {
                WinRTValue::Object(unsafe { IUnknown::from_raw(std::ptr::null_mut()) })
            }
            WinRTType::HString => WinRTValue::HString(windows_core::HSTRING::new()),
            WinRTType::HResult => WinRTValue::HResult(windows_core::HRESULT(0)),
            WinRTType::OutValue(_) => WinRTValue::OutValue(std::ptr::null_mut(), self.clone()),
            WinRTType::IAsyncOperation(guid) => {
                panic!("Cannot create default value for IAsyncOperation {:?}", guid)
            }
            WinRTType::ArrayOfIUnknown => WinRTValue::ArrayOfIUnknown(
                crate::value::ArrayOfIUnknownData(windows::core::Array::new()),
            ),
            WinRTType::Interface(guid) => {
                panic!("Cannot create default value for Interface {:?}", guid)
            }
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
                WinRTType::IAsyncOperation(GUID) => Ok(WinRTValue::IAsyncOperation(
                    IUnknown::from_raw(ptr).cast()?,
                    *GUID,
                )),
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

enum TypeKind {
    Void,
    Bool,
    Char,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    ISize,
    USize,
    Pointer,
    String,
    Struct,
    Object,
    Array,
    ArrayRef,
}

// pub enum TypeHandle {
//     I32,
//     I64,
//     HResult,
//     HString,
//     Pointer,
//     IUnknown,
//     IInspectable,
//     Interface(usize),
//     ArrayOfInterface(usize),
//     RuntimeClass,
//     IAsyncOperation(usize),
// }

pub trait WinRTTypeContext {}
pub trait TypeHandle {
    type Context: WinRTTypeContext;
    fn kind(&self) -> TypeKind;
}

// AbiType =
// | i32
// | ...
// | pointer
// | struct(id)

// Method
// need to support look up method id by
// calling convention (ty1, ty2, ...) -> return type
// (calling convention, return_type: AbiType, vec(AbiType))

// AbiContext
// | struct(id) -> libffi's structure
// | method(id) -> libffi's Cif
