use std::fmt;
use std::fmt::Display;

pub struct Disassembler;

#[derive(Debug)]
pub enum Operand {
    /* Unused */
    Unused,
    /* 12-bit address */
    Address(u16),
    /* 4-bit register index */
    Register(u8),
    /* 4-bit / 8-bit immediate */
    Immediate(u8),
    /* Index register */
    Index,
    /* Delay Timer (dt) register */
    DelayTimer,
    /* Sound Timer (st) register */
    SoundTimer,
    /* Key Press */
    Key,
    /* Font */
    Font,
    /* BCD */
    Bcd
}

impl Operand {
    pub fn unused(&self) -> bool {
        match self {
            Operand::Unused => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub mnemonic: &'static str,
    pub opcode: u16,
    pub operands: [Operand; 3]
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Immediate(v) => write!(f, "0x{:02x}", v),
            Operand::Register(idx) => write!(f, "v{:x}", idx),
            Operand::Address(addr) => write!(f, "#{:03x}", addr),
            Operand::DelayTimer => write!(f, "dt"),
            Operand::SoundTimer => write!(f, "st"),
            Operand::Key => write!(f, "k"),
            Operand::Index => write!(f, "i"),
            Operand::Font => write!(f, "f"),
            Operand::Bcd => write!(f, "b"),
            Operand::Unused => Ok(())
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mnemonic)?;
        match self.operands.as_ref() {
            [Operand::Unused, Operand::Unused, Operand::Unused] => Ok(()),
            [a, Operand::Unused, Operand::Unused] => write!(f, " {}", a),
            [a, b, Operand::Unused] => write!(f, " {}, {}", a, b),
            [a, b, c] => write!(f, " {}, {}, {}", a, b, c),
            _ => Ok(())
        }
    }
}

impl Instruction {
    pub fn new(mnemonic: &'static str, opcode: u16, operands: [Operand; 3]) -> Option<Self> {
        Some(Instruction {
            mnemonic,
            opcode,
            operands
        })
    }
}

impl Default for Disassembler {
    fn default() -> Self {
        Self {}
    }
}

impl Disassembler {
    pub fn disassemble(&self, rom: &[u8]) -> Option<Instruction> {
        if rom.len() < 2 { return None };

        let a = rom[0] as u16;
        let b = rom[1] as u16;
        let opcode: u16 = (a << 8) | b;

        let nnn = opcode & 0x0fff;
        let n = (opcode & 0x000f) as u8;
        let kk = (opcode & 0x00ff) as u8;
        let x = ((opcode & 0x0f00) >> 8) as u8;
        let y = ((opcode & 0x00f0) >> 4) as u8;

        let call = |mnemonic| Instruction::new(mnemonic, opcode, [Operand::Unused, Operand::Unused, Operand::Unused]);
        let unary = |mnemonic, a| Instruction::new(mnemonic, opcode, [a, Operand::Unused, Operand::Unused]);
        let binary = |mnemonic, a, b| Instruction::new(mnemonic, opcode, [a, b, Operand::Unused]);
        let ternary = |mnemonic, a, b, c| Instruction::new(mnemonic, opcode, [a, b, c]);

        match opcode & 0xf000 {
            0x0000 => match opcode {
                0x0000 => call("nop"),
                0x00e0 => call("cls"),
                0x00ee => call("ret"),
                0x00fd => call("exit"),
                _ => None
            },
            0x1000 => unary("jp", Operand::Address(nnn)),
            0x2000 => unary("call", Operand::Address(nnn)),
            0x3000 => binary("se", Operand::Register(x), Operand::Immediate(kk)),
            0x4000 => binary("sne", Operand::Register(x), Operand::Immediate(kk)),
            0x5000 => binary("sne", Operand::Register(x), Operand::Register(y)),
            0x6000 => binary("ld", Operand::Register(x), Operand::Immediate(kk)),
            0x7000 => binary("add", Operand::Register(x), Operand::Immediate(kk)),
            0x8000 => match opcode & 0xf00f {
                0x8000 => binary("ld", Operand::Register(x), Operand::Register(y)),
                0x8001 => binary("or", Operand::Register(x), Operand::Register(y)),
                0x8002 => binary("and", Operand::Register(x), Operand::Register(y)),
                0x8003 => binary("xor", Operand::Register(x), Operand::Register(y)),
                0x8004 => binary("add", Operand::Register(x), Operand::Register(y)),
                0x8005 => binary("sub", Operand::Register(x), Operand::Register(y)),
                0x8006 => binary("shr", Operand::Register(x), Operand::Register(y)),
                0x8007 => binary("subn", Operand::Register(x), Operand::Register(y)),
                0x800e => binary("shl", Operand::Register(x), Operand::Register(y)),
                _ => None
            },
            0x9000 => binary("sne", Operand::Register(x), Operand::Register(y)),
            0xa000 => binary("ld", Operand::Index, Operand::Address(nnn)),
            0xb000 => binary("jp", Operand::Register(0), Operand::Address(nnn)),
            0xc000 => binary("rnd", Operand::Register(x), Operand::Immediate(kk)),
            0xd000 => ternary("drw", Operand::Register(x), Operand::Register(y), Operand::Immediate(n)),
            0xe000 => match opcode & 0xf0ff {
                0xe09e => unary("skp", Operand::Register(x)),
                0xe0a1 => unary("sknp", Operand::Register(x)),
                _ => None
            },
            0xf000 => match opcode & 0xf0ff {
                0xf007 => binary("ld", Operand::Register(x), Operand::DelayTimer),
                0xf00a => binary("ld", Operand::Register(x), Operand::Key),
                0xf015 => binary("ld", Operand::DelayTimer, Operand::Register(x)),
                0xf018 => binary("ld", Operand::SoundTimer, Operand::Register(x)),
                0xf01e => binary("add", Operand::Index, Operand::Register(x)),
                0xf029 => binary("ld", Operand::Font, Operand::Register(x)),
                0xf033 => binary("ld", Operand::Bcd, Operand::Register(x)),
                0xf055 => binary("ld", Operand::Index, Operand::Register(x)),
                0xf065 => binary("ld", Operand::Register(x), Operand::Index),
                _ => None
            },
            _ => None
        }
    }
}