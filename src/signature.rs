use libffi::middle::{Arg, Cif, arg};

use crate::{call, types::WinRTType, value::WinRTValue};

pub struct Parameter {
    pub typ: WinRTType,
    pub value_index: usize,
    pub is_out: bool,
}

pub struct MethodSignature {
    index: usize,
    out_count: usize,
    parameters: Vec<Parameter>,
    cif: Option<Cif>,
}

impl MethodSignature {
    pub fn new(index: usize) -> Self {
        MethodSignature {
            index,
            out_count: 0,
            parameters: Vec::new(),
            cif: None,
        }
    }

    pub fn add(&mut self, typ: WinRTType) -> &mut Self {
        self.parameters.push(Parameter {
            is_out: false,
            typ,
            value_index: self.parameters.len() - self.out_count,
        });
        self
    }

    pub fn add_out(&mut self, typ: WinRTType) -> &mut Self {
        self.parameters.push(Parameter {
            is_out: true,
            typ,
            value_index: self.out_count,
        });
        self.out_count += 1;
        self
    }

    pub fn build_cif(&mut self) {
        use libffi::middle::Type;
        let mut types: Vec<Type> = Vec::with_capacity(self.parameters.len() + 1);
        types.push(Type::pointer()); // com object's this pointer
        for param in &self.parameters {
            types.push(if param.is_out {
                // out parameters are always pointers
                Type::pointer()
            } else {
                param.typ.libffi_type()
            })
        }
        self.cif = Some(Cif::new(types.into_iter(), Type::i32()));
    }

    pub fn call_dynamic(
        &self,
        obj: *mut std::ffi::c_void,
        args: &[WinRTValue],
    ) -> windows_core::Result<Vec<WinRTValue>> {
        call::call_winrt_method_dynamic(
            self.index,
            obj,
            &self.parameters,
            args,
            self.out_count,
            self.cif.as_ref().unwrap(),
        )
    }
}

pub struct MethodInfo {
    pub index: usize,
    pub parameters: Vec<Parameter>,
}
pub enum Method {
    CifMethod { info: MethodInfo, cif: Cif },
    PtrPtrMethod { info: MethodInfo },
}

pub struct VTableSignature {
    pub methods: Vec<MethodSignature>,
}

impl VTableSignature {
    pub fn new() -> Self {
        VTableSignature {
            methods: Vec::new(),
        }
    }

    pub fn add_method(
        &mut self,
        builder: fn(sig: &mut MethodSignature) -> &MethodSignature,
    ) -> &mut Self {
        let mut method = MethodSignature::new(self.methods.len());
        builder(&mut method);
        method.build_cif();
        self.methods.push(method);
        self
    }
}
