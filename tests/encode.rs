use rriscv::instructions::{
    functions::{OpImmFunct3, RV32M_Funct3},
    itype::Itype,
    rtype::Rtype,
    Instruction,
};

struct TestCase<T> {
    instruction: Instruction<T>,
    expect: &'static str,
}

trait TestCaseEval<T> {
    fn eval(&self);
}

impl TestCaseEval<Rtype> for TestCase<Rtype> {
    fn eval(&self) {
        assert_eq!(self.expect.to_string(), self.instruction.to_string());
    }
}
impl TestCaseEval<Itype> for TestCase<Itype> {
    fn eval(&self) {
        assert_eq!(self.expect.to_string(), self.instruction.to_string());
    }
}

#[test]
fn encode_r() {
    let cases: &[&TestCase<Rtype>; 1] = &[&TestCase {
        expect: &"MUL x10,x9,x11",
        instruction: Instruction::MUL(Rtype {
            opcode: rriscv::instructions::opcodes::MajorOpcode::OP,
            rd: 10,
            rs1: 9,
            rs2: 11,
            funct3: num::FromPrimitive::from_u8(RV32M_Funct3::MUL as u8).unwrap(),
            funct7: 1,
        }),
    }];

    for &case in cases.to_vec().iter() {
        case.eval();
    }
}

#[test]
fn encode_i() {
    let cases: &[&TestCase<Itype>; 1] = &[&TestCase {
        expect: &"ADDI x3,x9,128",
        instruction: Instruction::ADDI(Itype {
            opcode: rriscv::instructions::opcodes::MajorOpcode::OP_IMM,
            rd: 3,
            rs1: 9,
            imm12: 0x80,
            funct3: num::FromPrimitive::from_u8(OpImmFunct3::ADDI as u8).unwrap(),
        }),
    }];

    for &case in cases.to_vec().iter() {
        case.eval();
    }
}
