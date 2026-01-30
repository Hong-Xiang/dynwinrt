use windows::Win32::System::WinRT::{IActivationFactory, RoGetActivationFactory};
use windows_core::HSTRING;

use crate::value::WinRTValue;

pub fn ro_get_activation_factory(class_name: &HSTRING) -> windows_core::Result<IActivationFactory> {
    unsafe { RoGetActivationFactory::<IActivationFactory>(class_name) }
}

#[cfg(test)]
mod tests {
    use windows::{
        Foundation::{IUriEscapeStatics, IUriRuntimeClassFactory, Uri},
        Win32::System::WinRT::{
            IActivationFactory, RO_INIT_MULTITHREADED, RoGetActivationFactory, RoInitialize,
        },
    };
    use windows_core::{GUID, IInspectable, Interface, h};

    use crate::{interfaces, value::WinRTValue};

    use super::*;

    #[test]
    fn call_get_activation_factory() -> windows::core::Result<()> {
        // Ignore error if already initialized
        let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
        let esu = Uri::EscapeComponent(h!("1 + 1"))?;
        println!("Escaped string: {}", esu);
        let uri = Uri::CreateUri(h!("https://www.example.com/path?query=1#fragment"))?;
        let factory =
            unsafe { RoGetActivationFactory::<IActivationFactory>(h!("Windows.Foundation.Uri")) }?;
        let uriFactory = factory.cast::<IUriRuntimeClassFactory>()?;
        let uriStatic: IUriEscapeStatics = factory.cast()?;

        let uriFactoryInterface = interfaces::uri_factory();
        let result = uriFactoryInterface.methods[6].call_dynamic(
            uriFactory.as_raw(),
            &[WinRTValue::HString(
                h!("https://www.example.com/anotherpath?query=2#fragment2").clone(),
            )],
        )?;
        let rv1 = &result[0];
        let uri: Uri = rv1.as_object().unwrap().cast()?;
        println!("Result from dynamic call: {:?}", uri);
        println!("Uri: {}", uri.Path()?);

        let inspect: IInspectable = factory.cast()?;
        let activateFactory: IActivationFactory = unsafe { inspect.cast() }?;
        println!("Got activation factory {:?} {:?}", inspect, activateFactory);
        Ok(())
    }
}
