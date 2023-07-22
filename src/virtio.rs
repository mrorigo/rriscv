use std::char::MAX;

use elfloader::{PAddr, VAddr};

use crate::{
    cpu::TrapCause,
    memory::{MemoryOperations, RAM},
    mmio::VirtualDevice,
    mmu::MemoryRange,
};

#[derive(FromPrimitive)]
#[repr(usize)]
pub enum VirtioRegister {
    MagicValue = 0x0,
    Version = 0x4,
    DeviceId = 0x8,
    VendorId = 0xc,
    DeviceFeatures = 0x10,
    DeviceFeaturesSel = 0x14,
    DriverFeatures = 0x20,
    DriverFeaturesSel = 0x24,
    QueueSel = 0x30,
    QueueMax = 0x34,
    QueueNum = 0x38,
    QueueReady = 0x44,
    QueueNotify = 0x50,
    InterruptStatus = 0x60,
    InterruptACK = 0x64,
    Status = 0x70,
    QueueDescLow = 0x80,
    QueueDescHigh = 0x84,
    QueueDriverLow = 0x90,
    QueueDriverHigh = 0x94,
    QueueDeviceLow = 0xa0,
    QueueDeviceHigh = 0xa4,
    ConfigGeneration = 0x0fc,
    Config = 0x100,
}
// 0x2000 is an arbitary number.
const MAX_QUEUE_SIZE: u32 = 0x2000;

// To simulate disk access time.
// @TODO: Set more proper number. 500 core clocks may be too short.
const DISK_ACCESS_DELAY: u64 = 500;

const VIRTQ_DESC_F_NEXT: u16 = 1;

// 0: buffer is write-only = read from disk operation
// 1: buffer is read-only = write to disk operation
const VIRTQ_DESC_F_WRITE: u16 = 2;

const SECTOR_SIZE: u64 = 512;

pub struct MemoryWrapper {
    memory: RAM,
}

impl MemoryWrapper {
    fn new(base_address: PAddr, size: usize) -> Self {
        MemoryWrapper {
            memory: RAM::create(base_address, size),
        }
    }

    pub fn read8(&mut self, p_address: u64) -> u8 {
        self.memory.read8(p_address).unwrap()
    }

    pub fn read16(&mut self, p_address: u64) -> u16 {
        self.memory.read16(p_address).unwrap()
    }

    pub fn read32(&mut self, p_address: u64) -> u32 {
        self.memory.read32(p_address).unwrap()
        // (self.read16(p_address) as u32) << 16 | self.read16(p_address + 2) as u32
    }

    pub fn read64(&mut self, p_address: u64) -> u64 {
        self.memory.read64(p_address).unwrap()
        // (self.read32(p_address) as u64) << 32 | self.read32(p_address + 4) as u64
    }

    pub fn write8(&mut self, p_address: u64, value: u8) {
        self.memory.write8(p_address, value);
    }

    pub fn write16(&mut self, p_address: u64, value: u16) {
        self.memory.write16(p_address, value);
    }

    pub fn write32(&mut self, p_address: u64, value: u32) -> Option<TrapCause> {
        self.memory.write32(p_address, value)
    }

    pub fn write64(&mut self, p_address: u64, value: u64) -> Option<TrapCause> {
        self.memory.write64(p_address, value)
    }

    pub fn validate_address(&self, address: u64) -> bool {
        self.memory.base_address <= address
            && self
                .memory
                .base_address
                .wrapping_add(self.memory.size as u64)
                > address
    }
}

pub struct VirtioBlockDisk {
    used_ring_index: u16,
    clock: u64,
    notifications: Vec<u64>,
    interrupt_status: u32,
    registers: Vec<u32>,
    contents: Vec<u64>,
}

pub struct VIRTIO {
    range: MemoryRange,
    memory: MemoryWrapper,
    device: VirtioBlockDisk,
}

impl VIRTIO {
    pub fn create(range: MemoryRange) -> VIRTIO {
        let device = VirtioBlockDisk::new();
        let memory = MemoryWrapper::new(range.start, (range.end - range.start) as usize);
        VIRTIO {
            range,
            device,
            memory,
        }
    }

    pub fn is_interrupting(&self) -> bool {
        self.device.is_interrupting()
    }

    pub fn load_fs(&mut self, contents: Vec<u8>) {
        self.device.init(contents);
    }

    pub fn tick(&mut self) {
        self.device.tick(&mut self.memory);
    }
}

impl VirtualDevice for VIRTIO {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
    fn write(&mut self, _addr: VAddr, _value: u8) -> Option<TrapCause> {
        todo!()
    }

    fn read(&mut self, _addr: VAddr) -> Result<u8, TrapCause> {
        todo!()
    }
}

impl MemoryOperations<VIRTIO, u8> for VIRTIO {
    fn read8(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        todo!("virtio.read8(addr)");
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        todo!("virtio.write8(addr, value)");
    }

    fn read32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        let val = self.device.load(addr);
        println!("VIRTIO READ32 {:#x?} => {:#x?}", addr, val);
        Ok(val)
    }

    fn write32(&mut self, addr: VAddr, value: u32) -> Option<TrapCause> {
        println!("VIRTIO WRITE32 {:#x?} => {:#x?}", addr, value);
        // todo!("virtio.write32(addr, value)");
        self.device.store(addr, value);
        None
    }

    fn read64(&mut self, _addr: VAddr) -> Result<u64, TrapCause> {
        todo!()
    }

    fn write64(&mut self, _addr: VAddr, _value: u64) -> Option<TrapCause> {
        todo!()
    }

    fn read16(&mut self, _addr: VAddr) -> Option<u16> {
        todo!()
    }

    fn write16(&mut self, _addr: VAddr, _value: u16) -> Option<TrapCause> {
        todo!()
    }
}

// Mostly ripped from https://github.com/takahirox/riscv-rust/blob/master/src/device/virtio_block_disk.rs
impl VirtioBlockDisk {
    /// Creates a new `VirtioBlockDisk`.
    pub fn new() -> Self {
        let mut reg_vec = vec![0; 0x1000];
        reg_vec[VirtioRegister::MagicValue as usize] = 0x74726976;
        reg_vec[VirtioRegister::Version as usize] = 0x2;
        reg_vec[VirtioRegister::DeviceId as usize] = 0x2; // 2 = Block device
        reg_vec[VirtioRegister::VendorId as usize] = 0x554d4551;
        reg_vec[VirtioRegister::QueueMax as usize] = MAX_QUEUE_SIZE;

        VirtioBlockDisk {
            used_ring_index: 0,
            clock: 0,
            registers: reg_vec,
            contents: vec![],
            interrupt_status: 0,
            notifications: vec![],
        }
    }

    pub fn load(&self, addr: PAddr) -> u32 {
        let offs = (addr - 0x10001000) as usize;
        match num::FromPrimitive::from_usize(offs).unwrap() {
            VirtioRegister::DeviceFeatures => {
                self.registers[VirtioRegister::DeviceFeatures as usize]
                    >> (self.registers[VirtioRegister::DeviceFeaturesSel as usize] * 32)
            }
            VirtioRegister::DriverFeatures => {
                self.registers[VirtioRegister::DriverFeatures as usize]
                    >> (self.registers[VirtioRegister::DriverFeaturesSel as usize] * 32)
            }
            _ => self.registers[offs],
        }
    }
    /*        match addr - 0x10001000 {
        0x0 => 0x74726976,
        0x4 => 2,          // Device version number
        0x8 => 2,          // Virtio Subsystem Device id: 2 (Block device)
        0xc => 0x554d4551, // Virtio Subsystem Vendor id: 0x554d4551
        // Flags representing features the device supports
        0x10 => (self.device_features >> (self.device_features_sel * 32)) as u32,
        0x30 => self.queue_select,
        0x34 => MAX_QUEUE_SIZE, // Maximum virtual queue size
        // Virtual queue ready bit
        0x44 => self.queue_ready,
        0x70 => self.status,

        _ => todo!("virtio load {:#x?}", addr - 0x10001000),
    } */
    //}

    pub fn store(&mut self, addr: PAddr, value: u32) {
        let offs = (addr - 0x10001000) as usize;
        self.registers[offs as usize] = value;
        match num::FromPrimitive::from_usize(offs) {
            Some(VirtioRegister::QueueNotify) => {
                self.notifications.push(self.clock);
            }
            Some(VirtioRegister::InterruptACK) => match (value & 0x1) == 1 {
                true => self.interrupt_status &= !0x1,
                false => panic!("virtio: unknown ACK"),
            },
            _ => {}
        }
        // match addr - 0x10001000 {
        //     0x20 => self.driver_features = value,
        //     0x30 => self.queue_select = value,
        //     0x38 => self.queue_size = value,
        //     0x44 => self.queue_ready = value,
        //     0x50 => self.queue_notify = value,
        //     0x70 => self.status = value,
        //     0x80 => self.queue_desc_low = value,
        //     0x84 => self.queue_desc_high = value,
        //     // Virtual queue’s Driver Area 64 bit long physical address
        //     0x90 => self.queue_driver_low = value,
        //     0x94 => self.queue_driver_high = value,
        //     0xa0 => self.queue_device_low = value,
        //     0xa4 => self.queue_device_high = value,
        //     _ => todo!("virtio store {:#x?} @ {:#x?}", value, addr - 0x10001000),
        // }
    }

    /// Indicates whether `VirtioBlockDisk` raises an interrupt signal
    pub fn is_interrupting(&self) -> bool {
        (self.interrupt_status & 0x1) == 1
    }

    /// Initializes filesystem content.     ///
    /// # Arguments
    /// * `contents` filesystem content binary
    pub fn init(&mut self, contents: Vec<u8>) {
        assert!(self.contents.len() == 0); // Only call me once please

        for _i in 0..((contents.len() + 7) / 8) {
            self.contents.push(0);
        }
        for i in 0..contents.len() {
            // Wrangle in the bytes..
            let index = (i >> 3) as usize;
            let pos = (i % 8) * 8;
            self.contents[index] =
                (self.contents[index] & !(0xff << pos)) | ((contents[i] as u64) << pos);
        }
    }

    pub fn tick(&mut self, memory: &mut MemoryWrapper) {
        if self.notifications.len() > 0 && (self.clock == self.notifications[0] + DISK_ACCESS_DELAY)
        {
            // bit 0 in interrupt_status register indicates
            // the interrupt was asserted because the device has used a buffer
            // in at least one of the active virtual queues.
            self.interrupt_status |= 0x1;
            //self.handle_disk_access(memory);
            self.notifications.remove(0);
        }
        self.clock = self.clock.wrapping_add(1);
    }

    // /// Loads register content
    // ///
    // /// # Arguments
    // /// * `address`
    // pub fn load(&self, address: u64) -> u8 {
    //     //println!("Disk Load AD:{:X}", address);
    //     match address {
    //         // Magic number: 0x74726976
    //         0x10001000 => 0x76,
    //         0x10001001 => 0x69,
    //         0x10001002 => 0x72,
    //         0x10001003 => 0x74,
    //         // Device version: 2 (WAS:) 1 (Legacy device))
    //         0x10001004 => 2,
    //         // Virtio Subsystem Device id: 2 (Block device)
    //         0x10001008 => 2,
    //         // Virtio Subsystem Vendor id: 0x554d4551
    //         0x1000100c => 0x51,
    //         0x1000100d => 0x45,
    //         0x1000100e => 0x4d,
    //         0x1000100f => 0x55,
    //         // Flags representing features the device supports
    //         0x10001010 => ((self.device_features >> (self.device_features_sel * 32)) & 0xff) as u8,
    //         0x10001011 => {
    //             (((self.device_features >> (self.device_features_sel * 32)) >> 8) & 0xff) as u8
    //         }
    //         0x10001012 => {
    //             (((self.device_features >> (self.device_features_sel * 32)) >> 16) & 0xff) as u8
    //         }
    //         0x10001013 => {
    //             (((self.device_features >> (self.device_features_sel * 32)) >> 24) & 0xff) as u8
    //         }
    //         // Maximum virtual queue size
    //         0x10001034 => MAX_QUEUE_SIZE as u8,
    //         0x10001035 => (MAX_QUEUE_SIZE >> 8) as u8,
    //         0x10001036 => (MAX_QUEUE_SIZE >> 16) as u8,
    //         0x10001037 => (MAX_QUEUE_SIZE >> 24) as u8,
    //         // Guest physical page number of the virtual queue
    //         0x10001040 => self.queue_pfn as u8,
    //         0x10001041 => (self.queue_pfn >> 8) as u8,
    //         0x10001042 => (self.queue_pfn >> 16) as u8,
    //         0x10001043 => (self.queue_pfn >> 24) as u8,
    //         // Interrupt status
    //         0x10001060 => self.interrupt_status as u8,
    //         0x10001061 => (self.interrupt_status >> 8) as u8,
    //         0x10001062 => (self.interrupt_status >> 16) as u8,
    //         0x10001063 => (self.interrupt_status >> 24) as u8,
    //         // Device status
    //         0x10001070 => self.status as u8,
    //         0x10001071 => (self.status >> 8) as u8,
    //         0x10001072 => (self.status >> 16) as u8,
    //         0x10001073 => (self.status >> 24) as u8,
    //         // Configurations @TODO: Implement properly
    //         0x10001100 => 0x00,
    //         0x10001101 => 0x20,
    //         0x10001102 => 0x03,
    //         0x10001103 => 0,
    //         0x10001104 => 0,
    //         0x10001105 => 0,
    //         0x10001106 => 0,
    //         0x10001107 => 0,
    //         _ => 0,
    //     }
    // }

    // /// Stores register content
    // ///
    // /// # Arguments
    // /// * `address`
    // /// * `value`
    // pub fn store(&mut self, address: u64, value: u8) {
    //     //println!("Disk Store AD:{:X} VAL:{:X}", address, value);
    //     match address {
    //         0x10001014 => {
    //             self.device_features_sel = (self.device_features_sel & !0xff) | (value as u32);
    //         }
    //         0x10001015 => {
    //             self.device_features_sel =
    //                 (self.device_features_sel & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x10001016 => {
    //             self.device_features_sel =
    //                 (self.device_features_sel & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x10001017 => {
    //             self.device_features_sel =
    //                 (self.device_features_sel & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         0x10001020 => {
    //             self.driver_features = (self.driver_features & !0xff) | (value as u32);
    //         }
    //         0x10001021 => {
    //             self.driver_features =
    //                 (self.driver_features & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x10001022 => {
    //             self.driver_features =
    //                 (self.driver_features & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x10001023 => {
    //             self.driver_features =
    //                 (self.driver_features & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         0x10001028 => {
    //             self.guest_page_size = (self.guest_page_size & !0xff) | (value as u32);
    //         }
    //         0x10001029 => {
    //             self.guest_page_size =
    //                 (self.guest_page_size & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x1000102a => {
    //             self.guest_page_size =
    //                 (self.guest_page_size & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x1000102b => {
    //             self.guest_page_size =
    //                 (self.guest_page_size & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         0x10001030 => {
    //             self.queue_select = (self.queue_select & !0xff) | (value as u32);
    //         }
    //         0x10001031 => {
    //             self.queue_select = (self.queue_select & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x10001032 => {
    //             self.queue_select = (self.queue_select & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x10001033 => {
    //             self.queue_select = (self.queue_select & !(0xff << 24)) | ((value as u32) << 24);
    //             if self.queue_select != 0 {
    //                 panic!("Virtio: No multi queue support yet.");
    //             }
    //         }
    //         0x10001038 => {
    //             self.queue_size = (self.queue_size & !0xff) | (value as u32);
    //         }
    //         0x10001039 => {
    //             self.queue_size = (self.queue_size & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x1000103a => {
    //             self.queue_size = (self.queue_size & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x1000103b => {
    //             self.queue_size = (self.queue_size & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         0x1000103c => {
    //             self.queue_align = (self.queue_align & !0xff) | (value as u32);
    //         }
    //         0x1000103d => {
    //             self.queue_align = (self.queue_align & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x1000103e => {
    //             self.queue_align = (self.queue_align & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x1000103f => {
    //             self.queue_align = (self.queue_align & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         0x10001040 => {
    //             self.queue_pfn = (self.queue_pfn & !0xff) | (value as u32);
    //         }
    //         0x10001041 => {
    //             self.queue_pfn = (self.queue_pfn & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x10001042 => {
    //             self.queue_pfn = (self.queue_pfn & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x10001043 => {
    //             self.queue_pfn = (self.queue_pfn & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         // @TODO: Queue request support
    //         0x10001050 => {
    //             self.queue_notify = (self.queue_notify & !0xff) | (value as u32);
    //         }
    //         0x10001051 => {
    //             self.queue_notify = (self.queue_notify & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x10001052 => {
    //             self.queue_notify = (self.queue_notify & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x10001053 => {
    //             self.queue_notify = (self.queue_notify & !(0xff << 24)) | ((value as u32) << 24);
    //             self.notify_clocks.push(self.clock);
    //         }
    //         0x10001064 => {
    //             // interrupt ack
    //             if (value & 0x1) == 1 {
    //                 self.interrupt_status &= !0x1;
    //             } else {
    //                 panic!("Unknown ack {:X}", value);
    //             }
    //         }
    //         0x10001070 => {
    //             self.status = (self.status & !0xff) | (value as u32);
    //         }
    //         0x10001071 => {
    //             self.status = (self.status & !(0xff << 8)) | ((value as u32) << 8);
    //         }
    //         0x10001072 => {
    //             self.status = (self.status & !(0xff << 16)) | ((value as u32) << 16);
    //         }
    //         0x10001073 => {
    //             self.status = (self.status & !(0xff << 24)) | ((value as u32) << 24);
    //         }
    //         _ => {}
    //     };
    // }

    /// Fast path of transferring the data from disk to memory.
    ///
    /// # Arguments
    /// * `memory`
    /// * `mem_addresss` Physical address. Must be eight-byte aligned.
    /// * `disk_address` Must be eight-byte aligned.
    /// * `length` Must be eight-byte aligned.
    fn transfer_from_disk(
        &mut self,
        memory: &mut MemoryWrapper,
        mem_address: u64,
        disk_address: u64,
        length: u64,
    ) {
        debug_assert!(
            (mem_address % 8) == 0,
            "Memory address should be eight-byte aligned. {:X}",
            mem_address
        );
        debug_assert!(
            (disk_address % 8) == 0,
            "Disk address should be eight-byte aligned. {:X}",
            disk_address
        );
        debug_assert!(
            (length % 8) == 0,
            "Length should be eight-byte aligned. {:X}",
            length
        );
        for i in 0..(length / 8) {
            let disk_index = ((disk_address + i * 8) >> 3) as usize;
            memory.write64(mem_address + i * 8, self.contents[disk_index]);
        }
    }

    /// Fast path of transferring the data from memory to disk.
    ///
    /// # Arguments
    /// * `memory`
    /// * `mem_addresss` Physical address. Must be eight-byte aligned.
    /// * `disk_address` Must be eight-byte aligned.
    /// * `length` Must be eight-byte aligned.
    fn transfer_to_disk(
        &mut self,
        memory: &mut MemoryWrapper,
        mem_address: u64,
        disk_address: u64,
        length: u64,
    ) {
        debug_assert!(
            (mem_address % 8) == 0,
            "Memory address should be eight-byte aligned. {:X}",
            mem_address
        );
        debug_assert!(
            (disk_address % 8) == 0,
            "Disk address should be eight-byte aligned. {:X}",
            disk_address
        );
        debug_assert!(
            (length % 8) == 0,
            "Length should be eight-byte aligned. {:X}",
            length
        );
        for i in 0..(length / 8) {
            let disk_index = ((disk_address + i * 8) >> 3) as usize;
            self.contents[disk_index] = memory.read64(mem_address + i * 8);
        }
    }

    /// Reads a byte from disk.
    ///
    /// # Arguments
    /// * `addresss` Address in disk
    fn read_from_disk(&mut self, address: u64) -> u8 {
        let index = (address >> 3) as usize;
        let pos = (address % 8) * 8;
        (self.contents[index] >> pos) as u8
    }

    /// Writes a byte to disk.
    ///
    /// # Arguments
    /// * `addresss` Address in disk
    /// * `value` Data written to disk
    fn write_to_disk(&mut self, address: u64, value: u8) {
        let index = (address >> 3) as usize;
        let pos = (address % 8) * 8;
        self.contents[index] = (self.contents[index] & !(0xff << pos)) | ((value as u64) << pos);
    }

    // fn get_page_address(&self) -> u64 {
    //     self.queue_pfn as u64 * self.guest_page_size as u64
    // }

    // Virtqueue layout: Starting at page address
    //
    // struct virtq {
    //   struct virtq_desc desc[queue_size]; // queue_size * 16bytes
    //   struct virtq_avail avail;           // 2 * 2bytes + queue_size * 2bytes
    //   uint8 pad[padding];                 // until queue_align
    //   struct virtq_used used;             // 2 * 2bytes + queue_size * 8bytes
    // }
    //
    // struct virtq_desc {
    //   uint64 addr;
    //   uint32 len;
    //   uint16 flags;
    //   uint16 next;
    // }
    //
    // struct virtq_avail {
    //   uint16 flags;
    //   uint16 idx;
    //   uint16 ring[queue_size];
    // }
    //
    // struct virtq_used {
    //   uint16 flags;
    //   uint16 idx;
    //   struct virtq_used_elem ring[queue_size];
    // }
    //
    // struct virtq_used_elem {
    //   uint32 id;
    //   uint32 len;
    // }

    // fn get_base_desc_address(&self) -> u64 {
    //     self.get_page_address()
    // }

    // fn get_base_avail_address(&self) -> u64 {
    //     self.get_base_desc_address() + self.queue_size as u64 * 16
    // }

    // fn get_base_used_address(&self) -> u64 {
    //     let align = self.queue_align as u64;
    //     let queue_size = self.queue_size as u64;
    //     ((self.get_base_avail_address() + 4 + queue_size * 2 + align - 1) / align) * align
    // }

    // // @TODO: Follow the virtio block specification more propertly.
    // fn handle_disk_access(&mut self, memory: &mut MemoryWrapper) {
    //     let base_desc_address = self.get_base_desc_address();
    //     let base_avail_address = self.get_base_avail_address();
    //     let base_used_address = self.get_base_used_address();
    //     let queue_size = self.queue_size as u64;

    //     let _avail_flag = memory.read16(base_avail_address) as u64;
    //     let _avail_index = memory.read16(base_avail_address.wrapping_add(2)) as u64;
    //     let desc_index_address = base_avail_address
    //         .wrapping_add(4)
    //         .wrapping_add((self.used_ring_index as u64 % queue_size) * 2);
    //     let desc_head_index = (memory.read16(desc_index_address) as u64) % queue_size;

    //     /*
    //     println!("Desc AD:{:X}", base_desc_address);
    //     println!("Avail AD:{:X}", base_avail_address);
    //     println!("Used AD:{:X}", base_used_address);
    //     println!("Avail flag:{:X}", _avail_flag);
    //     println!("Avail index:{:X}", _avail_index);
    //     println!("Used ring index:{:X}", self.used_ring_index);
    //     println!("Desc head index:{:X}", desc_head_index);
    //     */
    //     let mut _blk_type = 0;
    //     let mut _blk_reserved = 0;
    //     let mut blk_sector = 0;
    //     let mut desc_num = 0;
    //     let mut desc_next = desc_head_index;
    //     loop {
    //         let desc_element_address = base_desc_address + 16 * desc_next;
    //         let desc_addr = memory.read64(desc_element_address);
    //         let desc_len = memory.read32(desc_element_address.wrapping_add(8));
    //         let desc_flags = memory.read16(desc_element_address.wrapping_add(12));
    //         desc_next = (memory.read16(desc_element_address.wrapping_add(14)) as u64) % queue_size;

    //         /*
    //         println!("Desc addr:{:X}", desc_addr);
    //         println!("Desc len:{:X}", desc_len);
    //         println!("Desc flags:{:X}", desc_flags);
    //         println!("Desc next:{:X}", desc_next);
    //         */
    //         // Assuming address in memory equals to or greater than DRAM_BASE.
    //         match desc_num {
    //             0 => {
    //                 // First descriptor: Block description
    //                 // struct virtio_blk_req {
    //                 //   uint32 type;
    //                 //   uint32 reserved;
    //                 //   uint64 sector;
    //                 // }

    //                 // Read/Write operation can be distinguished with the second descriptor flags
    //                 // so we can ignore blk_type?
    //                 _blk_type = memory.read32(desc_addr);
    //                 _blk_reserved = memory.read32(desc_addr.wrapping_add(4));
    //                 blk_sector = memory.read64(desc_addr.wrapping_add(8));
    //                 /*
    //                 println!("Blk type:{:X}", _blk_type);
    //                 println!("Blk reserved:{:X}", _blk_reserved);
    //                 println!("Blk sector:{:X}", blk_sector);
    //                 */
    //             }
    //             1 => {
    //                 // Second descriptor: Read/Write disk
    //                 match (desc_flags & VIRTQ_DESC_F_WRITE) == 0 {
    //                     true => {
    //                         // write to disk
    //                         if (desc_addr % 8) == 0
    //                             && ((blk_sector * SECTOR_SIZE) % 8) == 0
    //                             && (desc_len % 8) == 0
    //                         {
    //                             // Enter fast path if possible
    //                             self.transfer_to_disk(
    //                                 memory,
    //                                 desc_addr,
    //                                 blk_sector * SECTOR_SIZE,
    //                                 desc_len as u64,
    //                             );
    //                         } else {
    //                             for i in 0..desc_len as u64 {
    //                                 let data = memory.read8(desc_addr + i);
    //                                 self.write_to_disk(blk_sector * SECTOR_SIZE + i, data);
    //                             }
    //                         }
    //                     }
    //                     false => {
    //                         // read from disk
    //                         if (desc_addr % 8) == 0
    //                             && ((blk_sector * SECTOR_SIZE) % 8) == 0
    //                             && (desc_len % 8) == 0
    //                         {
    //                             // Enter fast path if possible
    //                             self.transfer_from_disk(
    //                                 memory,
    //                                 desc_addr,
    //                                 blk_sector * SECTOR_SIZE,
    //                                 desc_len as u64,
    //                             );
    //                         } else {
    //                             for i in 0..desc_len as u64 {
    //                                 let data = self.read_from_disk(blk_sector * SECTOR_SIZE + i);
    //                                 memory.write8(desc_addr + i, data);
    //                             }
    //                         }
    //                     }
    //                 };
    //             }
    //             2 => {
    //                 // Third descriptor: Result status
    //                 if (desc_flags & VIRTQ_DESC_F_WRITE) == 0 {
    //                     panic!("Third descriptor should be write.");
    //                 }
    //                 if desc_len != 1 {
    //                     panic!("Third descriptor length should be one.");
    //                 }
    //                 memory.write8(desc_addr, 0); // 0 means succeeded
    //             }
    //             _ => {}
    //         };

    //         desc_num += 1;

    //         if (desc_flags & VIRTQ_DESC_F_NEXT) == 0 {
    //             break;
    //         }
    //     }

    //     if desc_num != 3 {
    //         panic!("Descript chain length should be three.");
    //     }

    //     memory.write32(
    //         base_used_address
    //             .wrapping_add(4)
    //             .wrapping_add((self.used_ring_index as u64 % queue_size) * 8),
    //         desc_head_index as u32,
    //     );

    //     self.used_ring_index = self.used_ring_index.wrapping_add(1);
    //     memory.write16(base_used_address.wrapping_add(2), self.used_ring_index);
    // }
}
