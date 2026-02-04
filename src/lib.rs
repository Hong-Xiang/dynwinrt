use windows::Foundation::IStringable_Impl;
use windows::Storage::FileAccessMode;
use windows::{Data::Xml::Dom::*, core::*};
use windows_future::IAsyncOperation;

mod abi;
mod call;
mod interfaces;
mod result;
mod roapi;
mod signature;
mod types;
mod value;
mod winapp;

mod bindings;
mod dasync;

pub struct IIds;
impl IIds {
    pub const IFileOpenPickerFactory: windows_core::GUID = bindings::IFileOpenPickerFactory::IID;
    pub const IAsyncOperationPickFileResult: windows_core::GUID =
        IAsyncOperation::<bindings::PickFileResult>::IID;
}

pub fn export_add(x: f64, y: &f64) -> f64 {
    println!("export_add called with x = {}, y = {}", x, y);
    return x + y;
}

use crate::roapi::query_interface;
pub use crate::signature::{MethodSignature, InterfaceSignature};
pub use crate::types::WinRTType;
use crate::value::WinRTValue;
pub use crate::result::Result;
pub use crate::roapi::ro_get_activation_factory_2;
pub use crate::winapp::{WinAppSdkContext, initialize_winappsdk};
pub use interfaces::uri_vtable;
pub use crate::winapp::test_pick_open_picker_full_dynamic;

#[implement(windows::Foundation::IStringable)]
struct MyComObject {
    pub data: i32,
}

impl IStringable_Impl for MyComObject_Impl {
    fn ToString(&self) -> windows_core::Result<windows_core::HSTRING> {
        Ok(HSTRING::from(format!(
            "MyComObject ToString with data: {}",
            self.data
        )))
    }
}

impl Drop for MyComObject {
    fn drop(&mut self) {
        println!("MyComObject is being dropped");
    }
}

pub async fn get_async_string(op_string: IAsyncOperation<HSTRING>) -> windows_core::Result<String> {
    let s = op_string.await?;
    Ok(s.to_string())
}

pub async fn http_get_string(url: &str) -> windows_core::Result<String> {
    use windows::Foundation::Uri;
    use windows::Web::Http::HttpClient;
    let uri = Uri::CreateUri(&HSTRING::from(url))?;
    let http_client = HttpClient::new()?;
    let op = http_client.GetStringAsync(&uri)?;
    let s = op.await?;
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use windows::Foundation::{IStringable, Uri};

    use crate::value::WinRTValue;

    use super::*;

    #[test]
    fn from_raw_borrow_test() -> Result<()> {
        let uri = Uri::CreateUri(h!("https://www.example.com/path?query=1#fragment")).unwrap();
        let raw = uri.as_raw();
        let uri2 = unsafe { Uri::from_raw_borrowed(&raw) }.unwrap();
        let _ = uri2.clone();

        assert_eq!(uri.SchemeName().unwrap(), uri2.SchemeName().unwrap());
        Ok(())
    }

    #[test]
    fn http_call() -> Result<()> {
        futures::executor::block_on(async {
            use std::future::IntoFuture;

            let uri = Uri::CreateUri(h!("https://www.microsoft.com"))?;
            uri.SchemeName()?; // "http"
            uri.Host();
            uri.Port();
            let op = windows::Web::Http::HttpClient::new()?.GetStringAsync(&uri)?;
            let s = op.into_future().await?;
            // println!("Response: {}", s);

            Ok(())
        })
    }

    #[test]
    fn customCom() -> Result<()> {
        let my_obj = MyComObject { data: 42 };
        let sta: IStringable = my_obj.into();
        assert_eq!(
            sta.ToString()?.to_os_string(),
            "MyComObject ToString with data: 42"
        );
        Ok(())
    }

    #[test]
    fn simple_windows_api_should_work() -> Result<()> {
        let doc = XmlDocument::new()?;
        doc.LoadXml(h!("<html>hello world</html>"))?;

        let root = doc.DocumentElement()?;
        assert!(root.NodeName()? == "html");
        assert!(root.InnerText()? == "hello world");

        Ok(())
    }

    #[test]
    fn test_winrt_uri() -> Result<()> {
        use windows::Foundation::Uri;
        println!("URI guid = {:?} ...", Uri::IID);
        println!("URI name = {:?} ...", Uri::NAME);
        let uri = Uri::CreateUri(h!("https://www.example.com/path?query=1#fragment"))?;
        assert_eq!(uri.SchemeName()?, "https");
        assert_eq!(uri.Host()?, "www.example.com");
        assert_eq!(uri.Path()?, "/path");
        assert_eq!(uri.Query()?, "?query=1");
        assert_eq!(uri.Fragment()?, "#fragment");

        Ok(())
    }

    #[test]
    fn test_winrt_uri_interop_using_libffi() -> Result<()> {
        use libffi::middle::*;
        use std::ffi::c_void;
        use windows::Foundation::Uri;

        let uri = Uri::CreateUri(h!("http://www.example.com/path?query=1#fragment"))?;
        let mut hstr = HSTRING::new();
        let out_ptr: *mut c_void = std::ptr::from_mut(&mut hstr).cast();
        let obj = uri.as_raw();
        let fptr = call::get_vtable_function_ptr(obj.cast(), 17);
        let pars = vec![Type::pointer(), Type::pointer()];
        let cif = Cif::new(pars, Type::i32());
        let args = &[arg(&obj), arg(&out_ptr)];
        let hr: HRESULT = unsafe { cif.call(CodePtr(fptr), args) };
        hr.ok()?;
        assert_eq!(hstr, "http");

        Ok(())
    }

    #[test]
    fn test_winrt_uri_interop_using_signature() -> Result<()> {
        use windows::Foundation::Uri;

        let uri = Uri::CreateUri(h!("https://www.example.com/path?query=1#fragment"))?;

        let vtable = uri_vtable();

        let get_runtime_classname = &vtable.methods[4];
        assert_eq!(
            get_runtime_classname.call_dynamic(uri.as_raw(), &[])?[0]
                .as_hstring()
                .unwrap(),
            "Windows.Foundation.Uri"
        );

        let get_scheme = &vtable.methods[17];
        let scheme = get_scheme.call_dynamic(uri.as_raw(), &[])?;
        assert_eq!(scheme[0].as_hstring().unwrap(), "https");
        let get_path = &vtable.methods[13];
        let path = get_path.call_dynamic(uri.as_raw(), &[])?;
        assert_eq!(path[0].as_hstring().unwrap(), "/path");
        let get_port = &vtable.methods[19];
        let port = get_port.call_dynamic(uri.as_raw(), &[])?;
        assert_eq!(port[0].as_i32().unwrap(), 443);

        Ok(())
    }

    #[test]
    fn test_uri_call_dynamic() -> Result<()> {
        let uri = Uri::CreateUri(h!("https://www.example.com/path?query=1#fragment")).unwrap();
        let rptr = uri.as_raw();
        let ukn = unsafe { IUnknown::from_raw_borrowed(&rptr) }.unwrap();
        let obj = WinRTValue::Object(ukn.clone());
        let res = obj.call_single_out(17, &WinRTType::HString, &[]).unwrap();
        // let s : HSTRING = Default::default();
        // let mut p : *mut std::ffi::c_void = std::ptr::null_mut();
        // call::call_winrt_method_1(17, uri.as_raw(), &mut p);
        // let s : HSTRING = unsafe { core::mem::transmute(p) };
        assert_eq!(res.as_hstring().unwrap(), "https");
        Ok(())
    }

    #[test]
    fn test_calling_simple_add_extern_c() {
        extern "C" fn add(x: f64, y: &f64) -> f64 {
            return x + y;
        }

        use libffi::middle::*;

        let args = vec![Type::f64(), Type::pointer()];
        let cif = Cif::new(args.into_iter(), Type::f64());

        let n: f64 = unsafe { cif.call(CodePtr(add as *mut _), &[arg(&5f64), arg(&&6f64)]) };
        assert_eq!(11f64, n);
    }

    #[test]
    fn test_winmd_read() {
        use windows_metadata::*;

        let index = reader::Index::read(
            r"C:\Program Files (x86)\Windows Kits\10\UnionMetadata\10.0.26100.0\Windows.winmd",
        )
        .unwrap();

        let def = index.expect("Windows.Foundation", "Point");
        assert_eq!(def.namespace(), "Windows.Foundation");
        assert_eq!(def.name(), "Point");

        let extends = def.extends().unwrap();
        assert_eq!(extends.namespace(), "System");
        assert_eq!(extends.name(), "ValueType");

        let fields: Vec<_> = def.fields().collect();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "X");
        assert_eq!(fields[1].name(), "Y");
        assert_eq!(fields[0].ty(), Type::F32);
        assert_eq!(fields[1].ty(), Type::F32);
        println!("{:?}", fields);
    }

    #[test]
    fn test_winmd_read_uri() {
        use windows_metadata::*;
        let index = reader::Index::read(
            r"C:\Program Files (x86)\Windows Kits\10\UnionMetadata\10.0.26100.0\Windows.winmd",
        )
        .unwrap();
        let def = index.expect("Windows.Foundation", "Uri");
        // list all methods and print their signatures
        for method in def.methods() {
            let params: Vec<String> = method
                .params()
                .enumerate()
                .map(|(i, p)| format!("{}: {:?}", p.name(), method.signature(&[]).types))
                .collect();
            println!(
                "fn {:?} {}({}) -> {:?}",
                method.flags(),
                method.name(),
                params.join(", "),
                method.signature(&[]).return_type
            );
        }
    }

    #[test]
    fn test_winmd_read_http_client() {
        use windows_metadata::*;
        let index = reader::Index::read(
            r"C:\Program Files (x86)\Windows Kits\10\UnionMetadata\10.0.26100.0\Windows.winmd",
        )
        .unwrap();
        let def = index.expect("Windows.Web.Http", "HttpClient");
        // list all methods and print their signatures
        for (i, method) in def.methods().enumerate() {
            let params: Vec<String> = method
                .params()
                .enumerate()
                .map(|(i, p)| format!("{}: {:?}", p.name(), ""))
                .collect();
            println!(
                "#{}, fn {:?} {}({}) -> {:?}",
                i,
                // method.flags(),
                "",
                method.name(),
                params.join(", "),
                method.signature(&[]).return_type
            );
        }
    }
}

mod from_api_test {
    use windows::{Win32::System::Com::IUri, core::*};
    link!("urlmon.dll" "system" fn CreateUri(
    uri: PCWSTR, 
    flags: u32, 
    reserved: usize, 
    result: *mut *mut core::ffi::c_void) -> HRESULT);
    fn main() -> Result<()> {
        unsafe {
            let mut uri = core::mem::zeroed();
            let uri: IUri = CreateUri(w!("http://kennykerr.ca"), 0, 0, &mut uri)
                .and_then(|| Type::from_abi(uri))?;
            let domain = uri.GetDomain()?;
            let port = uri.GetPort()?;
            println!("{:?} ({port})", domain);
        }
        Ok(())
    }
}

pub fn windows_ai_ocr_api_call() {
    println!("windows_ai_ocr_api_call called");
    use winapp::*;
       use bindings::*;
    let options = WinAppSdkBootstrapOptions {
            major_version: 1,
            minor_version: 8,
            build_version: 0,
            revision_version: 0,
            bootstrap_dll_path: None,
        };
    // let result = initialize(options);
    // assert!(result.is_ok(), "message {}", result.err().unwrap().message());
    let factory = roapi::ro_get_activation_factory(h!("Microsoft.Windows.AI.Imaging.TextRecognizer"));
    assert!(factory.is_ok(), "message {}", factory.err().unwrap().message());
    println!("WinAppSdk initialized");
    let ready_state = TextRecognizer::GetReadyState().unwrap();
    println!("TextRecognizer ready state: {:?}", ready_state);
}

pub async fn windows_ai_ocr_api_call_projected(path: &str) -> windows::core::Result<()> {
    use winapp::*;
    use bindings::*;
    println!("windows_ai_ocr_api_call_projected called");
    
    let options = WinAppSdkBootstrapOptions {
        major_version: 1,
        minor_version: 8,
        build_version: 0,
        revision_version: 0,
        bootstrap_dll_path: None,
    };
    // initialize(options)?;
    println!("WinAppSdk initialized");
    let ready_state = TextRecognizer::GetReadyState().unwrap();
    println!("TextRecognizer ready state: {:?}", ready_state);

    if ready_state == AIFeatureReadyState::NotReady {
        let operation = TextRecognizer::EnsureReadyAsync()?.await?;
        if operation.Status()? != AIFeatureReadyResultState::Success {
            return Err(Error::from_hresult(HRESULT(1).into() /* E_FAIL */));
        }
    }

    let recognizer = TextRecognizer::CreateAsync()?.await?;
    println!("TextRecognizer created successfully");
    
    // Load SoftwareBitmap from file path (similar to C# BitmapDecoder pattern)
    // use windows::Storage::{StorageFile, FileAccessMode};
    use windows::Storage::StorageFile;

    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path))?.await?;
    println!("File loaded successfully from path: {}", path);
    let stream = file.OpenAsync(FileAccessMode::Read )?.await?;
    println!("File stream opened successfully");
    let decoder = windows::Graphics::Imaging::BitmapDecoder::CreateAsync(&stream)?.await?;
    println!("BitmapDecoder created successfully");
    let bitmap = decoder.GetSoftwareBitmapAsync()?.await?;
    println!("SoftwareBitmap obtained successfully");
    let raw = bitmap.as_raw();
    let bitmapb = unsafe { bindings::SoftwareBitmap::from_raw_borrowed(&raw)};
    println!("SoftwareBitmap wrapped successfully");
    let image_buffer = ImageBuffer::CreateForSoftwareBitmap(bitmapb).unwrap();
    println!("ImageBuffer created from file successfully");
    let result = recognizer.RecognizeTextFromImageAsync(&image_buffer)?.await?;
    println!("Text recognition completed successfully");
    // TODO: Call recognizer.RecognizeTextFromImage(image_buffer) when bitmap is available
    let lines = result.Lines()?;
    for i in 0 .. lines.len() {
        let line = lines[i].as_ref().unwrap();
        println!("Recognized line: {}", line.Text()?);
    }
    for line in result.Lines()?.into_iter() {
        println!("Recognized line: {}", line.as_ref().unwrap().Text()?);
    }
    Ok(())
}

pub async fn windows_ai_ocr_api_call_dynamic(path: &str) -> windows::core::Result<()> {
    use winapp::*;
    use bindings::*;
    println!("windows_ai_ocr_api_call_projected called");
    
    let options = WinAppSdkBootstrapOptions {
        major_version: 1,
        minor_version: 8,
        build_version: 0,
        revision_version: 0,
        bootstrap_dll_path: None,
    };
    // initialize(options)?;
    println!("WinAppSdk initialized");
    let factory = roapi::ro_get_activation_factory(h!("Microsoft.Windows.AI.Imaging.TextRecognizer"))?;
    let factory_ukn = WinRTValue::Object(factory.cast::<windows_core::IUnknown>()?);
    
    let text_recongizer_factory = query_interface(  factory_ukn, &ITextRecognizerStatics::IID)?;
    let get_ready_state = MethodSignature::new().add_out(WinRTType::I32).build(6);
    let ready_state = get_ready_state.call_dynamic(text_recongizer_factory.as_object().unwrap().as_raw(), &[]).unwrap()[0]
        .as_i32()
        .unwrap();
    // let ready_state = TextRecognizer::GetReadyState().unwrap();
    println!("TextRecognizer ready state: {:?}", ready_state);

    if ready_state == AIFeatureReadyState::NotReady.0  {
        println!("TextRecognizer is not ready, calling EnsureReadyAsync...");
        let operation = TextRecognizer::EnsureReadyAsync()?.await?;
        if operation.Status()? != AIFeatureReadyResultState::Success {
            return Err(Error::from_hresult(HRESULT(1).into() /* E_FAIL */));
        }
    }

    let recognizer = TextRecognizer::CreateAsync()?.await?;
    println!("TextRecognizer created successfully");
    
    // Load SoftwareBitmap from file path (similar to C# BitmapDecoder pattern)
    // use windows::Storage::{StorageFile, FileAccessMode};
    use windows::Storage::StorageFile;

    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path))?.await?;
    println!("File loaded successfully from path: {}", path);
    let stream = file.OpenAsync(FileAccessMode::Read )?.await?;
    println!("File stream opened successfully");
    let decoder = windows::Graphics::Imaging::BitmapDecoder::CreateAsync(&stream)?.await?;
    println!("BitmapDecoder created successfully");
    let bitmap = decoder.GetSoftwareBitmapAsync()?.await?;
    println!("SoftwareBitmap obtained successfully");
    let raw = bitmap.as_raw();
    let bitmapb = unsafe { bindings::SoftwareBitmap::from_raw_borrowed(&raw)};
    println!("SoftwareBitmap wrapped successfully");
    let image_buffer = ImageBuffer::CreateForSoftwareBitmap(bitmapb).unwrap();
    println!("ImageBuffer created from file successfully");
    let result = recognizer.RecognizeTextFromImageAsync(&image_buffer)?.await?;
    println!("Text recognition completed successfully");
    // TODO: Call recognizer.RecognizeTextFromImage(image_buffer) when bitmap is available
    let lines = result.Lines()?;
    for i in 0 .. lines.len() {
        let line = lines[i].as_ref().unwrap();
        println!("Recognized line: {}", line.Text()?);
    }
    for line in result.Lines()?.into_iter() {
        println!("Recognized line: {}", line.as_ref().unwrap().Text()?);
    }
    Ok(())
}