use dtb::{PathStructItems, Reader};

pub struct MMU<'a> {
    DeviceTable: Reader<'a>,
}

impl MMU<'_> {
    pub fn create() -> MMU<'static> {
        //        let mut dtb_data = Vec::new();
        let content: &'static [u8; 1590] = include_bytes!("../dtb.dtb");
        // for i in 0..content.len() {
        //     dtb_data[i] = content[i];
        // }
        match dtb::Reader::read(content) {
            Ok(dtb) => {
                dtb.struct_items().for_each(|rme| {
                    let node_name = match rme.node_name() {
                        Ok(node_name) => Some(node_name),
                        _ => Some(""),
                    };
                    let name = match rme.name() {
                        Ok(name) => Some(name),
                        _ => Some(""),
                    };
                    if node_name.is_some() && name.is_some() {
                        let value_str = match rme.value_str() {
                            Ok(value_str) => value_str,
                            _ => &"",
                        };

                        println!(
                            "  nn={} n={} sv={}  v={:#x?} ua={:#x?}",
                            node_name.unwrap(),
                            name.unwrap(),
                            value_str,
                            rme.value(),
                            rme.unit_address()
                        );
                    }
                });
                MMU { DeviceTable: dtb }
            }

            _ => panic!("DTB corrupt"),
        }
    }
}

// pub trait SV39Addr {
//     fn level0(&self) -> u16; // 9 bits
//     fn level1(&self) -> u16; // 9 bits
//     fn level2(&self) -> u16; // 9 bits
//     fn offset(&self) -> u16; // 12 bits
//     fn tag(&self) -> u32; // 25 bits
// }

// impl SV39Addr for VAddr {
//     fn tag(&self) -> u32 {
//         return ((self >> 39) & 0x1ffffff) as u32;
//     }
//     fn offset(&self) -> u16 {
//         return (self & 0xfff) as u16;
//     }
//     fn level0(&self) -> u16 {
//         return ((self >> 12) & 0x1ff) as u16;
//     }
//     fn level1(&self) -> u16 {
//         return ((self >> 21) & 0x1ff) as u16;
//     }
//     fn level2(&self) -> u16 {
//         return ((self >> 30) & 0x1ff) as u16;
//     }
// }

// #[repr(u8)]
// pub enum PTEPermBit {
//     VALID = 0,
//     READ = 1,
//     WRITE = 2,
//     EXECUTE = 3,
//     USER = 4,
// }

// type PageTableEntry = u64;

// pub trait PTE {
//     fn page_number(&self) -> u64; // 44 bits
//     fn has_permission_bit(&self, bit: PTEPermBit) -> bool;
//     fn set_permission_bit(&mut self, bit: PTEPermBit);
//     fn clear_permission_bit(&mut self, bit: PTEPermBit);
// }

// impl PTE for PageTableEntry {
//     fn page_number(&self) -> u64 {
//         (self >> 10) & 0xfffffffffff
//     }

//     fn has_permission_bit(&self, bit: PTEPermBit) -> bool {
//         ((self & 0x1f) & (1 << (bit as u8))) != 0
//     }

//     fn set_permission_bit(&mut self, bit: PTEPermBit) {
//         *self |= 1 << (bit as u64);
//     }

//     fn clear_permission_bit(&mut self, bit: PTEPermBit) {
//         *self ^= !(1 << (bit as u64) - 1);
//     }
// }
