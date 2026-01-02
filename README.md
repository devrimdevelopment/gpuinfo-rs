# gpuinfo-rs

[![GitHub](https://img.shields.io/badge/github-view_repo-blue?logo=github)](https://github.com/devrimdevelopment/gpuinfo-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/devrimdevelopment/gpuinfo-rs)

A unified, lightweight Rust library for querying GPU hardware metadata on **Linux** and **Android** systems.

`gpuinfo-rs` provides a safe, high-level API to interact directly with kernel drivers (ARM Mali and Qualcomm Adreno/KGSL) via ioctls, retrieving model names, revision codes, architecture details, and hardware capabilities.

It is specifically designed for **mobile SoCs** and **embedded / single-board computers (SBCs)** where standard graphics APIs (Vulkan, OpenGL) are either unavailable or insufficient for low-level hardware identification.

---

## Why gpuinfo-rs?

Retrieving accurate GPU information on mobile and embedded Linux is notoriously difficult:

* Vendor-specific kernel interfaces (ioctls) hide the real hardware IDs.
* Many solutions rely on fragile `/sys` parsing or require root privileges.

`gpuinfo-rs` solves this by:

* **Zero external dependencies** – communicates directly with the kernel
* **No heavy graphics libraries** required at compile- or runtime
* **Ultra lightweight** – feature-gated to include only the vendors you need
* **No root required** – works in normal user-space on standard Android/Linux setups
* **Type-safe & idiomatic Rust** – safe wrappers around raw ioctls and bitfields
---

## Supported GPUs & Platforms

| Vendor   | GPUs Supported                     | Typical Devices / SBCs                                                         |
| -------- | ---------------------------------- | ------------------------------------------------------------------------------ |
| ARM Mali | Midgard, Bifrost, Valhall, 5th Gen | Rockchip, Amlogic, MediaTek |
| Qualcomm | Adreno 6xx, 7xx (KGSL)             | Snapdragon-based devices    |

---

## Planned Support

* **Broadcom VideoCore** (V3D and earlier) → **Raspberry Pi** (Pi 4, Pi 5, etc.)
* **NVIDIA Tegra / Orin** (Ampere architecture) → **Jetson** series (Nano, Orin, AGX)

These additions will allow the library to cover virtually **all popular SBCs** (Raspberry Pi, Jetson, Rockchip-based boards, etc.).

---

## Features

* ARM Mali support via kernel ioctls
* Qualcomm Adreno support via KGSL
* Smart auto-detection of GPU driver nodes (`/dev/mali0`, `/dev/kgsl-3d0`)
* Feature-based compilation to keep binaries small
* Multiple query modes (basic info or extended hardware details)
* Safe, zero-cost abstractions over raw driver interfaces

---

## Installation

Add this to your `Cargo.toml`:

```text
[dependencies]
gpuinfo-rs = { git = "https://github.com/devrimdevelopment/gpuinfo-rs" }
```


```text
[dependencies]
gpuinfo-rs = "0.1.0" 
```

---

## Usage

### 1. Auto-Detection (Recommended)

The simplest way – automatically finds the active GPU driver.

```rust
use gpuinfo_rs::query_gpu_auto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = query_gpu_auto(None::<&str>)?; // optional hint path if needed

    println!("GPU Model: {}", info.model_name);
    println!("Vendor: {:?}", info.vendor);
    println!("Architecture: {:?}", info.family);
    println!("Shader Cores: {}", info.num_shader_cores);

    Ok(())
}
```

### 2. Manual Vendor Query

When you know the exact driver node.

```rust
// ARM Mali
let mali_info = gpuinfo_rs::mali::query_mali("/dev/mali0")?;

// Qualcomm Adreno
let adreno_info = gpuinfo_rs::adreno::query_adreno("/dev/kgsl-3d0")?;
```

---

## Build Configuration

Use Cargo features to minimize binary size on embedded targets:

| Feature       | Description                           | Default |
| ------------- | ------------------------------------- | ------- |
| `mali`        | Enable ARM Mali support               | Yes     |
| `adreno`      | Enable Qualcomm Adreno (KGSL) support | Yes     |
| `auto-detect` | Scan `/dev` for GPU nodes             | Yes     |

**Example: Build for Mali-only (e.g. Rockchip SBCs)**

```bash
cargo build --release --no-default-features --features mali
```

---

## Project Structure

```text
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENCE
├── examples/
│ ├── auto_detect_demo.rs             # Auto-detection example
│ ├── adreno_demo.rs                  # Qualcomm Adreno example
│ └── mode_demo.rs                    # Query mode showcase
└── src/
├── lib.rs                            # Public API entry point
├── info.rs                           # Shared GpuInfo structures
├── error.rs                          # Error types
├── detect.rs                         # Auto-detection logic
├── mali/
│ ├── mod.rs
│ ├── query.rs                        # Mali ioctl queries
│ ├── parser.rs                       # Bitfield / register parsing
│ └── database.rs                     # Mali hardware ID lookup tables
└── adreno/
├── mod.rs
├── query.rs                          # Adreno KGSL queries
└── database.rs                       # Adreno chip database
```

---

## Examples

The repository includes demos:

```bash
# Auto-detection demo
cargo run --example auto_detect --features "auto-detect"

# Mali-specific demo
cargo run --example mali_demo --features mali

# Adreno-specific demo
cargo run --example adreno_demo --features adreno
```

---

## Roadmap / Contributing

* Add Broadcom VideoCore support (Raspberry Pi)
* Add NVIDIA Tegra/Orin support (Jetson)
* Extend database coverage (more chip IDs, confidence levels)
* Optional fallback to sysfs/Vulkan when ioctls are unavailable

Contributions are welcome! Especially:

* New chip ID mappings
* Support for additional vendors
* Testing on real hardware

Please open an issue or PR on GitHub.

---

## License

Licensed under either of:
* MIT License ([LICENSE-MIT](LICENSE-MIT))

---

Made with ❤️ for the embedded Rust community.
