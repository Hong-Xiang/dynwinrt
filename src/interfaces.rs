use crate::signature::{InterfaceSignature, MethodSignature};
use crate::types::WinRTType;

pub fn uri_factory() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new("".to_string(), Default::default());
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(
            MethodSignature::new()
                .add(WinRTType::HString)
                .add_out(WinRTType::Object),
        );
    vtable
}

pub fn uri_vtable() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new(
        "Windows.Foundation.IUriRuntimeClass".to_string(),
        Default::default(),
    );
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 6 get_AbsoluteUri
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 7 get_DisplayUri
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 8 get_Domain
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 9 get_Extension
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 10 get_Fragment
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 11 get_Host
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 12 get_Password
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 13 get_Path
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 14 get_Query
        .add_method(MethodSignature::new()) // 15 get_QueryParsed
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 16 get_RawUri
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 17 get_SchemeName
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 18 get_UserName
        .add_method(MethodSignature::new().add_out(WinRTType::I32)) // 19 get_Port
        .add_method(MethodSignature::new()); // 20 get_Suspicious;
    vtable
}

pub fn IAsyncOperationWithProgress() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new(
        "Windows.Foundation.IAsyncOperationWithProgress".to_string(),
        Default::default(),
    );
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(MethodSignature::new()) // 6 SetProgress
        .add_method(MethodSignature::new()) // 7 GetProgress
        .add_method(MethodSignature::new()) // 8 SetCompleted
        .add_method(MethodSignature::new()) // 9 GetCompleted
        .add_method(MethodSignature::new().add_out(WinRTType::HString)); // 10 GetResults
    vtable
}

pub fn IAsyncOperation() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new(
        "Windows.Foundation.IAsyncOperation".to_string(),
        Default::default(),
    );
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(MethodSignature::new()) // 6 SetCompleted
        .add_method(MethodSignature::new()) // 7 GetCompleted
        .add_method(MethodSignature::new().add_out(WinRTType::Object)); // 8 GetResults
    vtable
}

pub fn FileOpenPickerFactory() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new(
        "Windows.Storage.Pickers.IFileOpenPickerFactory".to_string(),
        Default::default(),
    );
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(
            MethodSignature::new()
                .add(WinRTType::I64)
                .add_out(WinRTType::Object),
        ); // 6 CreateWithMode
    vtable
}

pub fn PickFileResult() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new(
        "Windows.Storage.Pickers.PickFileResult".to_string(),
        Default::default(),
    );
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(MethodSignature::new().add_out(WinRTType::HString)); // 6 get_File
    vtable
}

pub fn FileOpenPicker() -> InterfaceSignature {
    let mut vtable = InterfaceSignature::new(
        "Windows.Storage.Pickers.IFileOpenPicker".to_string(),
        Default::default(),
    );
    vtable
        .add_method(MethodSignature::new()) // 0 QueryInterface
        .add_method(MethodSignature::new()) // 1 AddRef
        .add_method(MethodSignature::new()) // 2 Release
        .add_method(MethodSignature::new()) // 3 GetIids
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 4 GetRuntimeClassName
        .add_method(MethodSignature::new()) // 5 GetTrustLevel
        .add_method(MethodSignature::new().add(WinRTType::I32)) // 6 put_ViewMode
        .add_method(MethodSignature::new().add_out(WinRTType::I32)) // 7 get_ViewMode
        .add_method(MethodSignature::new().add(WinRTType::Object)) // 8 put_SuggestedStartLocation
        .add_method(MethodSignature::new().add_out(WinRTType::Object)) // 9 get_SuggestedStartLocation
        .add_method(MethodSignature::new().add(WinRTType::HString)) // 10 put_CommitButtonText
        .add_method(MethodSignature::new().add_out(WinRTType::HString)) // 11 get_CommitButtonText
        .add_method(MethodSignature::new().add_out(WinRTType::Object)) // 12 get_FileTypeFilter
        .add_method(MethodSignature::new().add_out(WinRTType::Object)); // 13 PickSingleFileAsync
    vtable
}
