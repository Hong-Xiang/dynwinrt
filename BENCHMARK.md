# dynwinrt Benchmark Results

C++/WinRT static projection vs dynamic invocation (dynwinrt).

Environment: Snapdragon X Elite, Windows 11, Node.js v22.

---

## JavaScript End-to-End

Static path:  JS → **node-addon-api** → **C++/WinRT** → COM vtable (pure C++ addon, no Rust).
Dynamic path: JS → **napi-rs** → **dynwinrt** (Rust + libffi) → COM vtable.

All method handles and objects pre-created before measurement.

### By parameter count

Fixed return type, varying number of input parameters:

| Params | API | Static | Dynamic | Ratio | Overhead |
|--------|-----|--------|---------|-------|----------|
| 0 in → 1 out | `Uri.get_Host()` | 80 ns | 1.09 µs | 13.6x | +1.01 µs |
| 1 in → 1 out | `Uri.CombineUri(hstring)` | 1.31 µs | 3.38 µs | 2.6x | +2.07 µs |
| 2 in → 1 out | `Uri.CreateWithRelativeUri(hstring, hstring)` | 1.38 µs | 5.13 µs | 3.7x | +3.75 µs |

### By input type

Fixed call shape (1 in → 1 out object), varying input type:

| Input type | API | Static | Dynamic | Ratio | Overhead |
|------------|-----|--------|---------|-------|----------|
| i32 | `PropertyValue.CreateInt32(42)` | 370 ns | 1.95 µs | 5.3x | +1.58 µs |
| f64 | `PropertyValue.CreateDouble(3.14)` | 370 ns | 3.09 µs | 8.4x | +2.72 µs |
| bool | `PropertyValue.CreateBoolean(true)` | 334 ns | 3.00 µs | 9.0x | +2.67 µs |
| hstring | `PropertyValue.CreateString("hello")` | 393 ns | 4.17 µs | 10.6x | +3.78 µs |
| struct (3×f64) | `Geopoint.Create(BasicGeoposition)` | 410 ns | 6.70 µs | 16.3x | +6.29 µs |

### By return type

Fixed call shape (0 in → 1 out getter on pre-created Uri), varying return type.
Dynamic includes `.toNumber()` / `.toBool()` / `.toString()` to match static's direct JS value return.

| Return type | API | Static | Dynamic | Ratio | Overhead |
|-------------|-----|--------|---------|-------|----------|
| i32 | `Uri.get_Port()` | 48 ns | 1.55 µs | 32x | +1.50 µs |
| bool | `Uri.get_Suspicious()` | 47 ns | 1.78 µs | 38x | +1.73 µs |
| hstring | `Uri.get_Host()` | 233 ns | 1.24 µs | 5.3x | +1.01 µs |

High ratios on i32/bool are because C++/WinRT getters are extremely fast (~48ns). Absolute overhead is consistent (~1-1.7µs) across all return types.

### By parameter count (with cached args)

Same as above, but args pre-created outside the loop — isolates pure invoke overhead:

| Params | API | Static | Dynamic (cached) | Ratio | Overhead |
|--------|-----|--------|-------------------|-------|----------|
| 0 in → 1 out | `Uri.get_Host()` | 80 ns | 1.09 µs | 13.6x | +1.01 µs |
| 1 in → 1 out | `Uri.CombineUri(hstring)` | 1.31 µs | 2.95 µs | 2.3x | +1.64 µs |
| 2 in → 1 out | `Uri.CreateWithRelativeUri(hstring, hstring)` | 1.38 µs | 2.63 µs | 1.9x | +1.25 µs |

With cached args, overhead drops to **~1-1.6µs** regardless of param count.

### Arg caching (napi call savings)

Same `PropertyValue.Create*(v)`, comparing cached arg (1 napi call) vs uncached (2 napi calls):

| Type | Static | Cached (1 napi) | Uncached (2 napi) | Cached ratio | Uncached ratio |
|------|--------|----------------|-------------------|--------------|----------------|
| i32 | 370 ns | 1.35 µs | 4.95 µs | 3.6x | 13.4x |
| f64 | 370 ns | 1.36 µs | 2.85 µs | 3.7x | 7.7x |
| bool | 334 ns | 1.29 µs | 3.29 µs | 3.9x | 9.8x |
| hstring | 393 ns | 1.58 µs | 3.10 µs | 4.0x | 7.9x |

Note: JS benchmarks have ~30% run-to-run variance due to V8 JIT/GC. The key finding is consistent: **caching saves ~1-3.6µs per argument** (one fewer napi boundary crossing). If `invoke()` could accept raw JS values directly (planned optimization), this saving would be automatic.

### Method handle caching

| Approach | Time |
|----------|------|
| Cached | 2.34 µs |
| Uncached (lookup by name each call) | 2.65 µs |
| **Savings** | **~300 ns/call** |

### Batch workload

| Workload | Static | Dynamic | Ratio | Overhead |
|----------|--------|---------|-------|----------|
| 200× create Uri + read Host | 395 µs | 2.10 ms | 5.3x | +1.71 ms |

### How to run

```bash
# Build both addons
cd bindings/js && npm run build
cd bindings/js/static-bench && npm install && npm run build

# Run benchmark
cd bindings/js && npx tsx samples/benchmark.ts
```

---

## Rust Core Engine

Same WinRT operations, measured in pure Rust with criterion.rs.

### By parameter count

| Params | API | Static | Dynamic | Ratio | Overhead |
|--------|-----|--------|---------|-------|----------|
| 0 in → 1 out | `Uri.get_Host()` | 6.7 ns | 65 ns | 9.7x | +58 ns |
| 1 in → 1 out | `Uri.CombineUri(hstring)` | 2.5 µs | 2.8 µs | 1.1x | +300 ns |
| 2 in → 1 out | `Uri.CreateWithRelativeUri(hstring, hstring)` | 684 ns | 953 ns | 1.4x | +269 ns |

### By input type

| Input type | API | Static | Dynamic | Ratio | Overhead |
|------------|-----|--------|---------|-------|----------|
| i32 | `PropertyValue.CreateInt32(42)` | 105 ns | 155 ns | 1.5x | +50 ns |
| f64 | `PropertyValue.CreateDouble(3.14)` | 105 ns | 152 ns | 1.4x | +47 ns |
| bool | `PropertyValue.CreateBoolean(true)` | 103 ns | 151 ns | 1.5x | +48 ns |
| hstring | `PropertyValue.CreateString("hello")` | 107 ns | 322 ns | 3.0x | +215 ns |
| object | `PropertyValue.CreateInspectable(obj)` | 7.1 ns | 57 ns | 8.0x | +50 ns |
| struct (3×f64) | `Geopoint.Create(BasicGeoposition)` | 116 ns | 618 ns | 5.3x | +502 ns |

### By return type

| Return type | API | Static | Dynamic | Ratio | Overhead |
|-------------|-----|--------|---------|-------|----------|
| i32 | `Uri.get_Port()` | 2.8 ns | 50 ns | 17.9x | +47 ns |
| bool | `Uri.get_Suspicious()` | 3.6 ns | 135 ns | 37.5x | +131 ns |
| hstring | `Uri.get_Host()` | 7.6 ns | 72 ns | 9.5x | +64 ns |
| object | `Uri.CombineUri(hstring)` | 735 ns | 1.04 µs | 1.4x | +303 ns |

### Overhead isolation

| Path | hstring getter | i32 getter |
|------|---------------|------------|
| Static (compiler-inlined) | 6.6 ns | 2.8 ns |
| Raw vtable (no framework) | 7.4 ns | 24 ns |
| Dynamic (dynwinrt) | 51 ns | 72 ns |
| **Framework overhead** | **+43 ns** | **+48 ns** |

### Batch workload

| Workload | Static | Dynamic | Ratio | Overhead |
|----------|--------|---------|-------|----------|
| 100× create Uri + read property | 225 µs | 241 µs | 1.07x | +16 µs |

### How to run

```bash
cargo bench -p dynwinrt --bench bench
```

---

## Overhead by layer

| Layer | Per-call overhead | Source |
|-------|-------------------|--------|
| COM vtable dispatch | <1 ns | Function pointer indirection |
| dynwinrt Rust core | ~50 ns | RwLock + Vec alloc + value marshaling |
| JS ↔ Rust napi boundary | ~500-1000 ns | V8 value conversion, argument wrapping |
| C++/WinRT (node-addon-api) | ~50-400 ns | Direct vtable call + V8 string conversion |
