const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // beep here
            }
            self.st -= 1;
        }
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let msb = self.ram[self.pc as usize] as u16;
        let lsb = self.ram[(self.pc + 1) as usize] as u16;
        let op = (msb << 8) | lsb;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let h1 = (op & 0xf000) >> 12;
        let h2 = (op & 0x0f00) >> 8;
        let h3 = (op & 0x00f0) >> 4;
        let h4 = op & 0x000f;

        match (h1, h2, h3, h4) {
            // nop
            (0, 0, 0, 0) => return,
            // clear screen
            (0, 0, 0xe, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // return from subroutine
            (0, 0, 0xe, 0xe) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            // jump
            (1, _, _, _) => {
                let nnn = op & 0xfff;
                self.pc = nnn;
            },
            // call subroutine
            (2, _, _, _) => {
                self.push(self.pc);
                let nnn = op & 0xfff;
                self.pc = nnn;
            },
            // skip next instruction if vx == nn
            (3, _, _, _) => {
                let x = h2 as usize;
                let nn = (op & 0xff) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            // skip next instruction if vx != vy
            (4, _, _, _) => {
                let x = h2 as usize;
                let nn = (op & 0xff) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            // 5XY0; skip next instruction if vx == vy
            (5, _, _, 0) => {
                let x = h2 as usize;
                let y = h3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            _ => unimplemented!("unimplemented opcode: {}", op),
        }
    }
}