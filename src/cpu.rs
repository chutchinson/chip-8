use std::io::Write;

#[inline]
fn mask(opcode: u16, mask: u16) -> usize {
    (opcode & mask) as usize
}

#[inline]
fn lsb(n: u8) -> u8 {
    n & 0x01
}

#[inline]
fn msb(n: u8) -> u8 {
    n & 0x80
}

pub struct Cpu {
    halted: bool,
    memory: [u8; 4096],
    stack: [u16; 16],
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8
}

impl Cpu {
    
    pub fn new() -> Self {
        Cpu {
            halted: false,
            memory: [0; 4096],
            stack: [0; 16],
            v: [0; 16],
            i: 0,
            pc: 0,
            sp: 0
        }
    }

    pub fn load(&mut self, code: &[u8]) {
        let mut mem = &mut self.memory[0..];
        match mem.write(&code) {
            Ok(n) => { log!("loaded {} bytes", n) },
            _ => ()
        }
    }

    pub fn reset(&mut self) {
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.pc = 0;
        self.i = 0;
        self.sp = 0;
    }

    pub fn cycle(&mut self) {
        if self.halted {
            return
        }
        let opcode = self.fetch();
        let op = self.decode(opcode);
        op(self, opcode);
    }

    pub fn halt(&mut self) {
        self.halted = true;
        log!("[cpu] halt");
    }

    fn step(&mut self, n: u16) {
        self.pc += n;
        if self.pc as usize >= self.memory[0..].len() {
            self.halt();
        }
    }

    fn fetch(&self) -> u16 {
        let addr = self.pc as usize;
        let x = self.memory[addr + 0] as usize;
        let y = self.memory[addr + 1] as usize;
        let opcode = (x << 8) | y;
        opcode as u16
    }

    fn decode(&self, opcode: u16) -> fn(&mut Cpu, u16) {
        log!("[decode] {:04x}", opcode);
        let op = match opcode {
            0x00e0 => Cpu::cls,
            0x00ee => Cpu::ret,
            0x00fd => Cpu::exit,
            0x00fe => Cpu::nop,
            0x00ff => Cpu::nop,
            _ => Cpu::nop
        };
        let op = match opcode & 0xf000 {
            0x0000 => Cpu::sys,
            0x1000 => Cpu::jp,
            0x2000 => Cpu::call,
            0x3000 => Cpu::se_vx_kk,
            0x4000 => Cpu::sne_vx_kk,
            0x6000 => Cpu::ld_vx_kk,
            0xa000 => Cpu::ld,
            0xd000 => Cpu::drw,
            0xf000 => Cpu::ld_vx_i,
            0xa000 => Cpu::nop,
            0xb000 => Cpu::nop,
            0xc000 => Cpu::nop,
            0xd000 => Cpu::nop,
            _ => op
        };
        let op = match opcode & 0xf00f {
            0x5000 => Cpu::se_vx_vy,
            0x8000 => Cpu::ld_vx_vy,
            0x8001 => Cpu::or,
            0x8002 => Cpu::and,
            0x8003 => Cpu::xor,
            0x8004 => Cpu::add_vx_vy,
            0x8005 => Cpu::sub_vx_vy,
            0x8006 => Cpu::shr,
            0x8007 => Cpu::subn,
            0x800e => Cpu::shl,
            0x9000 => Cpu::sne_vx_vy,
            _ => op
        };
        let op = match opcode & 0xf0ff {
            0xf01e => Cpu::add_i_vx,
            0xe0a1 => Cpu::nop,
            0xf007 => Cpu::nop,
            0xf00a => Cpu::nop,
            0xf015 => Cpu::nop,
            0xf018 => Cpu::nop,
            0xf01e => Cpu::nop,
            0xf029 => Cpu::nop,
            0xf033 => Cpu::nop,
            0xf055 => Cpu::nop,
            0xf065 => Cpu::nop,
            _ => op
        };
        op
    }

    fn ret(&mut self, _opcode: u16) {
        self.step(2);
        log!("ret");
    }

    fn exit(&mut self, _opcode: u16) {
        self.halt();
    }

    fn add_vx_kk(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let kk = mask(opcode, 0x00ff) as u8;
        self.v[vx] = self.v[vx].wrapping_add(kk);
        self.step(2);
        log!("add v{:x}, {:02x}", vx, kk);
    }

    fn add_i_vx(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let v = self.v[vx as usize];
        self.i = self.i.wrapping_add(v as u16);
        self.step(2);
        log!("add i, v{:x}", v);
    }
    
    fn nop(&mut self, opcode: u16) {
        self.step(2);
        log!("nop");
    }

    fn sys(&mut self, opcode: u16) {
        self.step(2);
        log!("sys");
    }

    fn cls(&mut self, opcode: u16) {
        self.step(2);
        log!("cls");
    }

    fn call(&mut self, opcode: u16) {
        let addr = opcode & 0x0fff;
        self.pc = addr;
        log!("call {:#03x}", addr);
    }

    fn jp(&mut self, opcode: u16) {
        let addr = opcode & 0x0fff;
        self.pc = addr;
        log!("jp {:#03x}", addr);
    }

    fn se_vx_kk(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let kk = mask(opcode, 0x00ff) as u8;
        if self.v[vx] == kk {
            self.step(2);
        }
        self.step(2);
    }

    fn ld_vx_vy(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vy];
        self.step(2);
        log!("ld v{:x}, v{:x}", vx, vy);
    }

    fn or(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vx] | self.v[vy];
        self.step(2);
        log!("or v{:x}, v{:x}", vx, vy);
    }

    fn and(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vx] & self.v[vy];
        self.step(2);
        log!("and v{:x}, v{:x}", vx, vy);
    }

    fn xor(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vx] ^ self.v[vy];
        self.step(2);
        log!("xor v{:x}, v{:x}", vx, vy);
    }

    fn add_vx_vy(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        let sum = (self.v[vx] as usize) + (self.v[vy] as usize);
        self.v[0xf] = if sum > 0xff { 1 } else { 0 };
        self.v[vx] = sum as u8;
        self.step(2);
        log!("add v{:x}, v{:x}", vx, vy);
    }

    fn sub_vx_vy(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[0xf] = if self.v[vx] > self.v[vy] { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_sub(self.v[vy]);
        self.step(2);
        log!("sub v{:x}, v{:x}", vx, vy);
    }

    fn shr(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[0xf] = if lsb(self.v[vx]) == 0x001 { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_shr(1);
        self.step(2);
        log!("shr v{:x}", vx);
    }

    fn subn(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[0xf] = if self.v[vy] > self.v[vx] { 1 } else { 0 };
        self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
        self.step(2);
        log!("subn v{:x}, v{:x}", vx, vy);
    }

    fn shl(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        self.v[0xf] = if msb(self.v[vx]) == 0x80 { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_shl(1);
        self.step(2);
        log!("shl v{:x}", vx);
    }

    fn sne_vx_vy(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        if (self.v[vx] != self.v[vy]) {
            self.step(2);
        }
        self.step(2);
        log!("sne v{:x}, v{:x}", vx, vy);
    }

    fn se_vx_vy(&mut self, opcode: u16) {
        let vx = mask(opcode, 0x0f00) >> 8;
        let vy = mask(opcode, 0x00f0) >> 4;
        if self.v[vx] == self.v[vy] {
            self.step(2);
        }
        self.step(2);
        log!("se v{:x}, v{:x}", vx, vy);
    }

    fn sne_vx_kk(&mut self, opcode: u16) {
        let vx = ((opcode & 0x0f00) >> 8) as usize;
        let kk = (opcode & 0x00ff) as u8;
        if self.v[vx] != kk {
            self.step(2);
        }
        self.step(2);
        log!("sne v{:x}, {:02x}", vx, kk);
    }

    fn ld(&mut self, opcode: u16) {
        let addr = opcode & 0x0fff;
        self.i = addr;
        self.step(2);
        log!("ld i, {:#03x}", addr);
    }

    fn ld_vx_i(&mut self, opcode: u16) {
        let x = opcode & 0x0f00 >> 8;
        let len = x as usize;
        let mut v = &mut self.v[0..len];
        v.write(&self.memory[0..len]).unwrap();
        self.step(2);
        log!("ld v{:x}, i", x);
    }

    fn ld_vx_kk(&mut self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let kk = opcode & 0xff;
        self.v[x as usize] = kk as u8;
        self.step(2);
        log!("ld v{:x}, {:#02x}", x, kk);
    }

    fn drw(&mut self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = (opcode & 0x00f0) >> 4;
        let n = opcode & 0x000f;
        self.step(2);
        log!("drw v{:x}, v{:x}, {:#02x}", x, y, n);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nop() {
        let mut cpu = Cpu::new();
        cpu.nop(0x0000);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.halted, false);
        assert_eq!(cpu.v, [0; 16]);
        assert_eq!(cpu.i, 0);
    }

    #[test]
    fn cls() {
        let mut cpu = Cpu::new();
        cpu.cls(0x0123);
        assert_eq!(cpu.pc, 2);
        assert!(false);
    }

    #[test]
    fn ret() {
        let mut cpu = Cpu::new();
        cpu.ret(0x0000);
        assert_eq!(cpu.pc, 2);
        assert!(false);
    }

    #[test]
    fn sys() {
        let mut cpu = Cpu::new();
        cpu.sys(0x0000);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.halted, false);
        assert_eq!(cpu.v, [0; 16]);
        assert_eq!(cpu.i, 0);
    }

    #[test]
    fn jp() {
        let mut cpu = Cpu::new();
        cpu.jp(0x7fff);
        assert_eq!(cpu.pc, 0x0fff);
    }

    #[test]
    fn call() {
        let mut cpu = Cpu::new();
        cpu.call(0x12);
        assert_eq!(cpu.pc, 0x12);
        assert!(false);
    }

    #[test]
    fn se_vx_kk() {
        let mut cpu = Cpu::new();
        cpu.v[0x1] = 0x12;
        cpu.se_vx_kk(0x3112);
        assert_eq!(cpu.pc, 4);
        cpu.reset();
        cpu.v[0x1] = 0x12;
        cpu.se_vx_kk(0x0100);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn sne_vx_kk() {
        let mut cpu = Cpu::new();
        cpu.v[0x1] = 0x12;
        cpu.sne_vx_kk(0x0113);
        assert_eq!(cpu.pc, 4);
        cpu.reset();
        cpu.v[0x1] = 0x13;
        cpu.sne_vx_kk(0x0113);
        assert_eq!(cpu.pc, 2);
    }
    
    #[test]
    fn se_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[0x0] = 0x12;
        cpu.v[0x1] = 0x12;
        cpu.se_vx_vy(0x5010);
        assert_eq!(cpu.pc, 4);
        cpu.reset();
        cpu.v[0x0] = 0x01;
        cpu.v[0x1] = 0x02;
        cpu.se_vx_vy(0x5010);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn ld_vx_kk() {
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.ld_vx_kk(0x001f);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.v[0], 0x1f);
    }

    #[test]
    fn add_vx_kk() {
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.v[0] = 0x1f;
        cpu.add_vx_kk(0x7020);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.v[0], 0x3f);
        cpu.reset();
        cpu.v[0x0] = 0xff;
        cpu.add_vx_kk(0x7002);
        assert_eq!(cpu.v[0x0], 1);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn ld_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 10;
        cpu.v[1] = 20;
        cpu.ld_vx_vy(0x8010);
        assert_eq!(cpu.v[0], 20);
        assert_eq!(cpu.v[1], 20);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn or() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 0x1f;
        cpu.v[1] = 0xf1;
        cpu.or(0x8011);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.v[0], 0xff);
    }

    #[test]
    fn and() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 0x1f;
        cpu.v[1] = 0x1f;
        cpu.and(0x8012);
        assert_eq!(cpu.v[0], 0x1f);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn xor() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 0x1f;
        cpu.v[1] = 0x20;
        cpu.xor(0x8013);
        assert_eq!(cpu.v[0], 0x3f);
        assert_eq!(cpu.pc, 2);
    }

     #[test]
    fn add_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[0x0] = 250;
        cpu.v[0x1] = 10;
        cpu.add_vx_vy(0x8014);
        assert_eq!(cpu.v[0xf], 1);
        assert_eq!(cpu.v[0x0], 4);
        assert_eq!(cpu.pc, 2);
        cpu.reset();
        cpu.v[0x0] = 100;
        cpu.v[0x1] = 28;
        cpu.add_vx_vy(0x8014);
        assert_eq!(cpu.v[0xf], 0);
        assert_eq!(cpu.v[0x0], 128);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn sub_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v[0x0] = 100;
        cpu.v[0x1] = 20;
        cpu.sub_vx_vy(0x8015);
        assert_eq!(cpu.v[0x0], 80);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn shr() {
        let mut cpu = Cpu::new();
        cpu.v[0x2] = 8;
        cpu.shr(0x8206);
        assert_eq!(cpu.v[0xf], 0);
        assert_eq!(cpu.v[0x2], 4);
        assert_eq!(cpu.pc, 2);
        cpu.reset();
        cpu.v[0x2] = 7;
        cpu.shr(0x8206);
        assert_eq!(cpu.v[0xf], 1);
        assert_eq!(cpu.v[0x2], 3);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn subn() {
        let mut cpu = Cpu::new();
        cpu.v[0x0] = 10;
        cpu.v[0x1] = 20;
        cpu.subn(0x8017);
        assert_eq!(cpu.v[0xf], 1);
        assert_eq!(cpu.v[0x0], 10);
        assert_eq!(cpu.pc, 2);
        cpu.reset();
        cpu.v[0x0] = 10;
        cpu.v[0x1] = 5;
        cpu.subn(0x8017);
        assert_eq!(cpu.v[0xf], 0);
        assert_eq!(cpu.v[0x0], 251);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn shl() {
        let mut cpu = Cpu::new();
        cpu.v[0x2] = 8;
        cpu.shl(0x820e);
        assert_eq!(cpu.v[0xf], 0);
        assert_eq!(cpu.v[0x2], 16);
        assert_eq!(cpu.pc, 2);
        cpu.reset();
        cpu.v[0x2] = 0x80;
        cpu.shl(0x820e);
        assert_eq!(cpu.v[0xf], 1);
        assert_eq!(cpu.pc, 2);
    }
    
    #[test]
    fn sne_vx_vy() {
        assert!(false);
    }

    #[test]
    fn ld_i_addr() {
        assert!(false);
    }

    #[test]
    fn jp_v0_addr() {
        assert!(false);
    }

    #[test]
    fn rnd() {
        assert!(false);
    }

    #[test]
    fn drw() {
        let mut cpu = Cpu::new();
        cpu.drw(0x0000);
        assert!(false);
    }

    #[test]
    fn skp() {
        assert!(false);
    }

    #[test]
    fn sknp() {
        assert!(false);
    }

    #[test]
    fn ld_vx_dt() {
        assert!(false);
    }

    #[test]
    fn ld_vx_k() {
        assert!(false);
    }

    #[test]
    fn ld_dt_vx() {
        assert!(false);
    }

    #[test]
    fn ld_st_vx() {
        assert!(false);
    }

    #[test]
    fn add_i_vx() {
        let mut cpu = Cpu::new();
        cpu.v[0x0f] = 10;
        cpu.i = 1;
        cpu.add_i_vx(0x0f00);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.i, 11);
        cpu.i = 0xffff;
        cpu.v[0x0f] = 0xff;
        cpu.add_i_vx(0x0f00);
        assert_eq!(cpu.i, 0xffff);
    }

    #[test]
    fn ld_f_vx() {
        assert!(false);
    }

    #[test]
    fn ld_b_vx() {
        assert!(false);
    }

    #[test]
    fn ld_i_vx() {
        assert!(false);
    }

    #[test]
    fn ld_vx_i() {
        assert!(false);
    }

    #[test]
    fn scd() {
        assert!(false);
    }

    #[test]
    fn scr() {
        assert!(false);
    }

    #[test]
    fn scl() {
        assert!(false);
    }

    #[test]
    fn exit() {
        let mut cpu = Cpu::new();
        cpu.exit(0x0000);
        assert!(cpu.halted);
    }

    #[test]
    fn low() {
        assert!(false);
    }

    #[test]
    fn high() {
        assert!(false);
    }

    #[test]
    fn drw_vx_vy() {
        assert!(false);
    }

    #[test]
    fn ld_hf_vx() {
        assert!(false);
    }

    #[test]
    fn ld_r_vx() {
        assert!(false);
    }

    #[test]
    fn ld_vx_r() {
        assert!(false);
    }

}