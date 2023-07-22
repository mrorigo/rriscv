use elfloader::VAddr;

use crate::{memory::MemoryOperations, mmio::VirtualDevice, mmu::MemoryRange};

pub struct VirtioBlockDisk {
    used_ring_index: u16,
    clock: u64,
    device_features: u64,      // read only
    device_features_sel: u32,  // write only
    driver_features: u32,      // write only
    _driver_features_sel: u32, // write only
    guest_page_size: u32,      // write only
    queue_select: u32,         // write only
    queue_size: u32,           // write only
    queue_align: u32,          // write only
    queue_pfn: u32,            // read and write
    queue_notify: u32,         // write only
    interrupt_status: u32,     // read only
    status: u32,               // read and write
    notify_clocks: Vec<u64>,
    contents: Vec<u64>,
}

pub struct VIRTIO {
    range: MemoryRange,
    device: VirtioBlockDisk,
}

impl VIRTIO {
    pub fn create(range: MemoryRange) -> VIRTIO {
        VIRTIO {
            range,
            device: VirtioBlockDisk {
                used_ring_index: 0,
                clock: 0,
                device_features: 0,
                device_features_sel: 0,
                driver_features: 0,
                _driver_features_sel: 0,
                guest_page_size: 0,
                queue_select: 0,
                queue_size: 0,
                queue_align: 0x1000, // xv6 seems to expect this default value
                queue_pfn: 0,
                queue_notify: 0,
                status: 0,
                interrupt_status: 0,
                notify_clocks: Vec::new(),
                contents: vec![],
            },
        }
    }
}

impl VirtualDevice for VIRTIO {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
    fn write(&mut self, _addr: VAddr, _value: u8) -> bool {
        todo!()
    }

    fn read(&self, _addr: VAddr) -> u8 {
        todo!()
    }
}

impl MemoryOperations<VIRTIO, u8> for VIRTIO {
    fn read8(&self, addr: VAddr) -> Option<u8> {
        todo!("virtio.read8(addr)");
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> bool {
        todo!("virtio.write8(addr, value)");
    }

    fn read32(&self, addr: VAddr) -> Option<u32> {
        todo!("virtio.read32(addr)");
    }

    fn write32(&mut self, addr: VAddr, value: u32) {
        todo!("virtio.write32(addr, value)");
    }
}
