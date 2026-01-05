use std::borrow::Cow;
use std::fmt;

/// GPU vendor types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Mali,
    Adreno,
    Unknown,
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuVendor::Mali => write!(f, "ARM Mali"),
            GpuVendor::Adreno => write!(f, "Qualcomm Adreno"),
            GpuVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Mali-specific GPU data
#[derive(Debug, Clone)]
pub struct MaliData {
    pub gpu_id: u32,
    pub raw_gpu_id: u64,
    pub shader_core_mask: u64,
    pub num_l2_slices: u64,
    pub num_exec_engines: u32,
    pub num_fp32_fmas_per_core: u32,
    pub num_fp16_fmas_per_core: u32,
    pub num_texels_per_core: u32,
    pub num_pixels_per_core: u32,
}

/// Adreno-specific GPU data
#[derive(Debug, Clone)]
pub struct AdrenoData {
    pub chip_id: u32,
    pub gpu_model_code: u32,
    pub mmu_enabled: bool,
    pub gmem_size_bytes: u32,
    pub spec_confidence: Cow<'static, str>,  // Geändert von String zu Cow
    pub stream_processors: u32,
    pub max_freq_mhz: u32,
    pub process_nm: u32,
    pub release_year: u32,
    pub snapdragon_models: Vec<Cow<'static, str>>,  // Geändert von Vec<String> zu Vec<Cow>
}

/// Unified GPU information structure
#[derive(Debug, Clone)]
pub struct GpuInfo {
    // Common fields for all GPUs
    pub vendor: GpuVendor,
    pub gpu_name: Cow<'static, str>,           // Geändert von String zu Cow
    pub architecture: Cow<'static, str>,       // Geändert von String zu Cow
    pub architecture_major: u8,
    pub architecture_minor: u8,
    pub num_shader_cores: u32,
    pub num_l2_bytes: u64,
    pub num_bus_bits: u64,

    // Vendor-specific data (optional)
    pub mali_data: Option<MaliData>,
    pub adreno_data: Option<AdrenoData>,
}

impl GpuInfo {
    /// Create a new builder for GpuInfo (Mali-specific, for backward compatibility)
    pub fn builder() -> GpuInfoBuilder {
        GpuInfoBuilder::default()
    }

    /// Check if GPU supports FP16 operations
    pub fn supports_fp16(&self) -> bool {
        match self.vendor {
            GpuVendor::Mali => {
                if let Some(mali) = &self.mali_data {
                    mali.num_fp16_fmas_per_core > 0
                } else {
                    false
                }
            }
            GpuVendor::Adreno => {
                // Adreno 6xx and newer typically support FP16
                self.architecture_major >= 6
            }
            _ => false,
        }
    }

    /// Calculate total FP32 FLOPS at given frequency (in Hz)
    pub fn calculate_fp32_flops(&self, frequency_hz: u64) -> u64 {
        match self.vendor {
            GpuVendor::Mali => {
                if let Some(mali) = &self.mali_data {
                    mali.num_fp32_fmas_per_core as u64 *
                    self.num_shader_cores as u64 *
                    frequency_hz * 2
                } else {
                    0
                }
            }
            GpuVendor::Adreno => {
                // For Adreno: 2 ops per ALU per cycle
                // Using stream processors count from adreno_data if available
                if let Some(adreno) = &self.adreno_data {
                    adreno.stream_processors as u64 * 2 * frequency_hz
                } else {
                    // Fallback: estimate based on shader cores
                    self.num_shader_cores as u64 * 128 * 2 * frequency_hz
                }
            }
            _ => 0,
        }
    }

    /// Get GPU information as a formatted string
    pub fn to_string(&self) -> String {
        match self.vendor {
            GpuVendor::Mali => {
                if !self.gpu_name.is_empty() {
                    if self.num_bus_bits > 0 {
                        format!(
                            "{} ({}), Architecture: {}.{}, Cores: {}, L2: {} KB, Bus: {} bits",
                            self.gpu_name,
                            self.architecture,
                            self.architecture_major,
                            self.architecture_minor,
                            self.num_shader_cores,
                            self.num_l2_bytes / 1024,
                            self.num_bus_bits
                        )
                    } else {
                        format!(
                            "{} ({}), Architecture: {}.{}, Cores: {}, L2: {} KB",
                            self.gpu_name,
                            self.architecture,
                            self.architecture_major,
                            self.architecture_minor,
                            self.num_shader_cores,
                            self.num_l2_bytes / 1024
                        )
                    }
                } else {
                    format!(
                        "GPU ID: 0x{:04X}, Cores: {}, L2: {} KB",
                        self.mali_data.as_ref().map(|m| m.gpu_id).unwrap_or(0),
                        self.num_shader_cores,
                        self.num_l2_bytes / 1024
                    )
                }
            }
            GpuVendor::Adreno => {
                let confidence = if let Some(adreno) = &self.adreno_data {
                    &adreno.spec_confidence
                } else {
                    ""
                };

                format!(
                    "{} ({} {}.{}), Cores: {}, GMEM: {} KB, Bus: {} bits {}",
                    self.gpu_name,
                    self.architecture,
                    self.architecture_major,
                    self.architecture_minor,
                    self.num_shader_cores,
                    self.num_l2_bytes / 1024,
                    self.num_bus_bits,
                    confidence
                )
            }
            _ => format!("Unknown GPU: {}", self.gpu_name),
        }
    }
}

impl fmt::Display for GpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Builder for GpuInfo structure (Mali-specific, for backward compatibility)
#[derive(Debug, Default)]
pub struct GpuInfoBuilder {
    // Common fields
    gpu_name: Option<Cow<'static, str>>,        // Geändert von Option<String> zu Option<Cow>
    architecture: Option<Cow<'static, str>>,    // Geändert von Option<String> zu Option<Cow>
    architecture_major: Option<u8>,
    architecture_minor: Option<u8>,
    num_shader_cores: Option<u32>,
    num_l2_bytes: Option<u64>,
    num_bus_bits: Option<u64>,

    // Mali-specific fields
    gpu_id: Option<u32>,
    raw_gpu_id: Option<u64>,
    shader_core_mask: Option<u64>,
    num_l2_slices: Option<u64>,
    num_exec_engines: Option<u32>,
    num_fp32_fmas_per_core: Option<u32>,
    num_fp16_fmas_per_core: Option<u32>,
    num_texels_per_core: Option<u32>,
    num_pixels_per_core: Option<u32>,
}

impl GpuInfoBuilder {
    // Builder methods (mit Cow-Unterstützung für backward compatibility)
    pub fn gpu_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.gpu_name = Some(name.into());
        self
    }

    pub fn architecture(mut self, arch: impl Into<Cow<'static, str>>) -> Self {
        self.architecture = Some(arch.into());
        self
    }

    pub fn architecture_major(mut self, major: u8) -> Self {
        self.architecture_major = Some(major);
        self
    }

    pub fn architecture_minor(mut self, minor: u8) -> Self {
        self.architecture_minor = Some(minor);
        self
    }

    pub fn gpu_id(mut self, id: u32) -> Self {
        self.gpu_id = Some(id);
        self
    }

    pub fn raw_gpu_id(mut self, id: u64) -> Self {
        self.raw_gpu_id = Some(id);
        self
    }

    pub fn num_shader_cores(mut self, cores: u32) -> Self {
        self.num_shader_cores = Some(cores);
        self
    }

    pub fn shader_core_mask(mut self, mask: u64) -> Self {
        self.shader_core_mask = Some(mask);
        self
    }

    pub fn num_l2_slices(mut self, slices: u64) -> Self {
        self.num_l2_slices = Some(slices);
        self
    }

    pub fn num_l2_bytes(mut self, bytes: u64) -> Self {
        self.num_l2_bytes = Some(bytes);
        self
    }

    pub fn num_bus_bits(mut self, bits: u64) -> Self {
        self.num_bus_bits = Some(bits);
        self
    }

    pub fn num_exec_engines(mut self, engines: u32) -> Self {
        self.num_exec_engines = Some(engines);
        self
    }

    pub fn num_fp32_fmas_per_core(mut self, fmas: u32) -> Self {
        self.num_fp32_fmas_per_core = Some(fmas);
        self
    }

    pub fn num_fp16_fmas_per_core(mut self, fmas: u32) -> Self {
        self.num_fp16_fmas_per_core = Some(fmas);
        self
    }

    pub fn num_texels_per_core(mut self, texels: u32) -> Self {
        self.num_texels_per_core = Some(texels);
        self
    }

    pub fn num_pixels_per_core(mut self, pixels: u32) -> Self {
        self.num_pixels_per_core = Some(pixels);
        self
    }

    /// Build GpuInfo (Mali-specific builder)
    pub fn build(self) -> Result<GpuInfo, &'static str> {
        let mali_data = MaliData {
            gpu_id: self.gpu_id.ok_or("GPU ID required")?,
            raw_gpu_id: self.raw_gpu_id.ok_or("Raw GPU ID required")?,
            shader_core_mask: self.shader_core_mask.unwrap_or(0),
            num_l2_slices: self.num_l2_slices.unwrap_or(0),
            num_exec_engines: self.num_exec_engines.unwrap_or(0),
            num_fp32_fmas_per_core: self.num_fp32_fmas_per_core.unwrap_or(0),
            num_fp16_fmas_per_core: self.num_fp16_fmas_per_core.unwrap_or(0),
            num_texels_per_core: self.num_texels_per_core.unwrap_or(0),
            num_pixels_per_core: self.num_pixels_per_core.unwrap_or(0),
        };

        Ok(GpuInfo {
            vendor: GpuVendor::Mali,
            gpu_name: self.gpu_name.ok_or("GPU name required")?,
            architecture: self.architecture.ok_or("Architecture required")?,
            architecture_major: self.architecture_major.ok_or("Architecture major required")?,
            architecture_minor: self.architecture_minor.ok_or("Architecture minor required")?,
            num_shader_cores: self.num_shader_cores.ok_or("Number of shader cores required")?,
            num_l2_bytes: self.num_l2_bytes.ok_or("L2 cache size required")?,
            num_bus_bits: self.num_bus_bits.unwrap_or(0),
            mali_data: Some(mali_data),
            adreno_data: None,
        })
    }
}