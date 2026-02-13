# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`dynwinrt` is a Rust-based runtime library that enables dynamic invocation of Windows Runtime (WinRT) APIs. Unlike static projections (PyWinRT, C++/WinRT), this library uses runtime metadata (.winmd files) and FFI (libffi) to call arbitrary WinRT methods without native code generation. The goal is to provide a foundation for JavaScript and Python bindings that don't require MSVC compilation or version-specific generated code.

## Build Commands

```bash
# Build the library
cargo build

# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Build in release mode
cargo build --release
```

## Environment Setup

**Critical**: Set the `WINAPPSDK_BOOTSTRAP_DLL_PATH` environment variable to the path of the WinAppSDK Bootstrap DLL before running tests that use WinAppSDK APIs (e.g., FileOpenPicker). Without this, WinAppSDK initialization will fail.

## Architecture

### Core Type System

The library implements a four-layer type system for bridging Rust and WinRT ABIs:

1. **WinRTType** ([types.rs:7](src/types.rs#L7)): High-level type descriptors including:
   - Primitives: I32, I64
   - Complex types: Object, HString, HResult
   - Advanced: IAsyncOperation(GUID), ArrayOfIUnknown, OutValue
2. **AbiType** ([abi.rs:2](src/abi.rs#L2)): ABI-level type representations (I32, I64, Ptr) that match calling conventions
3. **WinRTValue** ([value.rs:26](src/value.rs#L26)): Runtime value containers with rich operations:
   - Direct method invocation: `call_single_out()`, `call_single_out_2()`
   - Type casting: `cast(&iid)` for QueryInterface operations
   - Factory support: `from_activation_factory(class_name)`
   - Async operation wrapping with custom Future implementations
4. **AbiValue** ([abi.rs:26](src/abi.rs#L26)): Low-level ABI value storage for stack allocation

This separation allows the library to handle WinRT's type system dynamically while maintaining correct ABI compatibility and providing both high-performance direct calls and flexible dynamic dispatch.

### Method Invocation Mechanisms

The library provides two complementary approaches for calling WinRT methods:

#### 1. Direct Calls (High Performance)

For known signatures, direct type-safe calls ([call.rs:17-60](src/call.rs#L17-L60)):
- `call_winrt_method_1<T1>()`: Single parameter methods
- `call_winrt_method_2<T1, T2>()`: Two parameter methods
- `call_winrt_method_3<T1, T2, T3>()`: Three parameter methods

Used by `WinRTValue::call_single_out()` and `call_single_out_2()` for optimal performance when the method signature is known at runtime.

**Example**: See [lib.rs:211](src/lib.rs#L211) `test_uri_call_dynamic` for direct call usage.

#### 2. Dynamic Calls (Fully Flexible)

For arbitrary signatures via libffi ([call.rs:62](src/call.rs#L62), [signature.rs](src/signature.rs)):

1. **InterfaceSignature** describes a WinRT interface with its GUID and method list
2. **MethodSignature** describes each method's parameters (in/out) and types, builds libffi CIF
3. At runtime, the library:
   - Extracts function pointers from COM vtables using `get_vtable_function_ptr`
   - Allocates stack space for out parameters via `AbiValue`
   - Marshals arguments using libffi `Cif`
   - Invokes the method via `call_winrt_method_dynamic`
   - Converts out parameters back to WinRTValue types

**Example**: See [lib.rs:182](src/lib.rs#L182) `test_winrt_uri_interop_using_signature` for dynamic method invocation.

### Key Modules

- **call.rs**: Core FFI invocation logic
  - `get_vtable_function_ptr()`: Extracts method pointers from COM vtables
  - Direct calls: `call_winrt_method_1/2/3()` for known signatures
  - `call_winrt_method_dynamic()`: libffi-based dynamic invocation
- **signature.rs**: Interface and method signature definitions
  - `InterfaceSignature`: Describes WinRT interfaces with GUID and method lists
  - `MethodSignature`: Builder pattern for method parameter definitions
  - `Method`: Compiled method with pre-built libffi CIF for performance
- **types.rs**: Type system mapping (WinRTType ↔ AbiType)
- **abi.rs**: Low-level ABI representations (AbiType, AbiValue)
- **value.rs**: Runtime value containers with rich operations
  - `WinRTValue`: High-level wrapper with method call helpers
  - Factory creation, casting, async support, libffi argument marshaling
- **interfaces.rs**: Hand-written interface signatures (Uri, FileOpenPicker, IAsyncOperation, etc.)
  - Examples demonstrating signature definition patterns
  - Will be replaced by WinMD code generation in future
- **roapi.rs**: WinRT activation factory access via RoGetActivationFactory
- **winapp.rs**: WinAppSDK Bootstrap initialization
  - Dynamic DLL loading and MddBootstrapInitialize2 invocation
  - Package discovery via PackageManager
- **dasync.rs**: Custom Future implementations
  - `DynWinRTAsyncOperationWithProgress`: Async polling for IAsyncOperationWithProgress
  - `DynWinRTAsyncOperationIUnknown`: Async support for Object-returning operations
- **bindings.rs**: Static WinAppSDK bindings (generated via windows-bindgen)
  - Used for specific WinAppSDK types (TextRecognizer, ImageBuffer, etc.)
  - Demonstrates hybrid approach: static types + dynamic calls
- **result.rs**: Error handling with custom Result type

## Design Philosophy

### Dynamic vs Static Projection

This library intentionally uses **runtime** rather than **compile-time** approach:

- Interface shapes are represented as data (`InterfaceSignature`) not generated code
- WinMD metadata can be read at runtime or pre-processed
- No version coupling between the runtime and specific WinAppSDK versions
- Trades compile-time type safety for flexibility and ease of distribution

### Out Parameter Handling

WinRT methods use out parameters (pointers) for return values. The library handles this by:

1. Pre-allocating `AbiValue` storage on the stack
2. Passing pointers to these allocations via libffi
3. Converting the populated `AbiValue` to `WinRTValue` after the call succeeds

See `call_winrt_method_dynamic` in `call.rs:59` for implementation details.

## Testing Strategy

Tests use real Windows APIs without mocking:

- **Basic WinRT**: Uri, XmlDocument (from Windows.winmd)
  - [lib.rs:146](src/lib.rs#L146): `test_winrt_uri` - Static projection test
  - [lib.rs:161](src/lib.rs#L161): `test_winrt_uri_interop_using_libffi` - Raw libffi invocation
  - [lib.rs:182](src/lib.rs#L182): `test_winrt_uri_interop_using_signature` - Dynamic signature-based calls
  - [lib.rs:211](src/lib.rs#L211): `test_uri_call_dynamic` - Direct call optimization
- **Windows Web**: HttpClient with async operations
  - [lib.rs:106](src/lib.rs#L106): `http_call` - Async HTTP request test
- **WinAppSDK**: FileOpenPicker, TextRecognizer (OCR), ImageBuffer
  - Requires WinAppSDK Bootstrap initialization via `WINAPPSDK_BOOTSTRAP_DLL_PATH`
  - [lib.rs:366](src/lib.rs#L366): `windows_ai_ocr_api_call_projected` - Static projection
  - [lib.rs:509](src/lib.rs#L509): `windows_ai_ocr_api_call_dynamic` - Dynamic invocation
  - [lib.rs:438](src/lib.rs#L438): `ocr_demo` - Fully dynamic OCR pipeline
- **Metadata Reading**: Direct windows-metadata crate tests
  - [lib.rs:241](src/lib.rs#L241): `test_winmd_read` - Reading Point struct definition
  - [lib.rs:267](src/lib.rs#L267): `test_winmd_read_uri` - Reading Uri interface methods
  - [lib.rs:292](src/lib.rs#L292): `test_winmd_read_http_client` - Reading HttpClient methods
  - **Status**: WinMD reading works but not yet integrated into automatic signature generation

All tests assume Windows 10/11 with SDK installed at `C:\Program Files (x86)\Windows Kits\10\UnionMetadata\10.0.26100.0\Windows.winmd`.

## Common Patterns

### Creating an Interface Signature

```rust
// IInspectable-based interface (most WinRT types)
let mut iface = InterfaceSignature::define_from_iinspectable("IUri", uri_iid);
iface
    .add_method(MethodSignature::new().add_out(WinRTType::HString)) // get_Host
    .add_method(MethodSignature::new().add_out(WinRTType::I32));     // get_Port

// IUnknown-based interface (COM-only)
let mut iface = InterfaceSignature::define_from_iunknown("IUriFactory", factory_iid);
iface
    .add_method(MethodSignature::new()
        .add(WinRTType::HString)
        .add_out(WinRTType::Object)); // CreateUri

// Build methods with vtable index
let method = MethodSignature::new().add_out(WinRTType::HString).build(vtable_index);
```

### Calling Methods

```rust
// Dynamic call (flexible, slightly slower)
let result = method.call_dynamic(com_obj.as_raw(), &[WinRTValue::HString(hstring)])?;
let return_value = result[0].as_hstring().unwrap();

// Direct call (faster, requires WinRTValue wrapper)
let obj_value = WinRTValue::Object(com_obj);
let result = obj_value.call_single_out(vtable_index, &WinRTType::HString, &[])?;
let host = result.as_hstring().unwrap();

// With parameters
let result = obj_value.call_single_out(
    method_index,
    &WinRTType::Object,
    &[WinRTValue::I32(42)]
)?;
```

### Working with Factories

```rust
// Get activation factory
let factory = WinRTValue::from_activation_factory(h!("Windows.Foundation.Uri"))?;

// Cast to specific interface
let uri_factory = factory.cast(&IIds::IUriRuntimeClassFactory)?;

// Call factory method
let uri = uri_factory.call_single_out(
    6, // CreateUri method index
    &WinRTType::Object,
    &[WinRTValue::HString(h!("https://example.com").clone())]
)?;
```

## Current Status and Limitations

### What Works
- ✅ Basic types: I32, I64, Object, HString, HResult
- ✅ IAsyncOperation<T> with custom Future implementations
- ✅ Array types (ArrayOfIUnknown)
- ✅ Direct calls for known signatures (optimal performance)
- ✅ Dynamic calls via libffi (full flexibility)
- ✅ WinRT activation factories
- ✅ QueryInterface and type casting
- ✅ WinAppSDK Bootstrap initialization
- ✅ WinMD metadata reading (via windows-metadata crate)

### Known Limitations
- ❌ Structs/value types passed by value (no sizeof at runtime)
- ❌ Generic interfaces beyond IAsyncOperation (IVector&lt;T&gt;, IMap&lt;K,V&gt;, etc.)
- ❌ Automatic signature generation from WinMD (currently hand-written)
- ❌ Automatic vtable index calculation (must be manually counted)
- ⚠️ Direct call helpers only support 0-1 parameters currently (need more variants)

### Next Steps (Roadmap)
1. **WinMD Integration**: Parse .winmd files to automatically generate InterfaceSignature definitions
2. **Direct Call Expansion**: Generate type-safe direct call variants for more parameter combinations
3. **Generic Type Support**: Implement GUID computation for parameterized interfaces
4. **Value Type Support**: Implement struct layout and alignment calculation at runtime

## Related Projects

- [lazy-winrt](https://github.com/JesseCol/lazy-winrt): Original JavaScript prototype
- [JS Binding](https://github.com/Hong-Xiang/dynwinrt-js): JavaScript bindings using napi-rs
- [Python Binding](https://github.com/Hong-Xiang/dynwinrt-py): Python bindings using PyO3

## Implementation Notes

### Why libffi?

libffi provides portable FFI that can call functions with arbitrary signatures at runtime. This is essential because WinRT method signatures are only known after reading WinMD metadata.

### COM Object Lifetimes

The library uses `windows-core::IUnknown` smart pointers which automatically handle AddRef/Release. Raw pointers extracted via `as_raw()` are only used for the duration of a single call.

### Async Operations

The library implements custom Future traits ([dasync.rs](src/dasync.rs)) for dynamic async operations:

- **DynWinRTAsyncOperationWithProgress**: Handles `IAsyncOperationWithProgress<T, TProgress>`
  - Polls AsyncStatus until completed
  - Dynamically calls GetResults() via vtable
  - Used for HTTP operations and progress-reporting tasks

- **DynWinRTAsyncOperationIUnknown**: Handles `IAsyncOperation<T>` returning Object types
  - Generic implementation for any IAsyncOperation
  - Extracts IUnknown result via QueryInterface

- **WinRTValue Integration**: `IAsyncOperation` variant wraps IAsyncInfo + GUID
  - Implements `IntoFuture` via reference (`&WinRTValue`)
  - See [lib.rs:438](src/lib.rs#L438) `ocr_demo` for fully dynamic async pipeline

This allows `.await` on dynamically-invoked async methods without requiring compile-time generic parameters.
