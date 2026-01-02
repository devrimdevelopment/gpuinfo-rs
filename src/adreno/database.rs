use std::fmt;

/// Adreno GPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrenoArch {
    A4xx,
    A5xx,
    A6xx,
    A7xx,
    A8xx,
}

impl fmt::Display for AdrenoArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdrenoArch::A4xx => write!(f, "Adreno 4xx"),
            AdrenoArch::A5xx => write!(f, "Adreno 5xx"),
            AdrenoArch::A6xx => write!(f, "Adreno 6xx"),
            AdrenoArch::A7xx => write!(f, "Adreno 7xx"),
            AdrenoArch::A8xx => write!(f, "Adreno 8xx"),
        }
    }
}

/// Confidence level of the specifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecConfidence {
    /// Directly measured from known driver-reported chip IDs
    Measured,
    /// Confirmed via reverse engineering or reliable community sources
    ReverseEngineered,
    /// Estimated/heuristic (common for undisclosed modern specs)
    Heuristic,
}

impl fmt::Display for SpecConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecConfidence::Measured => write!(f, "Measured"),
            SpecConfidence::ReverseEngineered => write!(f, "Reverse Engineered"),
            SpecConfidence::Heuristic => write!(f, "Heuristic"),
        }
    }
}

/// Adreno GPU specifications based on chip ID
#[derive(Debug, Clone, Copy)]
pub struct AdrenoSpecs {
    pub name: &'static str,
    pub architecture: AdrenoArch,
    pub shader_cores: u32,         // Shader clusters / pipelines
    pub stream_processors: u32,    // Total ALUs (often speculative on newer GPUs)
    pub gmem_size_kb: u32,         // On-chip GMEM (sometimes estimated)
    pub bus_width_bits: u32,       // Memory bus width
    pub max_freq_mhz: u32,         // Typical/max boost frequency
    pub process_nm: u32,           // Manufacturing process
    pub year: u32,                 // Release year
    pub snapdragon_models: &'static [&'static str],
    pub confidence: SpecConfidence,
}

/// Generic fallback specs (static lifetime)
const GENERIC_LOW_END_5XX: AdrenoSpecs = AdrenoSpecs {
    name: "Adreno 5xx (low-end variant)",
    architecture: AdrenoArch::A5xx,
    shader_cores: 1,
    stream_processors: 96,
    gmem_size_kb: 256,
    bus_width_bits: 32,
    max_freq_mhz: 500,
    process_nm: 28,
    year: 2016,
    snapdragon_models: &["various 4xx/6xx low-end"],
    confidence: SpecConfidence::Heuristic,
};

const GENERIC_7XX: AdrenoSpecs = AdrenoSpecs {
    name: "Adreno 7xx (unknown variant)",
    architecture: AdrenoArch::A7xx,
    shader_cores: 5,
    stream_processors: 1024,
    gmem_size_kb: 3072,
    bus_width_bits: 192,
    max_freq_mhz: 900,
    process_nm: 4,
    year: 2022,
    snapdragon_models: &["8 Gen series"],
    confidence: SpecConfidence::Heuristic,
};

/// Comprehensive Adreno chip database
pub const ADRENO_CHIPS: &[(u32, AdrenoSpecs)] = &[
    // === Adreno 7xx series (2022+) ===
    (0x07030001, AdrenoSpecs {
        name: "Adreno 730",
        architecture: AdrenoArch::A7xx,
        shader_cores: 4,
        stream_processors: 768,
        gmem_size_kb: 2048,
        bus_width_bits: 128,
        max_freq_mhz: 900,
        process_nm: 4,
        year: 2022,
        snapdragon_models: &["8 Gen 1", "8+ Gen 1"],
        confidence: SpecConfidence::Measured,
    }),
    (0x07060001, AdrenoSpecs {
        name: "Adreno 740",
        architecture: AdrenoArch::A7xx,
        shader_cores: 6,
        stream_processors: 1024,
        gmem_size_kb: 3072,
        bus_width_bits: 256,
        max_freq_mhz: 680,
        process_nm: 4,
        year: 2023,
        snapdragon_models: &["8 Gen 2"],
        confidence: SpecConfidence::Measured,
    }),
    (0x07050000, AdrenoSpecs {
        name: "Adreno 750",
        architecture: AdrenoArch::A7xx,
        shader_cores: 6,
        stream_processors: 1536,
        gmem_size_kb: 4096,
        bus_width_bits: 256,
        max_freq_mhz: 1000,
        process_nm: 4,
        year: 2023,
        snapdragon_models: &["8 Gen 3"],
        confidence: SpecConfidence::ReverseEngineered,
    }),

    // === Adreno 6xx series ===
    (0x06010000, AdrenoSpecs {
        name: "Adreno 610",
        architecture: AdrenoArch::A6xx,
        shader_cores: 2,
        stream_processors: 128,
        gmem_size_kb: 384,
        bus_width_bits: 64,
        max_freq_mhz: 950,
        process_nm: 11,
        year: 2019,
        snapdragon_models: &["460", "662", "665"],
        confidence: SpecConfidence::Measured,
    }),
    (0x06010001, AdrenoSpecs {
        name: "Adreno 618",
        architecture: AdrenoArch::A6xx,
        shader_cores: 2,
        stream_processors: 256,
        gmem_size_kb: 512,
        bus_width_bits: 64,
        max_freq_mhz: 825,
        process_nm: 8,
        year: 2019,
        snapdragon_models: &["730", "732G", "735G", "SM7150"],
        confidence: SpecConfidence::Measured,
    }),
    (0x06010500, AdrenoSpecs {
        name: "Adreno 619",
        architecture: AdrenoArch::A6xx,
        shader_cores: 2,
        stream_processors: 256,
        gmem_size_kb: 512,
        bus_width_bits: 64,
        max_freq_mhz: 950,
        process_nm: 8,
        year: 2020,
        snapdragon_models: &["750G", "690", "480"],
        confidence: SpecConfidence::Measured,
    }),
    (0x06010200, AdrenoSpecs {
        name: "Adreno 612/615/616",
        architecture: AdrenoArch::A6xx,
        shader_cores: 2,
        stream_processors: 256,
        gmem_size_kb: 768,
        bus_width_bits: 64,
        max_freq_mhz: 850,
        process_nm: 10,
        year: 2019,
        snapdragon_models: &["670", "675", "710", "712"],
        confidence: SpecConfidence::Heuristic,
    }),
    (0x06020000, AdrenoSpecs {
        name: "Adreno 620",
        architecture: AdrenoArch::A6xx,
        shader_cores: 2,
        stream_processors: 256,
        gmem_size_kb: 768,
        bus_width_bits: 64,
        max_freq_mhz: 750,
        process_nm: 8,
        year: 2020,
        snapdragon_models: &["765", "765G", "768G"],
        confidence: SpecConfidence::ReverseEngineered,
    }),

    // === Adreno 5xx series ===
    (0x05000000, AdrenoSpecs {
        name: "Adreno 504/505",
        architecture: AdrenoArch::A5xx,
        shader_cores: 1,
        stream_processors: 96,
        gmem_size_kb: 256,
        bus_width_bits: 32,
        max_freq_mhz: 450,
        process_nm: 28,
        year: 2016,
        snapdragon_models: &["425", "429", "430", "435", "439"],
        confidence: SpecConfidence::ReverseEngineered,
    }),
    (0x05060000, AdrenoSpecs {
        name: "Adreno 506",
        architecture: AdrenoArch::A5xx,
        shader_cores: 1,
        stream_processors: 128,
        gmem_size_kb: 256,
        bus_width_bits: 32,
        max_freq_mhz: 650,
        process_nm: 14,
        year: 2016,
        snapdragon_models: &["450", "625", "626", "632"],
        confidence: SpecConfidence::Measured,
    }),
    (0x05080000, AdrenoSpecs {
        name: "Adreno 508",
        architecture: AdrenoArch::A5xx,
        shader_cores: 2,
        stream_processors: 128,
        gmem_size_kb: 256,
        bus_width_bits: 64,
        max_freq_mhz: 650,
        process_nm: 14,
        year: 2017,
        snapdragon_models: &["630", "632"],
        confidence: SpecConfidence::ReverseEngineered,
    }),
    (0x05090000, AdrenoSpecs {
        name: "Adreno 509",
        architecture: AdrenoArch::A5xx,
        shader_cores: 2,
        stream_processors: 128,
        gmem_size_kb: 384,
        bus_width_bits: 64,
        max_freq_mhz: 720,
        process_nm: 14,
        year: 2017,
        snapdragon_models: &["636", "638"],
        confidence: SpecConfidence::ReverseEngineered,
    }),
    (0x05120000, AdrenoSpecs {
        name: "Adreno 512",
        architecture: AdrenoArch::A5xx,
        shader_cores: 2,
        stream_processors: 256,
        gmem_size_kb: 512,
        bus_width_bits: 64,
        max_freq_mhz: 850,
        process_nm: 14,
        year: 2017,
        snapdragon_models: &["660", "662"],
        confidence: SpecConfidence::ReverseEngineered,
    }),
    (0x05010000, AdrenoSpecs {
        name: "Adreno 510",
        architecture: AdrenoArch::A5xx,
        shader_cores: 2,
        stream_processors: 128,
        gmem_size_kb: 256,
        bus_width_bits: 32,
        max_freq_mhz: 600,
        process_nm: 14,
        year: 2016,
        snapdragon_models: &["430", "435", "616", "617"],
        confidence: SpecConfidence::Measured,
    }),
    (0x04020000, AdrenoSpecs {
        name: "Adreno 530",
        architecture: AdrenoArch::A5xx,
        shader_cores: 3,
        stream_processors: 256,
        gmem_size_kb: 512,
        bus_width_bits: 64,
        max_freq_mhz: 624,
        process_nm: 14,
        year: 2016,
        snapdragon_models: &["820", "821"],
        confidence: SpecConfidence::Measured,
    }),
    (0x05020000, AdrenoSpecs {
        name: "Adreno 540",
        architecture: AdrenoArch::A5xx,
        shader_cores: 3,
        stream_processors: 256,
        gmem_size_kb: 512,
        bus_width_bits: 64,
        max_freq_mhz: 710,
        process_nm: 10,
        year: 2017,
        snapdragon_models: &["835"],
        confidence: SpecConfidence::Measured,
    }),

    // === Adreno 4xx series ===
    (0x04010000, AdrenoSpecs {
        name: "Adreno 405",
        architecture: AdrenoArch::A4xx,
        shader_cores: 1,
        stream_processors: 48,
        gmem_size_kb: 128,
        bus_width_bits: 32,
        max_freq_mhz: 550,
        process_nm: 28,
        year: 2014,
        snapdragon_models: &["415", "425", "610"],
        confidence: SpecConfidence::Measured,
    }),
];

/// Find GPU specifications by chip ID
pub fn find_adreno_specs(chip_id: u32) -> Option<&'static AdrenoSpecs> {
    // 1. Exact match
    for &(id, ref specs) in ADRENO_CHIPS {
        if id == chip_id {
            return Some(specs);
        }
    }

    // 2. Base ID match (major/minor)
    let base_id = chip_id & 0xFFFF0000;
    for &(id, ref specs) in ADRENO_CHIPS {
        if (id & 0xFFFF0000) == base_id {
            return Some(specs);
        }
    }

    // 3. Generic series fallback
    let major = (chip_id >> 24) & 0xFF;

    match major {
        8 => Some(&AdrenoSpecs {
            name: "Adreno 8xx (unknown variant)",
            architecture: AdrenoArch::A8xx,
            shader_cores: 8,
            stream_processors: 2048,
            gmem_size_kb: 4096,
            bus_width_bits: 384,
            max_freq_mhz: 1100,
            process_nm: 3,
            year: 2024,
            snapdragon_models: &["8 Elite / future"],
            confidence: SpecConfidence::Heuristic,
        }),
        7 => Some(&GENERIC_7XX),
        6 => Some(&AdrenoSpecs {
            name: "Adreno 6xx (unknown low/mid variant)",
            architecture: AdrenoArch::A6xx,
            shader_cores: 2,
            stream_processors: 256,
            gmem_size_kb: 512,
            bus_width_bits: 64,
            max_freq_mhz: 800,
            process_nm: 8,
            year: 2019,
            snapdragon_models: &["various 4xx/6xx/7xx low-end"],
            confidence: SpecConfidence::Heuristic,
        }),
        5 => Some(&GENERIC_LOW_END_5XX),
        4 => Some(&AdrenoSpecs {
            name: "Adreno 4xx (unknown variant)",
            architecture: AdrenoArch::A4xx,
            shader_cores: 1,
            stream_processors: 48,
            gmem_size_kb: 128,
            bus_width_bits: 32,
            max_freq_mhz: 550,
            process_nm: 28,
            year: 2014,
            snapdragon_models: &["various 2xx/4xx low-end"],
            confidence: SpecConfidence::Heuristic,
        }),
        _ => None,
    }
}