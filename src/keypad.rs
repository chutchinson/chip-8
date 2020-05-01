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
    pub fn get(&self, key: usize) -> bool {
        self.state.get(key as u64)
    }
    pub fn set(&mut self, key: usize, state: bool) {
        self.state.set(key as u64, state);
    }
}