use bv::BitVec;

pub struct Keypad {
    state: BitVec<u16>
}

impl Keypad {
    pub fn new() -> Self {
        Keypad {
            state: BitVec::new_fill(false, 16)
        }
    }
    pub fn get(&self, key: u64) -> bool {
        self.state.get(key)
    }
    pub fn up(&mut self, key: u64) {
        self.state.set(key, false);
    }
    pub fn down(&mut self, key: u64) {
        self.state.set(key, true);
    }
}