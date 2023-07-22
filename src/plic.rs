use elfloader::VAddr;

use crate::{
    cpu::{MipMask, RegisterValue, TrapCause},
    memory::MemoryOperations,
    mmio::VirtualDevice,
    mmu::MemoryRange,
};

pub struct PLIC {
    range: MemoryRange,
    clock: u64,
    irq: u32,
    enabled: u64,
    threshold: u32,
    ips: [u8; 1024],
    prios: [u32; 1024],
    needs_update_irq: bool,
    virtio_ip_cache: bool,
}

const NIPS: usize = 1024;

const VIRTIO_IRQ: u32 = 4; // @TODO: From device tree!
const UART_IRQ: u32 = 40;

impl PLIC {
    pub fn create(range: MemoryRange) -> PLIC {
        PLIC {
            range,
            clock: 0,
            irq: 0,
            enabled: 0,
            threshold: 0,
            ips: [0; NIPS],
            prios: [0; NIPS],
            needs_update_irq: false,
            virtio_ip_cache: false,
        }
    }

    // Returns new `mip` register value
    pub fn tick(&mut self, virtio_irq: bool, uart_irq: bool, mip: RegisterValue) -> RegisterValue {
        self.clock = self.clock.wrapping_add(1);

        // Detect rising edge of virtio irq
        if self.virtio_ip_cache != virtio_irq {
            if virtio_irq {
                self.set_ip(VIRTIO_IRQ);
                println!("plic: virtio triggered irq");
            }
            self.virtio_ip_cache = virtio_irq;
        }

        // UART irq is only true for one tick
        if uart_irq {
            self.set_ip(UART_IRQ);
        }

        if self.needs_update_irq {
            self.needs_update_irq = false;
            self.update_irq(mip)
        } else {
            mip
        }
    }

    fn update_irq(&mut self, mip: u64) -> u64 {
        // Hardcoded VirtIO and UART
        // @TODO: Should be configurable with device tree

        let virtio_ip = ((self.ips[(VIRTIO_IRQ >> 3) as usize] >> (VIRTIO_IRQ & 7)) & 1) == 1;
        let uart_ip = ((self.ips[(UART_IRQ >> 3) as usize] >> (UART_IRQ & 7)) & 1) == 1;

        // Which should be prioritized, virtio or uart?

        let virtio_priority = self.prios[VIRTIO_IRQ as usize];
        let uart_priority = self.prios[UART_IRQ as usize];

        let virtio_enabled = ((self.enabled >> (VIRTIO_IRQ >> 2)) & 1) == 1;
        let uart_enabled = ((self.enabled >> (UART_IRQ >> 2)) & 1) == 1;

        let ips = [virtio_ip, uart_ip];
        let enables = [virtio_enabled, uart_enabled];
        let priorities = [virtio_priority, uart_priority];
        let irqs = [VIRTIO_IRQ, UART_IRQ];

        // println!(
        //     "ips: {:?} enables: {:?} priorities: {:?}  threshold: {:#?}",
        //     ips, enables, priorities, self.threshold
        // );

        let mut irq = 0;
        let mut priority = 0;
        for i in 0..2 {
            if ips[i] && enables[i] && priorities[i] > self.threshold && priorities[i] > priority {
                irq = irqs[i] >> 2;
                priority = priorities[i];
            }
        }

        self.irq = irq;
        // if self.irq != 0 {
        //     panic!("PLIC IRQ: {:X}", self.irq);
        // }

        mip | (if irq != 0 { MipMask::SEIP as u64 } else { 0 })
    }

    fn set_ip(&mut self, irq: u32) {
        let index = (irq >> 3) as usize;
        self.ips[index] = self.ips[index] | (1 << irq);
        self.needs_update_irq = true;
    }

    fn clear_ip(&mut self, irq: u32) {
        let index = (irq >> 3) as usize;
        self.ips[index] = self.ips[index] & !(1 << irq);
        self.needs_update_irq = true;
    }
}

impl VirtualDevice for PLIC {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }

    fn write(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        todo!();
    }

    fn read(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        todo!();
    }
}

impl MemoryOperations<PLIC, u8> for PLIC {
    fn read8(&mut self, address: VAddr) -> Result<u8, TrapCause> {
        panic!();
        match address {
            0x0c000000..=0x0c000fff => {
                let offset = address % 4;
                let index = ((address - 0xc000000) >> 2) as usize;
                let pos = offset << 3;
                Ok((self.prios[index] >> pos) as u8)
            }
            _ => panic!(),
        }
    }

    fn write32(&mut self, address: VAddr, value: u32) -> Option<TrapCause> {
        match address {
            0x0c000000..=0x0c000fff => {
                let index = (address - 0xc000000) as usize;
                self.prios[index] = value;
                self.needs_update_irq = true;
                println!("plint: prio for {:#?}: {:#?}", index, value);
                None
            }
            0x0c002080 => {
                self.enabled = (self.enabled & !0xffffffff) | (value as u64);
                self.needs_update_irq = true;
                None
            }
            0x0c002084 => {
                self.enabled = (self.enabled & !(0xffffffff << 32)) | ((value as u64) << 32);
                None
            }

            0x0c201000 => {
                self.threshold = value;
                self.needs_update_irq = true;
                None
            }
            0x0c201004 => {
                self.clear_ip(value);
                None
            }

            _ => todo!("{:#x?}", address),
        }
    }
}
