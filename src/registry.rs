use std::alloc::Layout;
use std::sync::{Arc, RwLock};

/// Primitive types that can appear as fields in WinRT value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

impl PrimitiveType {
    pub fn size_of(self) -> usize {
        match self {
            PrimitiveType::Bool | PrimitiveType::U8 => 1,
            PrimitiveType::I16 | PrimitiveType::U16 => 2,
            PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 => 4,
            PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => 8,
        }
    }

    pub fn align_of(self) -> usize {
        self.size_of()
    }

    fn libffi_type(self) -> libffi::middle::Type {
        match self {
            PrimitiveType::Bool => libffi::middle::Type::u8(),
            PrimitiveType::U8 => libffi::middle::Type::u8(),
            PrimitiveType::I16 => libffi::middle::Type::i16(),
            PrimitiveType::U16 => libffi::middle::Type::u16(),
            PrimitiveType::I32 => libffi::middle::Type::i32(),
            PrimitiveType::U32 => libffi::middle::Type::u32(),
            PrimitiveType::I64 => libffi::middle::Type::i64(),
            PrimitiveType::U64 => libffi::middle::Type::u64(),
            PrimitiveType::F32 => libffi::middle::Type::f32(),
            PrimitiveType::F64 => libffi::middle::Type::f64(),
        }
    }
}

/// Internal type identifier. Not exposed publicly — users only see `TypeHandle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TypeKind {
    Primitive(PrimitiveType),
    Struct(u32),
}

/// Internal struct data stored in the registry.
struct StructEntry {
    field_kinds: Vec<TypeKind>,
    field_offsets: Vec<usize>,
    layout: Layout,
}

/// Registry of value types. Always lives behind `Arc`, supports concurrent reads
/// and append-only mutation via `RwLock`.
pub struct TypeRegistry {
    structs: RwLock<Vec<StructEntry>>,
}

impl TypeRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(TypeRegistry {
            structs: RwLock::new(Vec::new()),
        })
    }

    pub fn primitive(self: &Arc<Self>, ty: PrimitiveType) -> TypeHandle {
        TypeHandle {
            registry: Arc::clone(self),
            kind: TypeKind::Primitive(ty),
        }
    }

    pub fn define_struct(self: &Arc<Self>, fields: &[TypeHandle]) -> TypeHandle {
        let field_kinds: Vec<TypeKind> = fields.iter().map(|h| h.kind).collect();
        let (field_offsets, layout) = self.compute_layout(&field_kinds);
        let mut structs = self.structs.write().unwrap();
        let id = structs.len() as u32;
        structs.push(StructEntry {
            field_kinds,
            field_offsets,
            layout,
        });
        TypeHandle {
            registry: Arc::clone(self),
            kind: TypeKind::Struct(id),
        }
    }

    // --- Internal query methods (take TypeKind, no Arc needed) ---

    fn size_of_kind(&self, kind: TypeKind) -> usize {
        match kind {
            TypeKind::Primitive(p) => p.size_of(),
            TypeKind::Struct(id) => self.structs.read().unwrap()[id as usize].layout.size(),
        }
    }

    fn align_of_kind(&self, kind: TypeKind) -> usize {
        match kind {
            TypeKind::Primitive(p) => p.align_of(),
            TypeKind::Struct(id) => self.structs.read().unwrap()[id as usize].layout.align(),
        }
    }

    fn layout_of_kind(&self, kind: TypeKind) -> Layout {
        match kind {
            TypeKind::Primitive(p) => Layout::from_size_align(p.size_of(), p.align_of()).unwrap(),
            TypeKind::Struct(id) => self.structs.read().unwrap()[id as usize].layout,
        }
    }

    fn field_count_kind(&self, kind: TypeKind) -> usize {
        match kind {
            TypeKind::Primitive(_) => panic!("Primitive types have no fields"),
            TypeKind::Struct(id) => self.structs.read().unwrap()[id as usize].field_kinds.len(),
        }
    }

    fn field_offset_kind(&self, kind: TypeKind, index: usize) -> usize {
        match kind {
            TypeKind::Primitive(_) => panic!("Primitive types have no fields"),
            TypeKind::Struct(id) => self.structs.read().unwrap()[id as usize].field_offsets[index],
        }
    }

    fn field_kind(&self, kind: TypeKind, index: usize) -> TypeKind {
        match kind {
            TypeKind::Primitive(_) => panic!("Primitive types have no fields"),
            TypeKind::Struct(id) => self.structs.read().unwrap()[id as usize].field_kinds[index],
        }
    }

    fn libffi_type_kind(&self, kind: TypeKind) -> libffi::middle::Type {
        match kind {
            TypeKind::Primitive(p) => p.libffi_type(),
            TypeKind::Struct(id) => {
                let structs = self.structs.read().unwrap();
                let field_types: Vec<libffi::middle::Type> = structs[id as usize]
                    .field_kinds
                    .iter()
                    .map(|f| self.libffi_type_kind(*f))
                    .collect();
                libffi::middle::Type::structure(field_types)
            }
        }
    }

    fn compute_layout(&self, fields: &[TypeKind]) -> (Vec<usize>, Layout) {
        let mut offsets = Vec::with_capacity(fields.len());
        let mut offset = 0usize;
        let mut max_align = 1usize;

        for field in fields {
            let field_align = self.align_of_kind(*field);
            let field_size = self.size_of_kind(*field);
            max_align = max_align.max(field_align);
            offset = (offset + field_align - 1) & !(field_align - 1);
            offsets.push(offset);
            offset += field_size;
        }

        let size = (offset + max_align - 1) & !(max_align - 1);
        (offsets, Layout::from_size_align(size, max_align).unwrap())
    }
}

/// A handle to a type in the registry. Carries an `Arc<TypeRegistry>` so it
/// can query layout and create values without needing a separate registry reference.
#[derive(Clone)]
pub struct TypeHandle {
    registry: Arc<TypeRegistry>,
    kind: TypeKind,
}

impl TypeHandle {
    pub fn size_of(&self) -> usize {
        self.registry.size_of_kind(self.kind)
    }

    pub fn align_of(&self) -> usize {
        self.registry.align_of_kind(self.kind)
    }

    pub fn layout(&self) -> Layout {
        self.registry.layout_of_kind(self.kind)
    }

    pub fn libffi_type(&self) -> libffi::middle::Type {
        self.registry.libffi_type_kind(self.kind)
    }

    pub fn field_count(&self) -> usize {
        self.registry.field_count_kind(self.kind)
    }

    pub fn field_offset(&self, index: usize) -> usize {
        self.registry.field_offset_kind(self.kind, index)
    }

    pub fn field_type(&self, index: usize) -> TypeHandle {
        TypeHandle {
            registry: Arc::clone(&self.registry),
            kind: self.registry.field_kind(self.kind, index),
        }
    }

    pub fn default_value(&self) -> ValueTypeData {
        ValueTypeData::new(self)
    }
}

/// A dynamically-typed value matching a struct layout from the registry.
///
/// Owns an aligned heap allocation. Holds a `TypeHandle` internally so
/// field access methods are self-contained.
pub struct ValueTypeData {
    type_handle: TypeHandle,
    ptr: *mut u8,
}

impl ValueTypeData {
    fn new(handle: &TypeHandle) -> Self {
        let layout = handle.layout();
        let ptr = if layout.size() > 0 {
            unsafe { std::alloc::alloc_zeroed(layout) }
        } else {
            std::ptr::null_mut()
        };
        Self {
            type_handle: handle.clone(),
            ptr,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn get_field<T: Copy>(&self, index: usize) -> T {
        let h = &self.type_handle;
        let offset = h.field_offset(index);
        assert_eq!(
            std::mem::size_of::<T>(),
            h.field_type(index).size_of(),
            "get_field<T> size mismatch"
        );
        unsafe { (self.ptr.add(offset) as *const T).read() }
    }

    pub fn set_field<T: Copy>(&mut self, index: usize, value: T) {
        let h = &self.type_handle;
        let offset = h.field_offset(index);
        assert_eq!(
            std::mem::size_of::<T>(),
            h.field_type(index).size_of(),
            "set_field<T> size mismatch"
        );
        unsafe { (self.ptr.add(offset) as *mut T).write(value) }
    }

    /// Call a COM method that takes this struct by value and returns an Object.
    /// ABI pattern: HRESULT Method(this_ptr, struct_by_value, *out_ptr)
    pub fn call_method_struct_to_object(
        &self,
        obj_raw: *mut std::ffi::c_void,
        method_index: usize,
    ) -> windows_core::Result<windows_core::IUnknown> {
        use crate::call::get_vtable_function_ptr;
        use libffi::middle::{arg, Cif, CodePtr, Type};
        use windows_core::Interface;

        let fptr = get_vtable_function_ptr(obj_raw, method_index);
        let cif = Cif::new(
            vec![
                Type::pointer(),
                self.type_handle.libffi_type(),
                Type::pointer(),
            ],
            Type::i32(),
        );

        let mut out: *mut std::ffi::c_void = std::ptr::null_mut();
        let data_ref = unsafe { &*self.ptr };
        let hr: windows_core::HRESULT = unsafe {
            cif.call(
                CodePtr(fptr),
                &[arg(&obj_raw), arg(data_ref), arg(&(&mut out))],
            )
        };
        hr.ok()?;
        Ok(unsafe { windows_core::IUnknown::from_raw(out as _) })
    }
}

impl Drop for ValueTypeData {
    fn drop(&mut self) {
        let layout = self.type_handle.layout();
        if layout.size() > 0 {
            unsafe { std::alloc::dealloc(self.ptr, layout) }
        }
    }
}

impl Clone for ValueTypeData {
    fn clone(&self) -> Self {
        let layout = self.type_handle.layout();
        if layout.size() == 0 {
            return Self {
                type_handle: self.type_handle.clone(),
                ptr: std::ptr::null_mut(),
            };
        }
        let ptr = unsafe {
            let p = std::alloc::alloc(layout);
            std::ptr::copy_nonoverlapping(self.ptr, p, layout.size());
            p
        };
        Self {
            type_handle: self.type_handle.clone(),
            ptr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_sizes() {
        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let f64_h = reg.primitive(PrimitiveType::F64);
        let i32_h = reg.primitive(PrimitiveType::I32);
        let u8_h = reg.primitive(PrimitiveType::U8);

        assert_eq!(f32_h.size_of(), 4);
        assert_eq!(f64_h.size_of(), 8);
        assert_eq!(i32_h.size_of(), 4);
        assert_eq!(u8_h.size_of(), 1);

        assert_eq!(f32_h.align_of(), 4);
        assert_eq!(f64_h.align_of(), 8);
    }

    #[test]
    fn point_layout() {
        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let point = reg.define_struct(&[f32_h.clone(), f32_h]);

        assert_eq!(point.size_of(), 8);
        assert_eq!(point.align_of(), 4);
        assert_eq!(point.field_count(), 2);
        assert_eq!(point.field_offset(0), 0);
        assert_eq!(point.field_offset(1), 4);
    }

    #[test]
    fn rect_layout() {
        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let rect = reg.define_struct(&[f32_h.clone(), f32_h.clone(), f32_h.clone(), f32_h]);

        assert_eq!(rect.size_of(), 16);
        assert_eq!(rect.align_of(), 4);
        assert_eq!(rect.field_offset(0), 0);
        assert_eq!(rect.field_offset(1), 4);
        assert_eq!(rect.field_offset(2), 8);
        assert_eq!(rect.field_offset(3), 12);
    }

    #[test]
    fn basic_geoposition_layout() {
        let reg = TypeRegistry::new();
        let f64_h = reg.primitive(PrimitiveType::F64);
        let geo = reg.define_struct(&[f64_h.clone(), f64_h.clone(), f64_h]);

        assert_eq!(geo.size_of(), 24);
        assert_eq!(geo.align_of(), 8);
        assert_eq!(geo.field_offset(0), 0);
        assert_eq!(geo.field_offset(1), 8);
        assert_eq!(geo.field_offset(2), 16);
    }

    #[test]
    fn mixed_field_alignment() {
        let reg = TypeRegistry::new();
        let u8_h = reg.primitive(PrimitiveType::U8);
        let i32_h = reg.primitive(PrimitiveType::I32);
        let s = reg.define_struct(&[u8_h.clone(), i32_h, u8_h]);

        assert_eq!(s.size_of(), 12);
        assert_eq!(s.align_of(), 4);
        assert_eq!(s.field_offset(0), 0);
        assert_eq!(s.field_offset(1), 4);
        assert_eq!(s.field_offset(2), 8);
    }

    #[test]
    fn nested_struct_layout() {
        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let f64_h = reg.primitive(PrimitiveType::F64);
        let inner = reg.define_struct(&[f32_h.clone(), f32_h]);
        let outer = reg.define_struct(&[inner, f64_h]);

        assert_eq!(outer.size_of(), 16);
        assert_eq!(outer.align_of(), 8);
        assert_eq!(outer.field_offset(0), 0);
        assert_eq!(outer.field_offset(1), 8);
    }

    #[test]
    fn value_type_data_field_access() {
        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let point = reg.define_struct(&[f32_h.clone(), f32_h]);

        let mut val = point.default_value();
        val.set_field(0, 10.0f32);
        val.set_field(1, 20.0f32);

        assert_eq!(val.get_field::<f32>(0), 10.0);
        assert_eq!(val.get_field::<f32>(1), 20.0);
    }

    #[test]
    fn value_type_data_clone() {
        let reg = TypeRegistry::new();
        let f64_h = reg.primitive(PrimitiveType::F64);
        let geo = reg.define_struct(&[f64_h.clone(), f64_h.clone(), f64_h]);

        let mut val = geo.default_value();
        val.set_field(0, 47.6f64);
        val.set_field(1, -122.3f64);
        val.set_field(2, 100.0f64);

        let cloned = val.clone();
        assert_eq!(cloned.get_field::<f64>(0), 47.6);
        assert_eq!(cloned.get_field::<f64>(1), -122.3);
        assert_eq!(cloned.get_field::<f64>(2), 100.0);

        val.set_field(0, 0.0f64);
        assert_eq!(cloned.get_field::<f64>(0), 47.6);
    }

    #[test]
    fn value_type_matches_windows_point_layout() {
        use windows::Foundation::Point;

        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let point = reg.define_struct(&[f32_h.clone(), f32_h]);

        assert_eq!(point.size_of(), std::mem::size_of::<Point>());
        assert_eq!(point.align_of(), std::mem::align_of::<Point>());

        let mut val = point.default_value();
        val.set_field(0, 10.0f32);
        val.set_field(1, 20.0f32);

        let win_point: &Point = unsafe { &*(val.as_ptr() as *const Point) };
        assert_eq!(win_point.X, 10.0);
        assert_eq!(win_point.Y, 20.0);
    }

    #[test]
    fn libffi_type_primitive() {
        let reg = TypeRegistry::new();
        let _ = reg.primitive(PrimitiveType::F32).libffi_type();
        let _ = reg.primitive(PrimitiveType::I64).libffi_type();
    }

    #[test]
    fn libffi_type_struct() {
        let reg = TypeRegistry::new();
        let f32_h = reg.primitive(PrimitiveType::F32);
        let f64_h = reg.primitive(PrimitiveType::F64);
        let point = reg.define_struct(&[f32_h.clone(), f32_h]);
        let _ = point.libffi_type();

        let outer = reg.define_struct(&[point, f64_h]);
        let _ = outer.libffi_type();
    }

    #[test]
    fn geopoint_create_via_registry() -> windows::core::Result<()> {
        use libffi::middle::{Cif, CodePtr, arg};
        use windows::Devices::Geolocation::Geopoint;
        use windows::Win32::System::WinRT::{
            IActivationFactory, RO_INIT_MULTITHREADED, RoGetActivationFactory, RoInitialize,
        };
        use windows::core::{Interface, h};
        use windows_core::HRESULT;

        use crate::call::get_vtable_function_ptr;

        let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };

        // 1. Define BasicGeoposition { Latitude: f64, Longitude: f64, Altitude: f64 }
        let reg = TypeRegistry::new();
        let f64_h = reg.primitive(PrimitiveType::F64);
        let geo_type = reg.define_struct(&[f64_h.clone(), f64_h.clone(), f64_h]);

        // 2. Create & fill value
        let mut geo_val = geo_type.default_value();
        geo_val.set_field(0, 47.643f64);
        geo_val.set_field(1, -122.131f64);
        geo_val.set_field(2, 100.0f64);

        // 3. Get IGeopointFactory (vtable index 6 = Create method)
        let afactory = unsafe {
            RoGetActivationFactory::<IActivationFactory>(
                h!("Windows.Devices.Geolocation.Geopoint"),
            )
        }?;
        let geopoint_factory =
            afactory.cast::<windows::Devices::Geolocation::IGeopointFactory>()?;
        let fptr = get_vtable_function_ptr(geopoint_factory.as_raw(), 6);

        // 4. Build Cif: fn(this: ptr, position: BasicGeoposition, out: ptr) -> HRESULT
        let cif = Cif::new(
            vec![
                libffi::middle::Type::pointer(),
                geo_type.libffi_type(),
                libffi::middle::Type::pointer(),
            ],
            libffi::middle::Type::i32(),
        );

        // 5. Call via libffi
        let mut out: *mut std::ffi::c_void = std::ptr::null_mut();
        let this = geopoint_factory.as_raw();
        let geo_data_ref = unsafe { &*geo_val.as_ptr() };
        let hr: HRESULT = unsafe {
            cif.call(
                CodePtr(fptr),
                &[arg(&this), arg(geo_data_ref), arg(&(&mut out))],
            )
        };
        hr.ok()?;

        // 6. Verify
        let geopoint = unsafe { Geopoint::from_raw(out) };
        let pos = geopoint.Position()?;
        assert!((pos.Latitude - 47.643).abs() < 1e-6);
        assert!((pos.Longitude - (-122.131)).abs() < 1e-6);
        assert!((pos.Altitude - 100.0).abs() < 1e-6);

        Ok(())
    }
}
