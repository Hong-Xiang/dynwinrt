#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use libffi::low::CodePtr;
    use windows::{
        AI::Actions::Hosting::ActionCatalog,
        Win32::System::WinRT::{
            IActivationFactory, RO_INIT_MULTITHREADED, RO_INIT_TYPE, RoGetActivationFactory,
            RoInitialize,
        },
    };
    use windows_collections::IVector;
    use windows_core::HRESULT;

    use crate::call::get_vtable_function_ptr;

    #[test]
    fn cryptographic_buffer_test() -> windows::core::Result<()> {
        use windows::Security::Cryptography::CryptographicBuffer;
        let value = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let buffer = CryptographicBuffer::CreateFromByteArray(&value)?;
        // let buffer = CryptographicBuffer::GenerateRandom(128)?;
        let base64 = CryptographicBuffer::EncodeToBase64String(&buffer)?;
        println!("Generated base64 string: {}", base64);
        Ok(())
    }

    #[tokio::test]
    async fn geolocation_value_type_test() -> windows::core::Result<()> {
        use windows::Devices::Geolocation::{BasicGeoposition, Geopoint, Geoposition};
        let position = BasicGeoposition {
            Latitude: 47.643,
            Longitude: -122.131,
            Altitude: 0.0,
        };
        let geopoint = Geopoint::Create(position)?;
        println!(
            "Geopoint created at lat: {}, lon: {}",
            geopoint.Position()?.Latitude,
            geopoint.Position()?.Longitude
        );
        // get current device location
        let locator = windows::Devices::Geolocation::Geolocator::new()?;
        let geoposition: Geoposition = locator.GetGeopositionAsync()?.await?;
        println!(
            "Current location: lat: {}, lon: {}",
            geoposition.Coordinate()?.Point()?.Position()?.Latitude,
            geoposition.Coordinate()?.Point()?.Position()?.Longitude
        );
        Ok(())
    }

    #[test]
    fn geolocation_value_type_dynamic() -> windows::core::Result<()> {
        use windows::Devices::Geolocation::{BasicGeoposition, Geopoint};
        use windows::core::h;
        use windows::core::{IInspectable, Interface};

        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED);
        }
        let position = BasicGeoposition {
            Latitude: 47.643,
            Longitude: -122.131,
            Altitude: 0.0,
        };
        let afactory = unsafe {
            RoGetActivationFactory::<IActivationFactory>(h!("Windows.Devices.Geolocation.Geopoint"))
        }?;
        let GeopointFactory = afactory.cast::<windows::Devices::Geolocation::IGeopointFactory>()?;
        let createFptr = get_vtable_function_ptr(GeopointFactory.as_raw(), 6);
        let create = unsafe {
            std::mem::transmute::<
                _,
                unsafe extern "system" fn(
                    *mut std::ffi::c_void,
                    BasicGeoposition,
                    // f64,
                    // f64,
                    // f64,
                    *mut *mut std::ffi::c_void,
                ) -> windows::core::HRESULT,
            >(createFptr)
        };
        let mut out = std::ptr::null_mut();
        let hr = unsafe { create(GeopointFactory.as_raw(), position, &mut out) };
        hr.ok()?;
        let geopoint = unsafe { Geopoint::from_raw(out) };
        let inspectable: IInspectable = geopoint.cast()?;
        let dynamic_geopoint: Geopoint = inspectable.cast()?;
        println!(
            "Dynamic Geopoint created at lat: {}, lon: {}",
            dynamic_geopoint.Position()?.Latitude,
            dynamic_geopoint.Position()?.Longitude
        );
        Ok(())
    }

    #[test]
    fn geolocation_value_type_dynamic_libffi() -> windows::core::Result<()> {
        use windows::Devices::Geolocation::{BasicGeoposition, Geopoint};
        use windows::core::h;
        use windows::core::{IInspectable, Interface};

        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED);
        }
        let position = BasicGeoposition {
            Latitude: 47.643,
            Longitude: -122.131,
            Altitude: 0.0,
        };
        let afactory = unsafe {
            RoGetActivationFactory::<IActivationFactory>(h!("Windows.Devices.Geolocation.Geopoint"))
        }?;
        let t : libffi::low::ffi_type = unsafe { std::mem::zeroed() };

        let BasicGeoPositionStruct = libffi::middle::Type::structure(vec![
            libffi::middle::Type::f64(), // Latitude
            libffi::middle::Type::f64(), // Longitude
            libffi::middle::Type::f64(), // Altitude
        ]);
        let f1 = Layout::new::<f64>();
        let (f2, offset2) = f1.extend(Layout::new::<f64>()).unwrap();
        let (f3, offset3) = f2.extend(Layout::new::<f64>()).unwrap();
        let sl = f3.pad_to_align();
        println!("Struct layout size: {}, align: {}", sl.size(), sl.align());
        println!("Field offsets: f1: 0, f2: {}, f3: {}", offset2, offset3);

        // let sptr = unsafe { (&position as *const _ as *mut std::ffi::c_void).add(0) };
        let sptr = unsafe { std::alloc::alloc(sl) };
        let pf1 = unsafe { sptr } as *mut f64;
        let pf2 = unsafe { sptr.add(offset2) } as *mut f64;
        let pf3 = unsafe { sptr.add(offset3) } as *mut f64;
        unsafe {
            *pf1 = 11.0;
            *pf2 = 22.0;
            *pf3 = 33.0;
        }
        println!(
            "After modifying fields {}, {}, {} ",
            position.Latitude, position.Longitude, position.Altitude
        );
        println!(
            "Struct values: f1: {}, f2: {}, f3: {}",
            unsafe { *pf1 },
            unsafe { *pf2 },
            unsafe { *pf3 }
        );

        println!(
            "position size : {} , align: {}, ptr: {:?}",
            std::mem::size_of::<BasicGeoposition>(),
            std::mem::align_of::<BasicGeoposition>(),
            &position as *const _
        );

        println!("dynamic struct ptr: {:?}", sptr);

        let create = libffi::middle::Cif::new(
            vec![
                libffi::middle::Type::pointer(), // this pointer
                BasicGeoPositionStruct,          // BasicGeoposition
                libffi::middle::Type::pointer(), // out parameter
            ]
            .into_iter(),
            libffi::middle::Type::i32(), // HRESULT
        );
        let GeopointFactory = afactory.cast::<windows::Devices::Geolocation::IGeopointFactory>()?;
        let createFptr = get_vtable_function_ptr(GeopointFactory.as_raw(), 6);
        let mut out = std::ptr::null_mut();
        let pOut = &mut out as *mut *mut std::ffi::c_void;
        let thisPtr = GeopointFactory.as_raw();
        let hr = unsafe {
            libffi::low::call::<HRESULT>(
                create.as_raw_ptr(),
                CodePtr(createFptr),
                vec![
                    &GeopointFactory.as_raw() as *const _ as *mut std::ffi::c_void,
                    // &position as *const _ as *mut std::ffi::c_void,
                    sptr as *const _ as *mut std::ffi::c_void,
                    &pOut as *const _ as *mut std::ffi::c_void,
                ]
                .as_mut_ptr(),
            )
        };
        hr.ok()?;
        let geopoint = unsafe { Geopoint::from_raw(out) };
        let inspectable: IInspectable = geopoint.cast()?;
        let dynamic_geopoint: Geopoint = inspectable.cast()?;
        println!(
            "Dynamic Geopoint created at lat: {}, lon: {}",
            dynamic_geopoint.Position()?.Latitude,
            dynamic_geopoint.Position()?.Longitude
        );
        Ok(())
    }

    #[tokio::test]
    async fn enumerate_device_test() -> windows::core::Result<()> {
        use windows::Devices::Enumeration::DeviceInformation;
        let devices = DeviceInformation::FindAllAsync()?.await?;
        let mut items = windows::core::Array::<DeviceInformation>::with_len(30);
        let count = devices.GetMany(10, &mut items)?;
        println!("Found {} devices", count);
        for device in items[..count as usize].iter() {
            println!(
                "Device: {} - {}",
                device.as_ref().unwrap().Name()?,
                device.as_ref().unwrap().Id()?
            );
        }
        Ok(())
    }
}
