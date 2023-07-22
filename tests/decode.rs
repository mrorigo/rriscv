//use syn::Ident;

use rriscv::{
    self,
    cpu::{self, Core},
    decoder::{CSStype, DecodedInstruction, InstructionDecoder, Itype, Jtype, Stype, Utype},
    memory::Memory,
    opcodes::{CompressedOpcode, MajorOpcode},
    pipeline::RawInstruction,
};

const VBASE: u64 = 0x8000_0000;

macro_rules! test_case {
    ( $n:ident, $a:expr ) => {
        #[no_mangle]
        #[test]
        pub fn $n() {
            let memory = &mut Memory::create(VBASE, 4096);
            let core = cpu::Core::create(0x0, memory);
            $a.run(core)
        }
    };
}

impl TestCase {
    fn run(&self, core: Core<'_>) {
        let decoded = core.decode_instruction(self.instruction);
        assert!(
            decoded == self.decoded,
            "{}: opcode {:?} != {:?}",
            self.description,
            decoded,
            self.decoded
        )
    }

    pub fn create(
        description: &'static str,
        instruction: u32,
        compressed: bool,
        decoded: DecodedInstruction,
    ) -> TestCase {
        TestCase {
            description,
            instruction: RawInstruction {
                compressed,
                word: instruction,
                pc: VBASE,
            },
            decoded,
        }
    }
}

pub struct TestCase {
    description: &'static str,
    instruction: RawInstruction,
    decoded: DecodedInstruction,
}

test_case! {lui1, TestCase::create(&"LUI X14,0x2004", 0x02004737,false, DecodedInstruction::U(Utype { opcode: MajorOpcode::LUI, rd: 14, imm20: 0x2004}))}
test_case! {lui2, TestCase::create(&"LUI X14,0x7ff4", 0x07ff4737, false, DecodedInstruction::U(Utype { opcode: MajorOpcode::LUI, rd: 14, imm20: 0x7ff4}))}
test_case! {auipc, TestCase::create(&"AUIPC X2,0x9", 0x00009117, false, DecodedInstruction::U(Utype {opcode: MajorOpcode::AUIPC, rd: 2, imm20:0x09}))}
test_case! {jal, TestCase::create(&"JAL X1,0x9", 0x076000ef, false, DecodedInstruction::J(Jtype {opcode: MajorOpcode::JAL, imm20: 0x3b, rd:1}))}

test_case! {addi, TestCase::create(&"ADDI X2,X2,-1520", 0xa1010113, false, DecodedInstruction::I(Itype { opcode: MajorOpcode::OP_IMM, rd:2, rs1: 2, imm12: 2576, funct3: 0b000 }))}

test_case! {sd,   TestCase::create(&"SD X1, 8(X2)", 0x00113423, false, DecodedInstruction::S(Stype{opcode: MajorOpcode::STORE, rs1: 2, rs2: 1, funct3: 0b011, imm12: 8}))}
test_case! {c_sdsp, TestCase::create(&"C.SDSP  x1,8(x2)", 0xe406, true, DecodedInstruction::CSS(CSStype {opcode: CompressedOpcode::C2, rs2: 1, funct3: 0b111, uimm: 8})) }
// test_case! {c_addi4spn, TestCase::ciw(&"C.ADDI4SPN X8,X2,16", OpCodes::ADDI, 0x0800, 8, 0b000, 16) }
test_case! {csrrw0, TestCase::create(&"CSRRW X0,mstatus,X15", 0x30079073, false, DecodedInstruction::I(Itype { opcode: MajorOpcode::SYSTEM, rd: 0, rs1: 15, funct3: 0b001, imm12: 0x300 })) }
test_case! {csrrw1, TestCase::create(&"CSRRW X1,mstatus,X15", 0x302790f3, false, DecodedInstruction::I(Itype { opcode: MajorOpcode::SYSTEM, rd: 1, rs1: 15, funct3: 0b001, imm12: 0x302 })) }

// test_case! {addiw, TestCase::i(&"ADDIW X11,X15,0", OpCodes::ADDIW, 0x0007859b, 11, 15, 0b000, 0) }
