use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)] pub struct DeterministicRng { state: u64 }
impl DeterministicRng { pub fn new(seed: u64) -> Self { Self { state: seed.max(1) } } pub fn next_u64(&mut self) -> u64 { let mut x = self.state; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.state = x; x } pub fn range(&mut self, upper: u64) -> u64 { if upper == 0 { 0 } else { self.next_u64() % upper } } }
pub fn floor_fraction(value: f64) -> i32 { value.floor() as i32 }
pub mod damage { pub fn split_tenacity_damage(damage: i32) -> (i32, i32) { let hp = damage / 2; (hp, damage - hp) } }
