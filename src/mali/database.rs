use std::collections::HashMap;
use std::sync::OnceLock;

// Product database structures
pub struct ProductEntry {
    pub id: u32,
    pub mask: u32,
    pub min_cores: u32,
    pub name: &'static str,
    pub architecture: &'static str,
    pub get_num_fp32_fmas_per_engine: fn(u32, u32, u32) -> u32,
    pub get_num_texels: fn(u32, u32, u32) -> u32,
    pub get_num_pixels: fn(u32, u32, u32) -> u32,
    pub get_num_exec_engines: fn(u32, u32, u32) -> u32,
}

const MASK_OLD: u32 = 0xFFFF;
const MASK_NEW: u32 = 0xF00F;

// Helper functions for product database
pub fn get_num_1(_: u32, _: u32, _: u32) -> u32 { 1 }
pub fn get_num_2(_: u32, _: u32, _: u32) -> u32 { 2 }
pub fn get_num_3(_: u32, _: u32, _: u32) -> u32 { 3 }
pub fn get_num_4(_: u32, _: u32, _: u32) -> u32 { 4 }
pub fn get_num_8(_: u32, _: u32, _: u32) -> u32 { 8 }
pub fn get_num_16(_: u32, _: u32, _: u32) -> u32 { 16 }
pub fn get_num_32(_: u32, _: u32, _: u32) -> u32 { 32 }
pub fn get_num_64(_: u32, _: u32, _: u32) -> u32 { 64 }

pub fn get_num_eng_g31(core_count: u32, _: u32, thread_features: u32) -> u32 {
    if core_count == 1 && (thread_features & 0xFFFF) == 0x2000 { 1 } else { 2 }
}

pub fn get_num_eng_g51(core_count: u32, _: u32, thread_features: u32) -> u32 {
    if core_count == 1 && (thread_features & 0xFFFF) == 0x2000 { 1 } else { 3 }
}

pub fn get_num_eng_g52(_: u32, core_features: u32, _: u32) -> u32 { core_features & 0xF }

pub fn get_num_fma_g510(_: u32, core_features: u32, _: u32) -> u32 {
    let variant = core_features & 0xF;
    match variant { 0 => 16, 2 | 3 => 24, _ => 32 }
}

pub fn get_num_tex_g510(_: u32, core_features: u32, _: u32) -> u32 {
    let variant = core_features & 0xF;
    match variant { 0 | 5 => 2, 1 | 2 | 6 => 4, _ => 8 }
}

pub fn get_num_pix_g510(_: u32, core_features: u32, _: u32) -> u32 {
    let variant = core_features & 0xF;
    match variant { 0 | 1 | 5 | 6 => 2, _ => 4 }
}

pub fn get_num_eng_g510(_: u32, core_features: u32, _: u32) -> u32 {
    let variant = core_features & 0xF;
    match variant { 0 | 1 | 5 | 6 => 1, _ => 2 }
}

const PRODUCT_VERSIONS: [ProductEntry; 38] = [
    // Mali-T600 series
    ProductEntry {
        id: 0x6956,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T600",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x0620,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T620",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x0720,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T720",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_1,
    },
    ProductEntry {
        id: 0x0750,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T760",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x0820,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T820",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_1,
    },
    ProductEntry {
        id: 0x0830,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T830",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x0860,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T860",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x0880,
        mask: MASK_OLD,
        min_cores: 1,
        name: "Mali-T880",
        architecture: "Midgard",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_3,
    },

    // Mali-G71/G72 (Bifrost)
    ProductEntry {
        id: 0x6000,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G71",
        architecture: "Bifrost",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_3,
    },
    ProductEntry {
        id: 0x6001,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G72",
        architecture: "Bifrost",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_1,
        get_num_pixels: get_num_1,
        get_num_exec_engines: get_num_3,
    },

    // Mali-G51/G76/G52/G31 (Bifrost)
    ProductEntry {
        id: 0x7000,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G51",
        architecture: "Bifrost",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_2,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_eng_g51,
    },
    ProductEntry {
        id: 0x7001,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G76",
        architecture: "Bifrost",
        get_num_fp32_fmas_per_engine: get_num_8,
        get_num_texels: get_num_2,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_3,
    },
    ProductEntry {
        id: 0x7002,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G52",
        architecture: "Bifrost",
        get_num_fp32_fmas_per_engine: get_num_8,
        get_num_texels: get_num_2,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_eng_g52,
    },
    ProductEntry {
        id: 0x7003,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G31",
        architecture: "Bifrost",
        get_num_fp32_fmas_per_engine: get_num_4,
        get_num_texels: get_num_2,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_eng_g31,
    },

    // Mali-G77/G57/G68/G78 (Valhall)
    ProductEntry {
        id: 0x9000,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G77",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_16,
        get_num_texels: get_num_4,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x9001,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G57",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_16,
        get_num_texels: get_num_4,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x9003,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G57",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_16,
        get_num_texels: get_num_4,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x9004,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G68",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_16,
        get_num_texels: get_num_4,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x9002,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G78",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_16,
        get_num_texels: get_num_4,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0x9005,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G78AE",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_16,
        get_num_texels: get_num_4,
        get_num_pixels: get_num_2,
        get_num_exec_engines: get_num_2,
    },

    // Mali-G710/G610 (Valhall)
    ProductEntry {
        id: 0xa002,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G710",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_32,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xa007,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G610",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_32,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },

    // Mali-G510/G310 (Valhall)
    ProductEntry {
        id: 0xa003,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G510",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_fma_g510,
        get_num_texels: get_num_tex_g510,
        get_num_pixels: get_num_pix_g510,
        get_num_exec_engines: get_num_eng_g510,
    },
    ProductEntry {
        id: 0xa004,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G310",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_fma_g510,
        get_num_texels: get_num_tex_g510,
        get_num_pixels: get_num_pix_g510,
        get_num_exec_engines: get_num_eng_g510,
    },

    // Immortalis-G715/Mali-G715/G615
    ProductEntry {
        id: 0xb002,
        mask: MASK_NEW,
        min_cores: 10,
        name: "Immortalis-G715",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xb002,
        mask: MASK_NEW,
        min_cores: 7,
        name: "Mali-G715",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xb002,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G615",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xb003,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G615",
        architecture: "Valhall",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },

    // Immortalis-G720/Mali-G720/G620
    ProductEntry {
        id: 0xc000,
        mask: MASK_NEW,
        min_cores: 10,
        name: "Immortalis-G720",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xc000,
        mask: MASK_NEW,
        min_cores: 6,
        name: "Mali-G720",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xc000,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G620",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xc001,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G620",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },

    // Immortalis-G925/Mali-G725/G625
    ProductEntry {
        id: 0xd000,
        mask: MASK_NEW,
        min_cores: 10,
        name: "Immortalis-G925",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xd000,
        mask: MASK_NEW,
        min_cores: 6,
        name: "Mali-G725",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xd001,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali-G625",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },

    // Mali G1 series
    ProductEntry {
        id: 0xe000,
        mask: MASK_NEW,
        min_cores: 10,
        name: "Mali G1-Ultra",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xe001,
        mask: MASK_NEW,
        min_cores: 6,
        name: "Mali G1-Premium",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
    ProductEntry {
        id: 0xe003,
        mask: MASK_NEW,
        min_cores: 1,
        name: "Mali G1-Pro",
        architecture: "Arm 5th Gen",
        get_num_fp32_fmas_per_engine: get_num_64,
        get_num_texels: get_num_8,
        get_num_pixels: get_num_4,
        get_num_exec_engines: get_num_2,
    },
];

// Lazy-initialized product lookup map
fn product_map() -> &'static HashMap<u32, Vec<&'static ProductEntry>> {
    static MAP: OnceLock<HashMap<u32, Vec<&'static ProductEntry>>> = OnceLock::new();

    MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for entry in &PRODUCT_VERSIONS {
            map.entry(entry.id).or_insert_with(Vec::new).push(entry);
        }
        map
    })
}

pub(crate) fn get_gpu_id(input_id: u32) -> u32 {
    for entry in PRODUCT_VERSIONS.iter() {
        if (input_id & entry.mask) == entry.id {
            return entry.id;
        }
    }
    input_id
}

pub(crate) fn lookup_product(gpu_id: u32, core_count: u32) -> Option<&'static ProductEntry> {
    product_map()
        .get(&gpu_id)?
        .iter()
        .filter(|e| core_count >= e.min_cores)
        .max_by_key(|e| e.min_cores)
        .copied()
}

pub(crate) fn extract_architecture(raw_gpu_id: u64) -> (u8, u8) {
    const COMPAT_SHIFT: u64 = 28;
    const COMPAT_MASK: u64 = 0xF;

    let is_64bit_id = ((raw_gpu_id >> COMPAT_SHIFT) & COMPAT_MASK) == COMPAT_MASK;

    if !is_64bit_id {
        (
            ((raw_gpu_id >> 28) & 0xF) as u8,
            ((raw_gpu_id >> 24) & 0xF) as u8,
        )
    } else {
        (
            ((raw_gpu_id >> 56) & 0xFF) as u8,
            ((raw_gpu_id >> 48) & 0xFF) as u8,
        )
    }
}

pub(crate) fn validate_gpu_info(info: &crate::info::GpuInfo) -> crate::error::GpuResult<()> {
    if info.num_shader_cores == 0 {
        return Err(crate::error::GpuError::InvalidData("GPU has zero shader cores".into()));
    }

    if info.num_l2_bytes == 0 {
        return Err(crate::error::GpuError::InvalidData("GPU has zero L2 cache".into()));
    }

    Ok(())
}