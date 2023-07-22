use rriscv::{
    memory::{MemoryOperations, RAM},
    pipeline::MemoryAccessWidth,
};

#[test]
pub fn zero_initialized() {
    let vbase: u64 = 0x8000_0000;
    let memory = &mut RAM::create(vbase, 4096);

    for i in 0..4095 {
        let r = memory.read8(vbase.wrapping_add(i)).unwrap();
        assert!(r == 0, "0 != {}", r)
    }
}

#[test]
pub fn write_read_single() {
    let vbase: u64 = 0x8000_0000;
    let memory = &mut RAM::create(vbase, 4096);

    for i in 0..4095 {
        memory.write8(vbase.wrapping_add(i), i as u8);
    }

    for i in 0..4095 {
        let ret = memory.read8(vbase.wrapping_add(i)).unwrap();
        assert!(ret == (i & 0xff) as u8, "{} != {}", i, ret);
    }

    //         let ret = memory
    //             .read_single(
    //                 vbase.wrapping_add(i),
    //                 rriscv::memory::MemoryAccessWidth::BYTE,
    //             )
    //             .unwrap();
    //         assert!(ret == (i & 0xff), "{} != {}", i, ret);

    //         let ret2 = memory
    //             .read_single(
    //                 vbase.wrapping_add(i + 1),
    //                 rriscv::memory::MemoryAccessWidth::BYTE,
    //             )
    //             .unwrap();
    //         assert!(ret2 == 0, "{} != 0", ret2)
    //     }
}
