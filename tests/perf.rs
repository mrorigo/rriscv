use std::time::Instant;

use rriscv::{
    cpu,
    instructions::{utype::Utype, Instruction, InstructionExcecutor},
    memory::Memory,
    pipeline::Stage,
};
const VBASE: u64 = 0x8000_0000;

#[test]
pub fn perf_auipc() {
    let start = Instant::now();

    let utype = Utype {
        imm20: 9,
        opcode: rriscv::instructions::opcodes::MajorOpcode::AUIPC,
        rd: 2,
    };
    let utype2 = Utype {
        imm20: 1,
        opcode: rriscv::instructions::opcodes::MajorOpcode::AUIPC,
        rd: 2,
    };
    let iterations = 10000000;

    let memory = &mut Memory::create(VBASE, 4096);
    let mut core = cpu::Core::create(0x0, memory);
    let mut sum: u64 = 0;

    for i in 0..iterations {
        let instr = Instruction::AUIPC(utype);
        match instr.run(&mut core) {
            Stage::WRITEBACK(wb) => sum = sum.wrapping_add(wb.unwrap().value),
            _ => panic!(),
        }
        let instr = Instruction::AUIPC(utype2);
        match instr.run(&mut core) {
            Stage::WRITEBACK(wb) => sum = sum.wrapping_add(wb.unwrap().value),
            _ => panic!(),
        }
    }

    let duration = start.elapsed();
    println!(
        "sum: {:#x?}  duration: {:?} ({}M/s)",
        sum,
        duration,
        (iterations as f32 / duration.as_secs_f32()) / 1e6
    );
    assert!(false)
}
