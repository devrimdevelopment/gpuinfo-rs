//! Parser for KGSL property structures
//! Consistent with Mali parser.rs architecture

use crate::error::{GpuError, GpuResult};

/// KGSL Property IDs (from kernel headers)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KgslPropertyId {
    DeviceInfo = 0x1,
    ChipId = 0x2,
    GmemInfo = 0x3,
    GpuModel = 0x4,
    // Add more as needed from kernel headers
}

impl TryFrom<u32> for KgslPropertyId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x1 => Ok(KgslPropertyId::DeviceInfo),
            0x2 => Ok(KgslPropertyId::ChipId),
            0x3 => Ok(KgslPropertyId::GmemInfo),
            0x4 => Ok(KgslPropertyId::GpuModel),
            _ => Err(()),
        }
    }
}

/// Parser configuration for different modes
#[derive(Debug, Clone, Copy)]
pub struct ParserConfig {
    /// Skip unknown properties instead of erroring
    pub lenient_mode: bool,
    /// Validate chip ID format and ranges
    pub validate_chip_id: bool,
    /// Require all mandatory properties
    pub require_mandatory: bool,
    /// Allow zero values for certain properties
    pub allow_zero_values: bool,
}

impl ParserConfig {
    /// Configuration for Parity mode (matches existing behavior)
    pub const PARITY: Self = Self {
        lenient_mode: true,
        validate_chip_id: false,
        require_mandatory: false,
        allow_zero_values: true,
    };

    /// Configuration for Extended mode (strict validation)
    pub const EXTENDED: Self = Self {
        lenient_mode: false,
        validate_chip_id: true,
        require_mandatory: true,
        allow_zero_values: false,
    };
}

/// KGSL Device Info structure with parsed fields
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ParsedDeviceInfo {
    /// Raw device ID from driver
    pub device_id: u32,
    /// Chip ID (includes architecture, generation, revision)
    pub chip_id: u32,
    /// MMU enabled flag
    pub mmu_enabled: bool,
    /// GPU memory base address
    pub gmem_baseaddr: u32,
    /// GPU memory size in bytes
    pub gmem_sizebytes: u32,
    /// GPU model code
    pub gpu_model: u32,
    
    // Additional validated fields
    /// Architecture major version (extracted from chip_id)
    pub arch_major: u8,
    /// Architecture minor version
    pub arch_minor: u8,
    /// Generation (Adreno 6xx, 7xx, etc.)
    pub generation: u8,
    /// Revision
    pub revision: u8,
}

impl ParsedDeviceInfo {
    /// Extract architecture from chip ID
    /// Format: 0xAABBCCDD where:
    ///   AA = architecture major
    ///   BB = architecture minor  
    ///   CC = generation
    ///   DD = revision
    pub fn extract_architecture(&mut self) -> Result<(), GpuError> {
        self.arch_major = ((self.chip_id >> 24) & 0xFF) as u8;
        self.arch_minor = ((self.chip_id >> 16) & 0xFF) as u8;
        self.generation = ((self.chip_id >> 8) & 0xFF) as u8;
        self.revision = (self.chip_id & 0xFF) as u8;
        
        Ok(())
    }
    
    /// Validate chip ID structure
    pub fn validate_chip_id(&self) -> GpuResult<()> {
        // Basic validation rules
        if self.chip_id == 0 {
            return Err(GpuError::InvalidData("Chip ID is zero".into()));
        }
        
        // Check reasonable ranges
        let major = self.arch_major;
        if major < 6 || major > 9 {  // Adreno 6xx-9xx range
            return Err(GpuError::UnsupportedArchitecture {
                chip_id: self.chip_id,
                architecture: format!("Adreno {major}xx"),
            });
        }
        
        Ok(())
    }
}

/// Unified parser for KGSL properties
pub struct KgslPropertyParser<'a> {
    /// Raw property buffer
    buffer: &'a [u8],
    /// Current position in buffer
    pos: usize,
    /// Parser configuration
    config: ParserConfig,
    /// Parsed device info (output)
    pub info: ParsedDeviceInfo,
}

impl<'a> KgslPropertyParser<'a> {
    /// Create a new parser
    pub fn new(buffer: &'a [u8], config: ParserConfig) -> Self {
        Self {
            buffer,
            pos: 0,
            config,
            info: ParsedDeviceInfo::default(),
        }
    }
    
    /// Parse device info structure directly (for DEVICE_INFO property)
    pub fn parse_device_info(mut self) -> GpuResult<ParsedDeviceInfo> {
        // KGSL_DEVICE_INFO is a fixed structure
        if self.buffer.len() < std::mem::size_of::<RawDeviceInfo>() {
            if self.config.lenient_mode {
                return Ok(ParsedDeviceInfo::default());
            } else {
                return Err(GpuError::BufferTooSmall {
                    expected: std::mem::size_of::<RawDeviceInfo>(),
                    actual: self.buffer.len(),
                });
            }
        }
        
        // Parse raw structure (little-endian)
        let raw = self.parse_raw_device_info()?;
        
        // Populate parsed info
        let mut info = ParsedDeviceInfo {
            device_id: raw.device_id,
            chip_id: raw.chip_id,
            mmu_enabled: raw.mmu_enabled != 0,
            gmem_baseaddr: raw.gmem_gpubaseaddr,
            gmem_sizebytes: raw.gmem_sizebytes,
            gpu_model: raw.gpu_model,
            ..Default::default()
        };
        
        // Extract architecture
        info.extract_architecture()?;
        
        // Validate if configured
        if self.config.validate_chip_id {
            info.validate_chip_id()?;
        }
        
        // Check mandatory fields
        if self.config.require_mandatory {
            if info.chip_id == 0 {
                return Err(GpuError::InvalidData("Missing mandatory chip ID".into()));
            }
            if info.gmem_sizebytes == 0 && !self.config.allow_zero_values {
                return Err(GpuError::InvalidData("GPU memory size is zero".into()));
            }
        }
        
        Ok(info)
    }
    
    /// Parse raw device info structure from buffer
    fn parse_raw_device_info(&mut self) -> GpuResult<RawDeviceInfo> {
        let mut raw = RawDeviceInfo::default();
        
        // Parse each field (little-endian)
        raw.device_id = self.read_u32()?;
        raw.chip_id = self.read_u32()?;
        raw.mmu_enabled = self.read_u32()?;
        raw.gmem_gpubaseaddr = self.read_u32()?;
        raw.gmem_sizebytes = self.read_u32()?;
        
        // Skip unknown fields if present
        let remaining = self.buffer.len() - self.pos;
        if remaining >= 8 {
            raw.unknown1 = self.read_u32()?;
            raw.unknown2 = self.read_u32()?;
        }
        
        if remaining >= 12 {
            raw.gpu_model = self.read_u32()?;
        }
        
        Ok(raw)
    }
    
    /// Read u32 from buffer (little-endian)
    fn read_u32(&mut self) -> GpuResult<u32> {
        if self.pos + 4 > self.buffer.len() {
            if self.config.lenient_mode {
                return Ok(0);
            } else {
                return Err(GpuError::BufferTooSmall {
                    expected: self.pos + 4,
                    actual: self.buffer.len(),
                });
            }
        }
        
        let bytes = &self.buffer[self.pos..self.pos + 4];
        self.pos += 4;
        
        Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| {
            GpuError::InvalidData("Failed to parse u32 from buffer".into())
        })?))
    }
}

// Raw structure matching kernel's kgsl_device_info
#[repr(C)]
#[derive(Debug, Clone, Default)]
struct RawDeviceInfo {
    device_id: u32,
    chip_id: u32,
    mmu_enabled: u32,
    gmem_gpubaseaddr: u32,
    gmem_sizebytes: u32,
    unknown1: u32,
    unknown2: u32,
    gpu_model: u32,
}

/// Parse KGSL device info buffer with configuration
pub fn parse_device_info(buffer: &[u8], config: ParserConfig) -> GpuResult<ParsedDeviceInfo> {
    let parser = KgslPropertyParser::new(buffer, config);
    parser.parse_device_info()
}

/// Parse KGSL device info buffer (Extended mode - strict with validation)
pub fn parse_device_info_strict(buffer: &[u8]) -> GpuResult<ParsedDeviceInfo> {
    parse_device_info(buffer, ParserConfig::EXTENDED)
}

/// Parse KGSL device info buffer (Parity mode - lenient, matches existing behavior)
pub fn parse_device_info_lenient(buffer: &[u8]) -> ParsedDeviceInfo {
    match parse_device_info(buffer, ParserConfig::PARITY) {
        Ok(info) => info,
        Err(_) => ParsedDeviceInfo::default(),
    }
}