use rriscv::memory::{Memory, MemoryOperations};

#[test]
pub fn zero_initialized() {
    let vbase: u64 = 0x8000_0000;
    let memory = &mut Memory::create();
    memory.add_segment(vbase, 4096);

    for i in 0..4095 {
        assert!(
            memory
                .read_single(
                    vbase.wrapping_add(i),
                    rriscv::memory::MemoryAccessWidth::BYTE
                )
                .unwrap()
                == 0
        )
    }
}

#[test]
pub fn write_read_single() {
    let vbase: u64 = 0x8000_0000;
    let memory = &mut Memory::create();
    memory.add_segment(vbase, 4096);

    for i in 0..4095 {
        memory.write_single(
            vbase.wrapping_add(i),
            i,
            rriscv::memory::MemoryAccessWidth::BYTE,
        );

        let ret = memory
            .read_single(
                vbase.wrapping_add(i),
                rriscv::memory::MemoryAccessWidth::BYTE,
            )
            .unwrap();
        assert!(ret == (i & 0xff), "{} != {}", i, ret);

        let ret2 = memory
            .read_single(
                vbase.wrapping_add(i + 1),
                rriscv::memory::MemoryAccessWidth::BYTE,
            )
            .unwrap();
        assert!(ret2 == 0, "{} != 0", ret2)
    }
}
