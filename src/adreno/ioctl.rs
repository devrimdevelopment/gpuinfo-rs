//! Typisierte KGSL ioctl-Strukturen und Funktionen
//! 
use std::os::unix::io::RawFd;

use crate::error::{GpuError, GpuResult};

/// KGSL Property Types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KgslPropertyType {
    DeviceInfo = 0x1,
}

/// KGSL Device Get Property ioctl structure
#[repr(C)]
pub struct KgslDeviceGetProperty {
    pub type_: u32,
    pub value: *mut std::ffi::c_void,
    pub sizebytes: u32,
}

/// KGSL Device Info structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KgslDeviceInfo {
    pub device_id: u32,
    pub chip_id: u32,
    pub mmu_enabled: u32,
    pub gmem_gpubaseaddr: u32,
    pub gmem_sizebytes: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub gpu_model: u32,
}

impl Default for KgslDeviceInfo {
    fn default() -> Self {
        Self {
            device_id: 0,
            chip_id: 0,
            mmu_enabled: 0,
            gmem_gpubaseaddr: 0,
            gmem_sizebytes: 0,
            unknown1: 0,
            unknown2: 0,
            gpu_model: 0,
        }
    }
}