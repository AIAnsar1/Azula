use gcd::Gcd;
use rand::Rng;
use std::convert::TryInto;



pub struct RangeIterator {
    pub(crate) active: bool,
    pub(crate) normalized_end: u32,
    pub(crate) normalized_first_pick: u32,
    pub(crate) normalized_pick: u32,
    pub(crate) actual_start: u32,
    pub(crate) step: u32,
}


impl RangeIterator {
    pub fn new(start: u32, end: u32) -> Self {
        let normalized_end = end - start + 1;
        let step = pick_random_coprime(normalized_end);
        let mut rng = rand::thread_rng();
        let normalized_first_pick = rng.gen_range(0..normalized_end);
        Self { active: true, normalized_end, step, normalized_first_pick, normalized_pick: normalized_first_pick, actual_start: start, }
    }
}


impl Iterator for RangeIterator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.active {
            return None
        }
        let current_pick = self.normalized_pick;
        let next_pick = (current_pick + self.step) % self.normalized_end;

        if next_pick == self.normalized_pick {
            self.active = false
        }
        self.normalized_pick = next_pick;
        Some((self.actual_start + current_pick).try_into().expect("Could Not Convert U32 To U16"))
    }
}



fn pick_random_coprime(end: u32) -> u32 {
    let range_boundary = end / 4;
    let lower_range = range_boundary;
    let uper_range = end - range_boundary;
    let mut rng = rand::thread_rng();
    let mut candidate = rng.gen_range(lower_range..uper_range);

    for _ in 0..10 {
        if end.gcd(candidate) == 1 {
            return candidate
        }
        candidate = rng.gen_range(lower_range..uper_range);
    }
    end - 1
}

























