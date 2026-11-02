//! Sounds, all made here from square, triangle and noise waves: no sound
//! files. Quiet ticks for steps, a chime for stepping through to another
//! slice, a thud at walls, a sweep when the maze turns.

use rodio::buffer::SamplesBuffer;
use rodio::{MixerDeviceSink, Source};
use std::num::NonZero;

const RATE: u32 = 44_100;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sound {
    /// a step inside the slice
    Step,
    /// a step through to the slices beside (true) or above and below
    Through(bool),
    Wall,
    Turn,
    Hint,
    /// solved; true for a new best
    Solved(bool),
    Menu,
}

pub struct Audio {
    sink: Option<MixerDeviceSink>,
    pub muted: bool,
}

#[derive(Clone, Copy)]
enum Wave {
    Square,
    Triangle,
    Noise,
}

/// One note: frequency sweeps from `f0` to `f1` over `secs`, with a short
/// attack and a linear decay.
fn note(out: &mut Vec<f32>, wave: Wave, f0: f32, f1: f32, secs: f32, vol: f32) {
    let n = (secs * RATE as f32) as usize;
    let mut phase = 0.0f32;
    let mut seed = 0x9e37_79b9u32;
    for i in 0..n {
        let k = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * k;
        phase = (phase + f / RATE as f32).fract();
        let s = match wave {
            Wave::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
            Wave::Noise => {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed as f32 / u32::MAX as f32) * 2.0 - 1.0
            }
        };
        let attack = (i as f32 / (RATE as f32 * 0.003)).min(1.0);
        out.push(s * vol * attack * (1.0 - k));
    }
}

/// The samples for a sound (also what the tests look at).
pub fn samples(s: Sound) -> Vec<f32> {
    let mut v = Vec::new();
    match s {
        Sound::Step => note(&mut v, Wave::Triangle, 300.0, 260.0, 0.035, 0.16),
        Sound::Through(across) => {
            // cyan across is the higher pair, amber up and down the lower
            let (a, b) = if across { (660.0, 990.0) } else { (440.0, 660.0) };
            note(&mut v, Wave::Triangle, a, a, 0.05, 0.28);
            note(&mut v, Wave::Triangle, b, b * 1.01, 0.11, 0.24);
        }
        Sound::Wall => note(&mut v, Wave::Noise, 0.0, 0.0, 0.05, 0.12),
        Sound::Turn => {
            note(&mut v, Wave::Triangle, 220.0, 880.0, 0.12, 0.2);
            note(&mut v, Wave::Triangle, 880.0, 660.0, 0.08, 0.14);
        }
        Sound::Hint => {
            for f in [784.0, 988.0, 1175.0] {
                note(&mut v, Wave::Square, f, f, 0.05, 0.08);
            }
        }
        Sound::Solved(best) => {
            for f in [523.0, 659.0, 784.0, 1047.0] {
                note(&mut v, Wave::Triangle, f, f, 0.1, 0.3);
            }
            if best {
                for f in [1175.0, 1319.0, 1568.0] {
                    note(&mut v, Wave::Square, f, f, 0.07, 0.1);
                }
            }
            note(&mut v, Wave::Triangle, 1047.0, 1047.0, 0.4, 0.3);
        }
        Sound::Menu => note(&mut v, Wave::Square, 880.0, 880.0, 0.02, 0.08),
    }
    v
}

impl Audio {
    pub fn new(muted: bool) -> Audio {
        let sink = match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(mut s) => {
                s.log_on_drop(false);
                Some(s)
            }
            Err(e) => {
                log::warn!("no audio: {e}");
                None
            }
        };
        Audio { sink, muted }
    }

    /// No device at all, for tests.
    #[cfg(test)]
    pub fn silent() -> Audio {
        Audio { sink: None, muted: true }
    }

    pub fn available(&self) -> bool {
        self.sink.is_some()
    }

    pub fn play(&self, s: Sound) {
        if self.muted {
            return;
        }
        let Some(sink) = &self.sink else { return };
        let buf = SamplesBuffer::new(NonZero::new(1).unwrap(), NonZero::new(RATE).unwrap(), samples(s));
        sink.mixer().add(buf.amplify(0.8));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_is_short_and_in_range() {
        for s in [Sound::Step, Sound::Through(true), Sound::Through(false), Sound::Wall, Sound::Turn, Sound::Hint, Sound::Solved(false), Sound::Solved(true), Sound::Menu] {
            let v = samples(s);
            assert!(!v.is_empty(), "{s:?} is silent");
            assert!(v.len() < RATE as usize * 2, "{s:?} is longer than two seconds");
            assert!(v.iter().all(|x| x.abs() <= 1.0), "{s:?} clips");
        }
    }

    #[test]
    fn steps_are_quieter_than_chimes() {
        let peak = |s| samples(s).iter().fold(0.0f32, |m, x| m.max(x.abs()));
        assert!(peak(Sound::Step) < peak(Sound::Through(true)));
    }
}
