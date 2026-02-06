use libffi::middle::Cif;
use windows::core::{GUID, HSTRING};

use crate::{call, types::WinRTType, value::WinRTValue};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub typ: WinRTType,
    pub value_index: usize,
    pub is_out: bool,
}

#[derive(Debug, Clone)]
pub struct MethodSignature {
    out_count: usize,
    parameters: Vec<Parameter>,
    return_type: WinRTType,
    is_opaque: bool,
}

impl MethodSignature {
    pub fn new() -> Self {
        MethodSignature {
            out_count: 0,
            parameters: Vec::new(),
            return_type: WinRTType::HResult,
            is_opaque: false,
        }
    }

    pub fn add(mut self, typ: WinRTType) -> Self {
        self.parameters.push(Parameter {
            is_out: false,
            typ,
            value_index: self.parameters.len() - self.out_count,
        });
        self
    }

    pub fn add_out(mut self, typ: WinRTType) -> Self {
        self.parameters.push(Parameter {
            is_out: true,
            typ,
            value_index: self.out_count,
        });
        self.out_count += 1;
        self
    }

    pub fn build(self, index: usize) -> Method {
        use libffi::middle::Type;
        let mut types: Vec<Type> = Vec::with_capacity(self.parameters.len() + 1);
        types.push(Type::pointer()); // com object's this pointer
        for param in &self.parameters {
            types.push(if param.is_out {
                // out parameters are always pointers
                Type::pointer()
            } else {
                param.typ.abi_type().libffi_type()
            })
        }
        let cif = Cif::new(types.into_iter(), self.return_type.abi_type().libffi_type());
        Method {
            info: MethodInfo {
                index,
                parameters: self.parameters,
                out_count: self.out_count,
            },
            cif,
        }
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    pub index: usize,
    pub parameters: Vec<Parameter>,
    pub out_count: usize,
}

#[derive(Debug)]
pub struct Method {
    info: MethodInfo,
    cif: Cif,
}

impl Method {
    pub fn call_dynamic(
        &self,
        obj: *mut std::ffi::c_void,
        args: &[WinRTValue],
    ) -> windows_core::Result<Vec<WinRTValue>> {
        call::call_winrt_method_dynamic(
            self.info.index,
            obj,
            &self.info.parameters,
            args,
            self.info.out_count,
            &self.cif,
        )
    }
}

#[derive(Debug)]
pub struct InterfaceSignature {
    pub name: String,
    pub iid: windows_core::GUID,
    pub methods: Vec<Method>,
}

impl InterfaceSignature {
    pub fn define_interface(name: String, iid: windows_core::GUID) -> Self {
        InterfaceSignature {
            name,
            iid,
            methods: Vec::new(),
        }
    }

    pub fn define_from_iunknown(name: &str, iid: GUID) -> Self {
        let mut t = InterfaceSignature::define_interface(name.to_owned(), iid);
        t.add_method(MethodSignature::new()) // 0 QueryInterface
            .add_method(MethodSignature::new()) // 1 AddRef
            .add_method(MethodSignature::new()); // 2 Release
        t
    }

    pub fn define_from_iinspectable(name: &str, iid: GUID) -> Self {
        let mut t = Self::define_from_iunknown(name, iid);
        t.add_method(MethodSignature::new()) // 3 GetIids
            .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
            .add_method(MethodSignature::new()); // 5 GetTrustLevel
        t
    }

    pub fn add_method(&mut self, signature: MethodSignature) -> &mut Self {
        let method = signature.build(self.methods.len());
        self.methods.push(method);
        self
    }
}

pub struct RuntimeClassSignature {
    name: HSTRING,
    static_interfaces: Vec<InterfaceSignature>,
    instance_interfaces: Vec<InterfaceSignature>,
}
