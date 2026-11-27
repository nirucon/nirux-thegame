use super::common::*;
use super::Level;
use crate::config::Difficulty;
use crate::fb::Fb;
use crate::input::Input;
pub struct Foot { t: u32 }
impl Foot { pub fn new(_d: Difficulty, _s: u32) -> Foot { Foot { t: 0 } } }
impl Level for Foot {
    fn update(&mut self, _i: &Input, _st: &mut Stats, _o: &mut Out) -> Status { self.t += 1; if self.t > 600 { Status::Cleared } else { Status::Playing } }
    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32) { fb.clear(0); draw_hud(fb, st, hi, 1); }
    fn bot(&mut self, _st: &Stats) -> Input { Input::default() }
    fn border(&self) -> u8 { 9 }
    fn boss_active(&self) -> bool { false }
    fn progress(&self) -> f32 { 0.0 }
}
