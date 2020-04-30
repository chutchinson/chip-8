use std::io::Write;
use rand::Rng;
use crate::timer::Timer;
use crate::gpu::Gpu;

static BOOTROM: &'static [u8] = &[
    0xf0, 0x90, 0x90, 0x90, 0xf0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xf0, 0x10, 0xf0, 0x80, 0xf0,
    0xf0, 0x10, 0xf0, 0x10, 0xf0,
    0x90, 0x90, 0xf0, 0x10, 0x10,
    0xf0, 0x80, 0xf0, 0x10, 0xf0,
    0xf0, 0x80, 0xf0, 0x90, 0xf0,
    0xf0, 0x10, 0x20, 0x40, 0x40,
    0xf0, 0x90, 0xf0, 0x90, 0xf0,
    0xf0, 0x90, 0xf0, 0x10, 0xf0,
    0xf0, 0x90, 0xf0, 0x90, 0x90,
    0xe0, 0x90, 0xe0, 0x90, 0xe0,
    0xf0, 0x80, 0x80, 0x80, 0xf0,
    0xe0, 0x90, 0x90, 0x90, 0xe0,
    0xf0, 0x80, 0xf0, 0x80, 0xf0,
    0xf0, 0x80, 0xf0, 0x80, 0x80
];

#[inline]
fn mask(opcode: u16, mask: u16) -> usize {
    (opcode & mask) as usize
}

#[inline]
fn addr(value: u16) -> usize {
    (value & 0x0fff).into()
}

#[inline]
fn lsb(n: u8) -> u8 {
    n & 0x01
}

#[inline]
fn msb(n: u8) -> u8 {
    n & 0x80
}

pub struct CpuContext<'a> {
    pub opcode: u16,
    pub gpu: &'a mut Gpu,
    pub sound_timer: &'a mut Timer,
    pub delay_timer: &'a mut Timer
}

impl<'a> CpuContext<'a> {
    pub fn op(&mut self, opcode: u16) -> &mut Self {
        self.opcode = opcode;
        self
    }
}

pub struct Cpu {
    halted: bool,
    memory: [u8; 4096],
    stack: [u16; 16],
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    dt: u8,
    st: u8
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
            sp: 0,
            dt: 0,
            st: 0
        }
    }

    pub fn load(&mut self, code: &[u8]) {
        let mut boot = &mut self.memory[0..0x1ff];
        match boot.write(&BOOTROM) {
            Ok(n) => { log!("loaded {} bytes into bootrom", n) },
            _ => ()
        };
        let mut mem = &mut self.memory[0x200..];
        match mem.write(&code) {
            Ok(n) => { log!("loaded {} bytes", n) },
            _ => ()
        }
    }

    pub fn reset(&mut self) {
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.dt = 0;
        self.st = 0;
    }

    pub fn cycle(&mut self, ctx: &mut CpuContext) {
        if self.halted {
            return
        }
        if self.st > 0 && ctx.sound_timer.active() {
            self.st = self.st.saturating_sub(1);
        }
        if self.dt > 0 && ctx.delay_timer.active() {
            self.dt = self.dt.saturating_sub(1);
        }
        let opcode = self.fetch();
        let op = self.decode(opcode);
        ctx.opcode = opcode;
        op(self, ctx);
    }

    pub fn halt(&mut self) {
        self.halted = true;
        log!("[cpu] halt");
    }

    pub fn dump(&self) {
        for r in 0..0x10 {
            print!("v{:x} = #{:02x} ", r, self.v[r]);
        }
        println!();
        println!("i = #{:02x}", self.i);
        println!("pc = #{:02x}", self.pc);
        println!("sp = #{:02x}", self.sp);
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

    fn decode(&self, opcode: u16) -> fn(&mut Cpu, &mut CpuContext) {
        log!("[decode] {:04x}", opcode);
        let op = match opcode {
            0x0000 => Cpu::sys,
            0x00e0 => Cpu::cls,
            0x00ee => Cpu::ret,
            0x00fd => Cpu::exit,
            0x00fe => unimplemented!(),
            0x00ff => unimplemented!(),
            _ => Cpu::nop
        };
        let op = match opcode & 0xf000 {
            0x1000 => Cpu::jp,
            0x2000 => Cpu::call,
            0x3000 => Cpu::se_vx_kk,
            0x4000 => Cpu::sne_vx_kk,
            0x6000 => Cpu::ld_vx_kk,
            0x7000 => Cpu::add_vx_kk,
            0xa000 => Cpu::ld,
            0xf000 => Cpu::ld_vx_i,
            0xb000 => Cpu::jp_v0_addr,
            0xc000 => Cpu::rnd,
            0xd000 => Cpu::drw,
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
            0xe09e => Cpu::skp,
            0xe0a1 => Cpu::sknp,
            0xf01e => Cpu::add_i_vx,
            0xf007 => Cpu::ld_vx_dt,
            0xf00a => Cpu::ld_vx_k,
            0xf015 => Cpu::ld_dt_vx,
            0xf018 => Cpu::ld_st_vx,
            0xf029 => Cpu::ld_f_vx,
            0xf033 => Cpu::ld_b_vx,
            0xf055 => Cpu::ld_i_vx,
            0xf065 => Cpu::ld_vx_i,
            _ => op
        };
        op
    }

    fn ret(&mut self, _ctx: &mut CpuContext) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
        log!("ret");
    }

    fn exit(&mut self, _ctx: &mut CpuContext) {
        self.halt();
        log!("exit");
    }

    fn nop(&mut self, _ctx: &mut CpuContext) {
        self.step(2);
        log!("nop");
    }

    fn sys(&mut self, _ctx: &mut CpuContext) {
        self.step(2);
        log!("sys");
    }

    fn cls(&mut self, ctx: &mut CpuContext) {
        ctx.gpu.clear();
        self.step(2);
        log!("cls");
    }

    fn call(&mut self, ctx: &mut CpuContext) {
        let addr = addr(ctx.opcode);
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr as u16;
        log!("call {:#03x}", addr);
    }

    fn jp(&mut self, ctx: &mut CpuContext) {
        let addr = ctx.opcode & 0x0fff;
        self.pc = addr;
        log!("jp {:#03x}", addr);
    }

    fn se_vx_kk(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let kk = mask(ctx.opcode, 0x00ff) as u8;
        if self.v[vx] == kk {
            self.step(2);
        }
        self.step(2);
        log!("se v{:x}, {:02x}", vx, kk);
    }

    fn ld_vx_kk(&mut self, ctx: &mut CpuContext) {
        let x = (ctx.opcode & 0x0f00) >> 8;
        let kk = ctx.opcode & 0xff;
        self.v[x as usize] = kk as u8;
        self.step(2);
        log!("ld v{:x}, {:#02x}", x, kk);
    }

    fn add_vx_kk(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let kk = mask(ctx.opcode, 0x00ff) as u8;
        self.v[vx] = self.v[vx].wrapping_add(kk);
        self.step(2);
        log!("add v{:x}, {:02x}", vx, kk);
    }

    fn ld_vx_vy(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vy];
        self.step(2);
        log!("ld v{:x}, v{:x}", vx, vy);
    }

    fn or(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vx] | self.v[vy];
        self.step(2);
        log!("or v{:x}, v{:x}", vx, vy);
    }

    fn and(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vx] & self.v[vy];
        self.step(2);
        log!("and v{:x}, v{:x}", vx, vy);
    }

    fn xor(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[vx] = self.v[vx] ^ self.v[vy];
        self.step(2);
        log!("xor v{:x}, v{:x}", vx, vy);
    }

    fn add_vx_vy(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        let sum = (self.v[vx] as usize) + (self.v[vy] as usize);
        self.v[0xf] = if sum > 0xff { 1 } else { 0 };
        self.v[vx] = sum as u8;
        self.step(2);
        log!("add v{:x}, v{:x}", vx, vy);
    }

    fn sub_vx_vy(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[0xf] = if self.v[vx] > self.v[vy] { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_sub(self.v[vy]);
        self.step(2);
        log!("sub v{:x}, v{:x}", vx, vy);
    }

    fn shr(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[0xf] = if lsb(self.v[vx]) == 0x001 { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_shr(1);
        self.step(2);
        log!("shr v{:x}", vx);
    }

    fn subn(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[0xf] = if self.v[vy] > self.v[vx] { 1 } else { 0 };
        self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
        self.step(2);
        log!("subn v{:x}, v{:x}", vx, vy);
    }

    fn shl(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        self.v[0xf] = if msb(self.v[vx]) == 0x80 { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_shl(1);
        self.step(2);
        log!("shl v{:x}", vx);
    }

    fn sne_vx_vy(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        if self.v[vx] != self.v[vy] {
            self.step(2);
        }
        self.step(2);
        log!("sne v{:x}, v{:x}", vx, vy);
    }

    fn se_vx_vy(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let vy = mask(ctx.opcode, 0x00f0) >> 4;
        if self.v[vx] == self.v[vy] {
            self.step(2);
        }
        self.step(2);
        log!("se v{:x}, v{:x}", vx, vy);
    }

    fn sne_vx_kk(&mut self, ctx: &mut CpuContext) {
        let vx = ((ctx.opcode & 0x0f00) >> 8) as usize;
        let kk = (ctx.opcode & 0x00ff) as u8;
        if self.v[vx] != kk {
            self.step(2);
        }
        self.step(2);
        log!("sne v{:x}, {:02x}", vx, kk);
    }

    fn ld(&mut self, ctx: &mut CpuContext) {
        let addr = ctx.opcode & 0x0fff;
        self.i = addr;
        self.step(2);
        log!("ld i, {:#03x}", addr);
    }

    fn jp_v0_addr(&mut self, ctx: &mut CpuContext) {
        let addr = ctx.opcode & 0x0fff;
        self.pc = addr.wrapping_add(self.v[0] as u16);
        log!("jp v0, {:03x}", addr);
    }

    fn rnd(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let kk = mask(ctx.opcode, 0x00ff) as u8;
        let mut rng = rand::thread_rng();
        let rnd: u8 = rng.gen();
        self.v[vx] = rnd & kk;
        self.step(2);
        log!("rnd v{:x}, {:02x}", vx, kk);
    }

    fn drw(&mut self, ctx: &mut CpuContext) {
        let x = (ctx.opcode & 0x0f00) >> 8;
        let y = (ctx.opcode & 0x00f0) >> 4;
        let n = ctx.opcode & 0x000f;
        let dx = self.v[x as usize] as u16;
        let dy = self.v[y as usize] as u16;
        self.v[0x0f] = if ctx.gpu.draw_sprite(&self.memory, self.i, n, dx, dy) { 1 } else { 0 };
        self.step(2);
        log!("drw {:x}, {:x}, {:#02x}", x, y, n);
    }

    fn skp(&mut self, ctx: &mut CpuContext) {
        unimplemented!();
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.step(2);
        log!("skp v{:x}", vx);
    }

    fn sknp(&mut self, ctx: &mut CpuContext) {
        unimplemented!();
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.step(2);
        log!("sknp v{:x}", vx);
    }

    fn ld_vx_dt(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.v[vx] = self.dt;
        self.step(2);
        log!("ld v{:x}, dt", vx);
    }

    fn ld_vx_k(&mut self, ctx: &mut CpuContext) {
        unimplemented!();
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.step(2);
        log!("ld v{:x}, k", vx);
    }

    fn ld_dt_vx(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.dt = self.v[vx];
        self.step(2);
        log!("ld dt, v{:x}", vx);
    }

    fn ld_st_vx(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.st = self.v[vx];
        self.step(2);
        log!("ld st, v{:x}", vx);
    }

    fn add_i_vx(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.i = self.i.wrapping_add(self.v[vx].into());
        self.step(2);
        log!("ld i, v{:x}", vx);
    }

    fn ld_f_vx(&mut self, ctx: &mut CpuContext) {
        // TODO: implement
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        self.i = 0;
        self.step(2);
        log!("ld f, v{:x}", vx);
    }

    fn ld_b_vx(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let loc = addr(self.i);
        let v = self.v[vx];
        self.memory[loc + 2] = v % 10;
        self.memory[loc + 1] = (v / 10) % 10;
        self.memory[loc + 0] = (v / 100) % 10;
        self.step(2);
        log!("ld b, v{:x}", vx);
    }

    fn ld_i_vx(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let mut memory = &mut self.memory[addr(self.i)..];
        let v = &self.v[0..vx];
        memory.write(v).unwrap();
        self.step(2);
        log!("ld i, v{:x}", vx);
    }

    fn ld_vx_i(&mut self, ctx: &mut CpuContext) {
        let vx = mask(ctx.opcode, 0x0f00) >> 8;
        let memory = &self.memory[addr(self.i)..];
        let mut v = &mut self.v[0..vx];
        v.write(memory).unwrap();
        self.step(2);
        log!("ld v{:x}, i", vx);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu_test<F>(exec: F) 
        where F: FnOnce(&mut Cpu, &mut CpuContext) -> () {
        let mut delay_timer = Timer::new(0);
        let mut sound_timer = Timer::new(0);
        let mut gpu = Gpu::new();
        let mut cpu = Cpu::new();
        let mut ctx = CpuContext {
            opcode: 0x0000,
            sound_timer: &mut sound_timer,
            delay_timer: &mut delay_timer,
            gpu: &mut gpu
        };
        exec(&mut cpu, &mut ctx);
    }

    #[test]
    fn nop() {
        cpu_test(|cpu, ctx| {
            ctx.opcode = 0x0000;
            cpu.nop(ctx);
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.sp, 0);
            assert_eq!(cpu.halted, false);
            assert_eq!(cpu.v, [0; 16]);
            assert_eq!(cpu.i, 0);
        });
    }

    #[test]
    fn cls() {
        cpu_test(|cpu, ctx| {
            ctx.opcode = 0x0123;
            cpu.cls(ctx);
            assert_eq!(cpu.pc, 2);
            assert!(false);
        });
    }

    #[test]
    fn ret() {
        cpu_test(|cpu, ctx| {
            ctx.opcode = 0x00ee;
            cpu.ret(ctx);
            assert_eq!(cpu.pc, 2);
            cpu.reset();
            ctx.opcode = 0x0000;
            cpu.nop(ctx);        // pc = 2
            ctx.opcode = 0x2009;
            cpu.call(ctx);       // pc = 9, sp = 1, stack = 2
            ctx.opcode = 0x00ee;
            cpu.ret(ctx);        // pc = 2, sp = 0
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.sp, 0);
        });
    }

    #[test]
    fn sys() {
        cpu_test(|cpu, ctx| {
            ctx.opcode = 0x0000;
            cpu.sys(ctx);
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.sp, 0);
            assert_eq!(cpu.halted, false);
            assert_eq!(cpu.v, [0; 16]);
            assert_eq!(cpu.i, 0);
        });
    }

    #[test]
    fn jp() {
        cpu_test(|cpu, ctx| {
            ctx.opcode = 0x7fff;
            cpu.jp(ctx);
            assert_eq!(cpu.pc, 0x0fff);
        });
    }

    #[test]
    fn call() {
        cpu_test(|cpu, ctx| {
            ctx.opcode = 0x0000;
            cpu.nop(ctx);
            ctx.opcode = 0x2117;
            cpu.call(ctx);
            assert_eq!(cpu.sp, 1);
            assert_eq!(cpu.pc, 0x117);
            assert_eq!(cpu.stack[1], 2);
        });
    }

    #[test]
    fn se_vx_kk() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x1] = 0x12;
            ctx.opcode = 0x3112;
            cpu.se_vx_kk(ctx);
            assert_eq!(cpu.pc, 4);
            cpu.reset();
            cpu.v[0x1] = 0x12;
            ctx.opcode = 0x0100;
            cpu.se_vx_kk(ctx);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn sne_vx_kk() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x1] = 0x12;
            ctx.opcode = 0x0113;
            cpu.sne_vx_kk(ctx);
            assert_eq!(cpu.pc, 4);
            cpu.reset();
            cpu.v[0x1] = 0x13;
            ctx.opcode = 0x0113;
            cpu.sne_vx_kk(ctx);
            assert_eq!(cpu.pc, 2);
        });  
    }
    
    #[test]
    fn se_vx_vy() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x0] = 0x12;
            cpu.v[0x1] = 0x12;
            ctx.opcode = 0x5010;
            cpu.se_vx_vy(ctx);
            assert_eq!(cpu.pc, 4);
            cpu.reset();
            cpu.v[0x0] = 0x01;
            cpu.v[0x1] = 0x02;
            ctx.opcode = 0x5010;
            cpu.se_vx_vy(ctx);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn ld_vx_kk() {
        cpu_test(|cpu, ctx| {
            cpu.reset();
            ctx.opcode = 0x001f;
            cpu.ld_vx_kk(ctx);
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.v[0], 0x1f);
        });
    }

    #[test]
    fn add_vx_kk() {
        cpu_test(|cpu, ctx| {
            cpu.reset();
            cpu.v[0] = 0x1f;
            ctx.opcode = 0x7020;
            cpu.add_vx_kk(ctx);
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.v[0], 0x3f);
            cpu.reset();
            cpu.v[0x0] = 0xff;
            ctx.opcode = 0x7002;
            cpu.add_vx_kk(ctx);
            assert_eq!(cpu.v[0x0], 1);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn ld_vx_vy() {
        cpu_test(|cpu, ctx| {
            cpu.v[0] = 10;
            cpu.v[1] = 20;
            ctx.opcode = 0x8010;
            cpu.ld_vx_vy(ctx);
            assert_eq!(cpu.v[0], 20);
            assert_eq!(cpu.v[1], 20);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn or() {
        cpu_test(|cpu, ctx| {
            cpu.v[0] = 0x1f;
            cpu.v[1] = 0xf1;
            ctx.opcode = 0x8011;
            cpu.or(ctx);
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.v[0], 0xff);
        });
    }

    #[test]
    fn and() {
        cpu_test(|cpu, ctx| {
            cpu.v[0] = 0x1f;
            cpu.v[1] = 0x1f;
            cpu.and(ctx.op(0x8012));
            assert_eq!(cpu.v[0], 0x1f);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn xor() {
        cpu_test(|cpu, ctx| {
            cpu.v[0] = 0x1f;
            cpu.v[1] = 0x20;
            cpu.xor(ctx.op(0x8013));
            assert_eq!(cpu.v[0], 0x3f);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn add_vx_vy() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x0] = 250;
            cpu.v[0x1] = 10;
            cpu.add_vx_vy(ctx.op(0x8014));
            assert_eq!(cpu.v[0xf], 1);
            assert_eq!(cpu.v[0x0], 4);
            assert_eq!(cpu.pc, 2);
            cpu.reset();
            cpu.v[0x0] = 100;
            cpu.v[0x1] = 28;
            cpu.add_vx_vy(ctx.op(0x8014));
            assert_eq!(cpu.v[0xf], 0);
            assert_eq!(cpu.v[0x0], 128);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn sub_vx_vy() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x0] = 100;
            cpu.v[0x1] = 20;
            cpu.sub_vx_vy(ctx.op(0x8015));
            assert_eq!(cpu.v[0x0], 80);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn shr() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x2] = 8;
            cpu.shr(ctx.op(0x8206));
            assert_eq!(cpu.v[0xf], 0);
            assert_eq!(cpu.v[0x2], 4);
            assert_eq!(cpu.pc, 2);
            cpu.reset();
            cpu.v[0x2] = 7;
            cpu.shr(ctx.op(0x8206));
            assert_eq!(cpu.v[0xf], 1);
            assert_eq!(cpu.v[0x2], 3);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn subn() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x0] = 10;
            cpu.v[0x1] = 20;
            cpu.subn(ctx.op(0x8017));
            assert_eq!(cpu.v[0xf], 1);
            assert_eq!(cpu.v[0x0], 10);
            assert_eq!(cpu.pc, 2);
            cpu.reset();
            cpu.v[0x0] = 10;
            cpu.v[0x1] = 5;
            cpu.subn(ctx.op(0x8017));
            assert_eq!(cpu.v[0xf], 0);
            assert_eq!(cpu.v[0x0], 251);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn shl() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x2] = 8;
            cpu.shl(ctx.op(0x820e));
            assert_eq!(cpu.v[0xf], 0);
            assert_eq!(cpu.v[0x2], 16);
            assert_eq!(cpu.pc, 2);
            cpu.reset();
            cpu.v[0x2] = 0x80;
            cpu.shl(ctx.op(0x820e));
            assert_eq!(cpu.v[0xf], 1);
            assert_eq!(cpu.pc, 2);
        });
    }
    
    #[test]
    fn sne_vx_vy() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x2] = 8;
            cpu.v[0x3] = 7;
            cpu.sne_vx_vy(ctx.op(0x9230));
            assert_eq!(cpu.pc, 4);
            cpu.reset();
            cpu.v[0x2] = 8;
            cpu.v[0x3] = 8;
            cpu.sne_vx_vy(ctx.op(0x9230));
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn ld() {
        cpu_test(|cpu, ctx| {
            cpu.ld(ctx.op(0xa777));
            assert_eq!(cpu.i, 0x777);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn jp_v0_addr() {
        cpu_test(|cpu, ctx| {
            cpu.v[0] = 0x32;
            cpu.jp_v0_addr(ctx.op(0xb032));
            assert_eq!(cpu.pc, 0x32 + 0x32);
        }); 
    }

    #[test]
    fn rnd() {
        cpu_test(|cpu, ctx| {
            for _ in 0..10 {
                cpu.reset();
                cpu.rnd(ctx.op(0xc1ff));
                assert_eq!(cpu.pc, 2);
                if cpu.v[0x1] != 0 {
                    return;
                }
            }
            assert!(false, "rng doesn't work on host platform");
        });    
    }

    #[test]
    fn drw() {
        cpu_test(|cpu, ctx| {
            // draw 8x5 sprite (sprite 0) at (2, 4)
            cpu.i = 0x0000;
            cpu.drw(ctx.op(0xd245)); 
            let index = 2 * ctx.gpu.width + 4;
            let addr = cpu.i as usize;
            // assert that vram matches sprite memory
            let char = &cpu.memory[addr..addr+40];
            let vram = &ctx.gpu.vram[index..index+40];
            assert_eq!(char, vram);
        });     
    }

    #[test]
    fn skp() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn sknp() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_vx_dt() {
        cpu_test(|cpu, ctx| {
            cpu.dt = 100;
            cpu.ld_vx_dt(ctx.op(0xf107));
            assert_eq!(cpu.dt, 100);
            assert_eq!(cpu.v[0x1], 100);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn ld_vx_k() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_dt_vx() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_st_vx() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn add_i_vx() {
        cpu_test(|cpu, ctx| {
            cpu.v[0x0f] = 10;
            cpu.i = 1;
            cpu.add_i_vx(ctx.op(0x0f00));
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.i, 11);
            cpu.i = 0xffff;
            cpu.v[0x0f] = 0xff;
            cpu.add_i_vx(ctx.op(0x0f00));
            assert_eq!(cpu.i, 0xffff);
        });
    }

    #[test]
    fn ld_f_vx() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_b_vx() {
        cpu_test(|cpu, ctx| {
            cpu.i = 0x10;
            cpu.v[1] = 123;
            cpu.ld_b_vx(ctx.op(0xf133));
            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.memory[0x10 + 0], 1);
            assert_eq!(cpu.memory[0x10 + 1], 2);
            assert_eq!(cpu.memory[0x10 + 2], 3);
        });
    }

    #[test]
    fn ld_i_vx() {
        cpu_test(|cpu, ctx| {
            cpu.i = 0x10;
            for i in 0x0..0xf {
                cpu.v[i] = (i * 2) as u8;
            }
            cpu.ld_i_vx(ctx.op(0xff55));
            assert_eq!(cpu.pc, 2);
            for i in 0x0..0xf {
                assert_eq!(cpu.memory[0x10 + i], (i * 2) as u8);
            }
        });
    }

    #[test]
    fn ld_vx_i() {
        cpu_test(|cpu, ctx| {
            cpu.i = 0x10;
            for i in 0x0..0xf {
                cpu.memory[cpu.i as usize + i] = (i * 2) as u8;
            }
            cpu.ld_vx_i(ctx.op(0xff65));
            assert_eq!(cpu.pc, 2);
            for i in 0x0..0xf {
                assert_eq!(cpu.v[i], (i * 2) as u8);
            }
        });
    }

    #[test]
    fn scd() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn scr() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn scl() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn exit() {
        cpu_test(|cpu, ctx| {
            cpu.exit(ctx.op(0x0000));
            assert!(cpu.halted);
        });   
    }

    #[test]
    fn low() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn high() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn drw_vx_vy() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_hf_vx() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_r_vx() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

    #[test]
    fn ld_vx_r() {
        cpu_test(|cpu, ctx| {
            assert!(false);
        });
    }

}