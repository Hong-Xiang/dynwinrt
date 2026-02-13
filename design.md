# Dynamic Projection for JavaScript and Python

## Objective

The primary objective of this project is to expose WinRT Windows APIs to dynamic languages, specifically JavaScript and Python, while ensuring a seamless developer experience. This approach eliminates the need for native compilation (such as C++ compilers) and removes strict version coupling between the projection and the Windows App SDK (WASDK) components.

## Motivation and Challenges

### Versioning Complexity

Current static projections, such as existing PyWinRT, necessitate the generation and compilation of native code for specific versions of WinRT components. This requirement results in a complex compatibility matrix and mandates the release of new projection packages for every WASDK release, and more importantly, does not expose full WinRT APIs to developers due to AOT compilation constraints.

### Developer Experience (DX)

Developers in the Node.js, Electron, and Python ecosystems expect a streamlined setup process, typically via `npm install` or `pip install`. They should not be required to configure MSVC tools or integrate C++ compilers into their build chains.

### Static vs. Dynamic Nature

A key distinction exists between static and dynamic language projections:

*   **Static Languages (C++, Rust, C#)**: These languages possess full type information at compile time, allowing static projections to tailor bindings precisely to what is utilized.
*   **Dynamic Languages (JavaScript, Python)**: These rely on runtime evaluation, where objects are often type-erased. Static projections can create gaps when a function encounters an unknown WinRT object at runtime that was not statically projected.

*Note: Further analysis on limitation of statical projection, is it trimming, generic, and maybe more limitations?*

## Architecture

The proposed architecture transitions from static native bindings to a dynamic approach, comprising two primary, separable components:

---

## **Implementation Status** (as of 2025)

✅ **Rust Core Runtime** ([dynwinrt](https://github.com/Hong-Xiang/dynwinrt)) is functional with:
- Complete 4-layer type system (WinRTType, AbiType, WinRTValue, AbiValue)
- Both libffi-based dynamic calls and direct type-safe calls
- WinRT activation factories and QueryInterface
- IAsyncOperation support with custom Future implementations
- WinAppSDK Bootstrap initialization
- WinMD metadata reading (via windows-metadata crate)
- Hybrid approach: static bindings + dynamic calls

🚧 **In Progress**:
- Automatic InterfaceSignature generation from WinMD
- Expanded direct call variants (currently 0-3 parameters)
- Generic interface support beyond IAsyncOperation

📋 **Not Started**:
- Value types (struct) passed by value
- JavaScript bindings (napi-rs integration)
- Python bindings (PyO3 integration)

---

### The Runtime Library

This component is a minimal runtime library native to the target ecosystem (e.g., a `.pyd` module or Node.js addon) that facilitates dynamic calls to arbitrary WinRT APIs.

#### FFI and ABI Handling

**Status**: ✅ Implemented using libffi

A minimal dynamic Foreign Function Interface (FFI) layer is responsible for:
*   ✅ Invoking arbitrary WinRT methods via function pointers with correct parameter type information using `libffi`
*   ✅ Managing `out` parameters through stack allocation via `AbiValue`
*   ✅ Direct type-safe calls for known signatures (bypassing libffi overhead)
*   ⚠️ Partial support of WinRT type system (primitives, objects, HSTRINGs, IAsyncOperation)
*   ❌ Not yet: Value types passed by value, struct size/alignment computation

This infrastructure can be mostly shared across dynamic languages.

**Implementation**: See [call.rs](src/call.rs), [abi.rs](src/abi.rs), [types.rs](src/types.rs)

#### Platform Primitives

**Status**: ✅ Implemented

*   ✅ The library wraps fundamental OS APIs: `RoInitialize`, `RoGetActivationFactory`, `QueryInterface`
*   ✅ **WinAppSDK Bootstrap**: Dynamic DLL loading and `MddBootstrapInitialize2` invocation for unpackaged applications

This infrastructure can be mostly shared across dynamic languages.

**Implementation**: See [roapi.rs](src/roapi.rs), [winapp.rs](src/winapp.rs)

#### Language Adaptation

**Status**: ⚠️ Partially implemented (Rust level only, JS/Python bindings not started)

*   ✅ `HSTRING` handling via `windows-core` crate
*   ✅ `HRESULT` to Result conversion via custom error types
*   ✅ `IAsyncOperation` to Rust Future via custom implementations
*   ❌ Not yet: JavaScript Promise integration (requires napi-rs bindings)
*   ❌ Not yet: Python awaitable integration (requires PyO3 bindings)

**Implementation**: See [result.rs](src/result.rs), [dasync.rs](src/dasync.rs), [value.rs](src/value.rs)

### Metadata Parser and Projection Generator

**Status**: ⚠️ WinMD reading works, code generation not yet implemented

This component bridges the gap between raw WinMD metadata and the runtime projection, operating in two distinct modes:

#### Mode A: Fully Lazy Assessment (Runtime)

**Status**: 🚧 Partial - can read WinMD but not auto-generate signatures

In this mode, the runtime parses `.winmd` files on the fly as APIs are accessed.
*   **Current**: WinMD reading demonstrated via `windows-metadata` crate (see [lib.rs:241-316](src/lib.rs#L241-L316))
*   **Working**: Can read type definitions, method signatures, parameter info
*   **Missing**: Automatic conversion from metadata to `InterfaceSignature` objects
*   **Advantages**: Proven stability in previous JavaScript projections and simpler distribution (no generation step required).
*   **Disadvantages**: Incurs runtime parsing overhead (potentially negligible compared to marshalling) and lacks IDE IntelliSense support.

#### Mode B: Design-Time Generation (Pre-processed)

**Status**: ❌ Not started

A CLI tool parses `.winmd` files to generate **non-native** code (pure `.js` or `.py` files) that defines interface shapes and method signatures for the runtime. This mode can also generate IDE helpers, such as TypeScript `.d.ts` files and Python `.pyi` stubs.
*   **Advantages**: Enhanced Developer Experience (IntelliSense/Autocomplete) and faster startup times (eliminating WinMD parsing).
*   **Disadvantages**: Requires a generation step, although strictly without native compilation.
*   **Current Workaround**: Hand-written signatures in [interfaces.rs](src/interfaces.rs) and static bindings via windows-bindgen for WinAppSDK types

## Developer Workflow

The usage workflow involves two stages:

### Step 1: Runtime Distribution
The runtime is distributed as a generic library for the target language (e.g., `pip install lazy-winrt`).

### Step 2: Projection Usage
Developers have two options for using the projection:
1.  **Direct Usage**: The runtime directly supports runtime interface specifications. Developers can use libraries directly by lazily loading namespaces with distributed WinMDs. The runtime parses the WinMDs and generates necessary interface shapes on the fly.
2.  **Generated Bindings**: Alternatively, developers can execute a tool (e.g., `npx lazy-winrt-gen`) to generate bindings and types specifically for the WinMDs they intend to use.

## Performance Considerations

### Overhead Analysis

The primary performance costs are attributed to WinMD parsing (in lazy mode) and the overhead of dynamic WinRT method invocation (dynamic dispatch). This invocation overhead is expected to be comparable to existing marshalling costs associated with crossing the JavaScript/Python boundary.

### Optimization Strategies

Adopting a hybrid approach—specifically, the design-time generation of interface shapes—can eliminate runtime WinMD parsing costs, reducing overhead strictly to FFI operations.

## Implementation Strategy

The runtime provides a minimal representation of WinRT types and values, along with conversions to language-native equivalents. It also offers a minimal interface specification language for the target language, enabling users to define WinRT interfaces and classes at runtime.

### Runtime Interface Specification Example

TypeScript interface as demonstrated below,
thus with the minimum runtime provided by the `lazy-winrt`,
all WinRT APIs can be specified and invoked dynamically using the target language.


```ts
import { WinRT } from "lazy-winrt";

const UriInterface = WinRT.Interface({
  namespace: "Windows.Foundation",
  name: "IUriRuntimeClass",
  guid: "<...guid of the interface...>",
  methods: [
    // get AbsoluteUri(): string
    WinRT.Method([WinRT.Out(WinRT.HSTRING)]), // Implicitly assumes first argument is ComPtr, result is HRESULT
    // get Domain(): string
    WinRT.Method([WinRT.Out(WinRT.HSTRING)]),
    // ... other methods
  ],
});

const UriFactoryInterface = WinRT.Interface({
  namespace: "Windows.Foundation",
  name: "IUriRuntimeClassFactory",
  guid: "<...guid of the interface...>",
  methods: [
    // CreateUri(string uri): IUriRuntimeClass
    WinRT.Method([WinRT.HSTRING, WinRT.Out(UriInterface)]),
  ],
});

class Uri {
  // Statically cached factory object in target dynamic language
  static Factory = WinRT.as(UriFactoryInterface, WinRT.getActivationFactory("Windows.Foundation.Uri"));

  constructor(uriString: string) {
    this._instance = WinRT.callMethod(UriFactoryInterface, 7, [
      uriString,
    ]);
  }

  get Host(): string {
    return WinRT.callMethod(UriInterface, 11, [this._instance]); // 11 is the vtable index of get_Host
  }
}
```

The WinMD parser allows for the generation of these interface specifications and developer-friendly projection classes at design time or runtime.

*TODO: workout desired async handling interface and generic support*.

### Stub Method Optimizations

**Status**: ✅ Implemented in Rust

To optimize performance, common method signatures can be implemented within the runtime library. This allows frequent operations to execute with speeds comparable to static projections.

**Current Implementation** ([call.rs:17-60](src/call.rs#L17-L60)):
```rust
// Direct type-safe calls for known signatures
pub fn call_winrt_method_1<T1>(vtable_index: usize, obj: *mut c_void, x1: T1) -> HRESULT;
pub fn call_winrt_method_2<T1, T2>(vtable_index: usize, obj: *mut c_void, x1: T1, x2: T2) -> HRESULT;
pub fn call_winrt_method_3<T1, T2, T3>(vtable_index: usize, obj: *mut c_void, x1: T1, x2: T2, x3: T3) -> HRESULT;
```

These are used by `WinRTValue::call_single_out()` ([value.rs:83-138](src/value.rs#L83-L138)) to avoid libffi overhead when the signature is known at runtime. Since only the actual ABI signature matters, many WinRT methods map to these stubs (e.g., getters, simple factories).

**Performance**: Direct calls avoid CIF construction and libffi dispatch, providing performance comparable to static projections while maintaining runtime flexibility.

**Future Work**: Generate more variants (4-8 parameters) and specialize for common patterns (getter, setter, factory).

### Known Challenges

#### Resolved ✅
*   ~~**Async Operations**~~: Custom Future implementations work for IAsyncOperation<T>

#### In Progress 🚧
*   **Signature Generation**: Manual interface signatures work, but need automatic WinMD → InterfaceSignature conversion
*   **Generic Type GUIDs**: Need runtime GUID computation for parameterized interfaces (IVector&lt;T&gt;, IAsyncOperation&lt;T&gt;)

#### Not Started ❌
*   **Value Types**: Struct size/alignment calculation at runtime (no sizeof available)
*   **Representation Mapping**: Special handling for JavaScript/Python representations; for example, `IVector` may need to map to array-like objects
*   **Generic Interface Support**: Full generic type support beyond IAsyncOperation

## 7. References

*   **Legacy JS Projection**: Historical JavaScript applications utilized a dynamic projection where the runtime read WinMDs, achieving acceptable performance.
*   [PyWinRT](https://github.com/pywinrt/pywinrt) Static C++/WinRT-based projections demonstrated significant versioning and distribution challenges.
*  [lazy-winrt](https://github.com/JesseCol/lazy-winrt) **"Lazy-WinRT" Prototype**: This prototype validated the feasibility and potential performance of parsing WinMDs and invoking methods dynamically 
*  dynwinrt: A Rust-based implementation inspired by Lazy-WinRT, leveraging `napi-rs` and `PyO3` to facilitate integration with JavaScript and Python.
* [Rust Core Runtime Lib](https://github.com/Hong-Xiang/dynwinrt)
* [JS Binding](https://github.com/Hong-Xiang/dynwinrt-js)
* [Py Binding](https://github.com/Hong-Xiang/dynwinrt-py)

