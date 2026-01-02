use crate::error::{GpuError, GpuResult};

/// Property IDs used in Mali property buffer (from kbase_gpuprops.h)
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropId {
    ProductId = 1,
    L2Log2CacheSize = 14,
    L2NumL2Slices = 15,
    RawL2Features = 29,
    RawCoreFeatures = 30,
    RawGpuId = 55,
    RawThreadFeatures = 59,
    CoherencyNumCoreGroups = 62,
}

impl TryFrom<u64> for PropId {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PropId::ProductId),
            14 => Ok(PropId::L2Log2CacheSize),
            15 => Ok(PropId::L2NumL2Slices),
            29 => Ok(PropId::RawL2Features),
            30 => Ok(PropId::RawCoreFeatures),
            55 => Ok(PropId::RawGpuId),
            59 => Ok(PropId::RawThreadFeatures),
            62 => Ok(PropId::CoherencyNumCoreGroups),
            _ => Err(()),
        }
    }
}

/// Parser configuration for different modes
#[derive(Debug, Clone, Copy)]
pub struct ParserConfig {
    /// Skip invalid properties instead of erroring
    pub lenient_mode: bool,
    /// Validate that core group masks are within bounds
    pub validate_group_bounds: bool,
    /// Accept core masks even when num_core_groups is zero
    pub accept_masks_without_groups: bool,
    /// Skip out-of-bounds core masks instead of ignoring them
    pub skip_out_of_bounds_masks: bool,
}

impl ParserConfig {
    /// Configuration for Parity mode (matches libgpuinfo exactly)
    pub const PARITY: Self = Self {
        lenient_mode: true,
        validate_group_bounds: false,
        accept_masks_without_groups: true,
        skip_out_of_bounds_masks: false,
    };

    /// Configuration for Extended mode (strict validation)
    pub const EXTENDED: Self = Self {
        lenient_mode: false,
        validate_group_bounds: true,
        accept_masks_without_groups: false,
        skip_out_of_bounds_masks: true,
    };
}

/// Parsed GPU properties from driver
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ParsedProperties {
    /// GPU product ID from driver
    pub gpu_id: u32,
    /// Log2 of L2 cache size per slice
    pub l2_log2_cache_size: u64,
    /// Number of L2 cache slices
    pub num_l2_slices: u64,
    /// Raw L2 features register value
    pub raw_l2_features: u64,
    /// Raw core features register value
    pub raw_core_features: u32,
    /// Raw GPU ID register value
    pub raw_gpu_id: u64,
    /// Raw thread features register value
    pub raw_thread_features: u32,
    /// Number of shader cores (calculated from mask)
    pub num_shader_cores: u32,
    /// Bitmask of available shader cores
    pub shader_core_mask: u64,
}

impl ParsedProperties {
    /// Create an empty ParsedProperties struct
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Unified parser for Mali property buffer
struct UnifiedPropParser<'a> {
    data: &'a [u8],
    pos: usize,
    config: ParserConfig,
}

impl<'a> UnifiedPropParser<'a> {
    /// Create a new parser for the given buffer with configuration
    fn new(data: &'a [u8], config: ParserConfig) -> Self {
        Self {
            data,
            pos: 0,
            config,
        }
    }

    /// Parse the entire buffer into properties
    fn parse(mut self) -> GpuResult<ParsedProperties> {
        let mut props = ParsedProperties::default();
        let mut num_core_groups = 0;
        let mut core_masks_received = 0;

        while let Some((prop_id, value)) = self.next_prop()? {
            match PropId::try_from(prop_id) {
                Ok(PropId::ProductId) => props.gpu_id = value as u32,
                Ok(PropId::L2Log2CacheSize) => props.l2_log2_cache_size = value,
                Ok(PropId::L2NumL2Slices) => props.num_l2_slices = value,
                Ok(PropId::RawL2Features) => props.raw_l2_features = value,
                Ok(PropId::RawCoreFeatures) => props.raw_core_features = value as u32,
                Ok(PropId::RawGpuId) => props.raw_gpu_id = value,
                Ok(PropId::RawThreadFeatures) => props.raw_thread_features = value as u32,
                Ok(PropId::CoherencyNumCoreGroups) => num_core_groups = value,
                Err(_) => {
                    // Handle core group masks (IDs 64-79) for Midgard/Bifrost
                    if (64..=79).contains(&prop_id) {
                        self.handle_core_mask(
                            prop_id,
                            value,
                            num_core_groups,
                            &mut props,
                            &mut core_masks_received,
                        )?;
                    }
                }
            }
        }

        props.num_shader_cores = props.shader_core_mask.count_ones() as u32;

        Ok(props)
    }

    /// Handle core group mask based on configuration
    fn handle_core_mask(
        &self,
        prop_id: u64,
        value: u64,
        num_core_groups: u64,
        props: &mut ParsedProperties,
        core_masks_received: &mut u64,
    ) -> GpuResult<()> {
        let group_idx = prop_id - 64;

        // Check if this mask should be accepted based on configuration
        let should_accept = if num_core_groups == 0 {
            // No core groups defined
            self.config.accept_masks_without_groups
        } else if group_idx < num_core_groups {
            // Valid mask within bounds
            true
        } else {
            // Out of bounds mask
            !self.config.skip_out_of_bounds_masks
        };

        if should_accept {
            props.shader_core_mask |= value;
            if num_core_groups > 0 && group_idx < num_core_groups {
                *core_masks_received += 1;
            }
        } else if self.config.validate_group_bounds {
            // In Extended mode, we note but don't error on out-of-bounds masks
        }

        Ok(())
    }

    /// Get next property from buffer
    fn next_prop(&mut self) -> GpuResult<Option<(u64, u64)>> {
        if self.pos + 4 > self.data.len() {
            return Ok(None);
        }

        // Read key (4 bytes, little-endian)
        let key_bytes = self.read_bytes(4)?;
        let key = u32::from_le_bytes(key_bytes.try_into().map_err(|_| {
            GpuError::InvalidData("Failed to parse property key".into())
        })?);

        // Extract property ID and size
        let prop_id = (key >> 2) as u64;
        let prop_size = key & 3;

        // Determine value size
        let value_size = match prop_size {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => {
                if self.config.lenient_mode {
                    return Ok(None); // Skip invalid size in lenient mode
                } else {
                    return Err(GpuError::InvalidPropertySize(prop_size));
                }
            }
        };

        // Read value (little-endian)
        let value = self.read_value(value_size, prop_size)?;

        Ok(Some((prop_id, value)))
    }

    /// Read bytes from buffer at current position
    fn read_bytes(&mut self, size: usize) -> GpuResult<&[u8]> {
        if self.pos + size > self.data.len() {
            if self.config.lenient_mode {
                // Return empty slice in lenient mode to trigger graceful failure
                self.pos = self.data.len(); // Skip to end
                return Ok(&[]);
            } else {
                return Err(GpuError::BufferTooSmall {
                    expected: self.pos + size,
                    actual: self.data.len(),
                });
            }
        }

        let slice = &self.data[self.pos..self.pos + size];
        self.pos += size;
        Ok(slice)
    }

    /// Read a value of the specified size
    fn read_value(&mut self, size: usize, prop_size: u32) -> GpuResult<u64> {
        let bytes = self.read_bytes(size)?;

        // If bytes is empty (lenient mode hit buffer end), return 0
        if bytes.is_empty() {
            return Ok(0);
        }

        match prop_size {
            0 => Ok(bytes[0] as u64),
            1 => Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| {
                GpuError::InvalidData("Failed to parse u16 property".into())
            })?) as u64),
            2 => Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| {
                GpuError::InvalidData("Failed to parse u32 property".into())
            })?) as u64),
            3 => Ok(u64::from_le_bytes(bytes.try_into().map_err(|_| {
                GpuError::InvalidData("Failed to parse u64 property".into())
            })?)),
            _ => {
                if self.config.lenient_mode {
                    Ok(0)
                } else {
                    Err(GpuError::InvalidPropertySize(prop_size))
                }
            }
        }
    }
}

/// Parse properties buffer into structured data with configuration
pub fn parse_properties(buffer: &[u8], config: ParserConfig) -> GpuResult<ParsedProperties> {
    let parser = UnifiedPropParser::new(buffer, config);
    parser.parse()
}

/// Parse properties buffer into structured data (Extended mode - strict with validation)
pub fn parse_properties_strict(buffer: &[u8]) -> GpuResult<ParsedProperties> {
    parse_properties(buffer, ParserConfig::EXTENDED)
}

/// Parse properties buffer into structured data (Parity mode - lenient, matches libgpuinfo)
pub fn parse_properties_lenient(buffer: &[u8]) -> ParsedProperties {
    match parse_properties(buffer, ParserConfig::PARITY) {
        Ok(props) => props,
        Err(_) => ParsedProperties::empty(),
    }
}