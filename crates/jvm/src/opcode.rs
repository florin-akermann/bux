//! The opcodes version 0.1 emits, each named as the JVM specification names it.

use lumen_ir::Comparison;

pub(crate) const ICONST_0: u8 = 0x03;
pub(crate) const ICONST_1: u8 = 0x04;
pub(crate) const BIPUSH: u8 = 0x10;
pub(crate) const SIPUSH: u8 = 0x11;
pub(crate) const LDC: u8 = 0x12;
pub(crate) const LDC_W: u8 = 0x13;
pub(crate) const LDC2_W: u8 = 0x14;
pub(crate) const ILOAD: u8 = 0x15;
pub(crate) const LLOAD: u8 = 0x16;
pub(crate) const ALOAD: u8 = 0x19;
pub(crate) const ISTORE: u8 = 0x36;
pub(crate) const LSTORE: u8 = 0x37;
pub(crate) const ASTORE: u8 = 0x3A;
pub(crate) const AASTORE: u8 = 0x53;
pub(crate) const POP: u8 = 0x57;
pub(crate) const POP2: u8 = 0x58;
pub(crate) const DUP: u8 = 0x59;
pub(crate) const DUP2: u8 = 0x5C;
pub(crate) const LADD: u8 = 0x61;
pub(crate) const LSUB: u8 = 0x65;
pub(crate) const LMUL: u8 = 0x69;
pub(crate) const LDIV: u8 = 0x6D;
pub(crate) const LREM: u8 = 0x71;
pub(crate) const LNEG: u8 = 0x75;
pub(crate) const IXOR: u8 = 0x82;
pub(crate) const IINC: u8 = 0x84;
pub(crate) const I2L: u8 = 0x85;
pub(crate) const LCMP: u8 = 0x94;
pub(crate) const IFEQ: u8 = 0x99;
pub(crate) const IF_ICMPEQ: u8 = 0x9F;
pub(crate) const GOTO: u8 = 0xA7;
pub(crate) const IRETURN: u8 = 0xAC;
pub(crate) const LRETURN: u8 = 0xAD;
pub(crate) const ARETURN: u8 = 0xB0;
pub(crate) const RETURN: u8 = 0xB1;
pub(crate) const GETSTATIC: u8 = 0xB2;
pub(crate) const GETFIELD: u8 = 0xB4;
pub(crate) const PUTFIELD: u8 = 0xB5;
pub(crate) const INVOKEVIRTUAL: u8 = 0xB6;
pub(crate) const INVOKESPECIAL: u8 = 0xB7;
pub(crate) const INVOKESTATIC: u8 = 0xB8;
pub(crate) const INVOKEINTERFACE: u8 = 0xB9;
pub(crate) const NEW: u8 = 0xBB;
pub(crate) const ANEWARRAY: u8 = 0xBD;
pub(crate) const ATHROW: u8 = 0xBF;
pub(crate) const CHECKCAST: u8 = 0xC0;
pub(crate) const INSTANCEOF: u8 = 0xC1;
pub(crate) const WIDE: u8 = 0xC4;
pub(crate) const IFNULL: u8 = 0xC6;

/// How far a comparison's opcode sits from `ifeq`, which is as far as it sits from `if_icmpeq`.
///
/// The two runs of six are in the same order, which is not the order a comparison is written in,
/// so this is where the two orders are reconciled and nowhere else.
pub(crate) const fn step(comparison: Comparison) -> u8 {
    match comparison {
        Comparison::Equal => 0,
        Comparison::NotEqual => 1,
        Comparison::Less => 2,
        Comparison::GreaterOrEqual => 3,
        Comparison::Greater => 4,
        Comparison::LessOrEqual => 5,
    }
}
