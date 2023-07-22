//use syn::Ident;

use rriscv::{
    self,
    cpu::{self, Core, Register},
    decoder::{DecodedInstruction, InstructionDecoder},
    memory::Memory,
    opcodes::OpCodes,
};

const VBASE: u64 = 0x8000_0000;

macro_rules! test_case {
    ( $n:ident, $a:expr ) => {
        #[no_mangle]
        #[test]
        pub fn $n() {
            let memory = Memory::create(VBASE, 4096);
            let core = cpu::Core::create(0x0, &memory);
            run_test_case(core, $a)
        }
    };
}

macro_rules! prop_assertion {
    ( $decoded:expr, $case:expr, $prop:ident ) => {
        match $case.$prop {
            Some(val) => {
                assert!(
                    $decoded.$prop == val,
                    "{} {:?} != {:?}",
                    stringify!($prop),
                    val,
                    $decoded.$prop
                )
            }
            None => {}
        }
    };
}

fn run_test_case(mut core: Core<'_>, case: TestCase) {
    let decoded = core.decode_instruction(case.instruction);
    assert!(decoded.opcode.is_some());
    assert!(
        decoded.opcode.unwrap() == case.opcode,
        "opcode {:?} != {:?}",
        decoded.opcode.unwrap(),
        case.opcode
    );
    prop_assertion!(decoded, case, rd);
    prop_assertion!(decoded, case, rs1);
    prop_assertion!(decoded, case, rs2);
    prop_assertion!(decoded, case, imm5);
    prop_assertion!(decoded, case, imm12);
    prop_assertion!(decoded, case, imm20);
}

pub struct TestCase {
    name: &'static str,
    opcode: OpCodes,
    instruction: (bool, u32),
    rd: Option<u8>,
    rs1: Option<u8>,
    rs2: Option<u8>,
    funct3: Option<u8>,
    imm5: Option<u8>,
    imm12: Option<u16>,
    imm20: Option<u32>,
}

impl TestCase {
    // pub fn create12(
    //     name: &'static str,
    //     opcode: OpCodes,
    //     instruction: u32,
    //     rd: u8,
    //     imm12: u16,
    // ) -> TestCase {
    //     TestCase {
    //         name,
    //         opcode,
    //         instruction: (false, instruction),
    //         rd: Some(rd),
    //         rs1: None,
    //         rs2: None,
    //         funct3: None,
    //         imm5: None,
    //         imm12: Some(imm12),
    //         imm20: None,
    //     }
    // }

    pub fn uj(
        name: &'static str,
        opcode: OpCodes,
        instruction: u32,
        rd: u8,
        imm20: u32,
    ) -> TestCase {
        TestCase {
            name,
            opcode,
            instruction: (false, instruction),
            rd: Some(rd),
            rs1: None,
            rs2: None,
            funct3: None,
            imm5: None,
            imm12: None,
            imm20: Some(imm20),
        }
    }

    pub fn i(
        name: &'static str,
        opcode: OpCodes,
        instruction: u32,
        rd: u8,
        rs1: u8,
        funct3: u8,
        imm12: u16,
    ) -> TestCase {
        TestCase {
            name,
            opcode,
            instruction: (false, instruction),
            rd: Some(rd),
            rs1: Some(rs1),
            rs2: None,
            funct3: None,
            imm5: None,
            imm12: Some(imm12),
            imm20: None,
        }
    }

    pub fn s(
        name: &'static str,
        opcode: OpCodes,
        instruction: u32,
        rs1: u8,
        rs2: u8,
        funct3: u8,
        imm20: u32,
    ) -> TestCase {
        TestCase {
            name,
            opcode,
            instruction: (false, instruction),
            rs1: Some(rs1),
            rs2: Some(rs2),
            rd: None,
            funct3: Some(funct3),
            imm5: None,
            imm20: Some(imm20),
            imm12: None,
        }
    }
    // Stack-relative Store (Compressed)
    pub fn css(
        name: &'static str,
        opcode: OpCodes,
        instruction: u32,
        rs2: u8,
        funct3: u8,
        imm20: u32,
    ) -> TestCase {
        TestCase {
            name,
            opcode,
            instruction: (true, instruction),
            rs1: None,
            rs2: Some(rs2),
            rd: Some(2),
            funct3: Some(funct3),
            imm5: None,
            imm20: Some(imm20),
            imm12: None,
        }
    }

    pub fn ciw(
        name: &'static str,
        opcode: OpCodes,
        instruction: u32,
        rd: u8,
        funct3: u8,
        imm12: u16,
    ) -> TestCase {
        TestCase {
            name,
            opcode,
            instruction: (true, instruction),
            rs1: Some(2),
            rs2: None,
            rd: Some(rd),
            funct3: Some(funct3),
            imm5: None,
            imm12: Some(imm12),
            imm20: None,
        }
    }

    pub fn cs(
        name: &'static str,
        opcode: OpCodes,
        instruction: u32,
        rd: u8,
        rs2: u8,
        funct3: u8,
    ) -> TestCase {
        TestCase {
            name,
            opcode,
            instruction: (true, instruction),
            rs1: Some(rd),
            rs2: Some(rs2),
            rd: Some(rd),
            funct3: Some(funct3),
            imm5: None,
            imm12: None,
            imm20: None,
        }
    }
}

test_case! {lui1, TestCase::uj(&"LUI X14,0x2004", OpCodes::LUI, 0x02004737, 14, 0x2004)}
test_case! {lui2, TestCase::uj(&"LUI X14,0x7ff4", OpCodes::LUI, 0x07ff4737, 14, 0x7ff4)}
test_case! {auipc, TestCase::uj(&"AUIPC X2,0x9", OpCodes::AUIPC, 0x00009117, 2, 0x09)}
test_case! {jal, TestCase::uj(&"JAL X1,0x9", OpCodes::JAL, 0x076000ef, 1, 0x3b)}

test_case! {addi, TestCase::i(&"ADDI X2,X2,-1520", OpCodes::ADDI, 0xa1010113, 2, 2, 0b000, 2576)}

test_case! {sd,   TestCase::s(&"SD X1, 8(X2)", OpCodes::SD, 0x00113423, 2 ,1, 0b011, 8)}
test_case! {c_sdsp, TestCase::css(&"C.SDSP  x1,8(x2)", OpCodes::SD, 0xe406, 1, 0b111, 8) }
test_case! {c_addi4spn, TestCase::ciw(&"C.ADDI4SPN X8,X2,16", OpCodes::ADDI, 0x0800, 8, 0b000, 16) }
