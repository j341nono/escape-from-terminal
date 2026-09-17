use std::{
    f32::consts::TAU,
    time::{Duration, Instant},
};

use rodio::{OutputStream, OutputStreamBuilder, Sink, Source, source::SineWave};

use crate::monster::AiState;

const NEARBY_DISTANCE: f32 = 7.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioState {
    Normal,
    Nearby,
    Chase,
}

impl AudioState {
    pub fn for_gameplay(is_playing: bool, monster_state: AiState, distance: f32) -> Self {
        if is_playing {
            Self::from_monster(monster_state, distance)
        } else {
            Self::Normal
        }
    }

    pub fn from_monster(monster_state: AiState, distance: f32) -> Self {
        if monster_state == AiState::Chasing {
            Self::Chase
        } else if distance < NEARBY_DISTANCE {
            Self::Nearby
        } else {
            Self::Normal
        }
    }

    fn nearby_volume(self, distance: f32) -> f32 {
        if self != Self::Nearby {
            return 0.0;
        }
        let closeness = ((NEARBY_DISTANCE - distance) / NEARBY_DISTANCE).clamp(0.0, 1.0);
        0.02 + closeness * 0.035
    }
}

pub struct AudioManager {
    backend: Option<AudioBackend>,
    state: AudioState,
    started: Instant,
}

struct AudioBackend {
    _stream: OutputStream,
    ambient: Sink,
    nearby: Sink,
    chase: Sink,
}

impl AudioManager {
    pub fn new() -> Self {
        match AudioBackend::new() {
            Ok(backend) => Self {
                backend: Some(backend),
                state: AudioState::Normal,
                started: Instant::now(),
            },
            Err(error) => {
                eprintln!("Audio unavailable. Continuing without sound. ({error})");
                Self {
                    backend: None,
                    state: AudioState::Normal,
                    started: Instant::now(),
                }
            }
        }
    }

    pub fn update(&mut self, state: AudioState, distance: f32, spotted: bool) {
        if let Some(backend) = &self.backend {
            // The loop sources persist for the whole session; only their levels change.
            backend.set_levels(state, distance, self.started.elapsed());
            if self.state != state {
                self.state = state;
            }
            if spotted {
                backend.play_spotted_stinger();
            }
        }
    }
}

impl AudioBackend {
    fn new() -> Result<Self, rodio::StreamError> {
        let mut stream = OutputStreamBuilder::open_default_stream()?;
        stream.log_on_drop(false);
        let ambient = Sink::connect_new(stream.mixer());
        let nearby = Sink::connect_new(stream.mixer());
        let chase = Sink::connect_new(stream.mixer());

        ambient.append(SineWave::new(47.0).amplify(0.018).repeat_infinite());
        nearby.append(SineWave::new(62.0).repeat_infinite());
        chase.append(SineWave::new(84.0).repeat_infinite());
        nearby.set_volume(0.0);
        chase.set_volume(0.0);

        Ok(Self {
            _stream: stream,
            ambient,
            nearby,
            chase,
        })
    }

    fn set_levels(&self, state: AudioState, distance: f32, elapsed: Duration) {
        self.ambient.set_volume(0.018);
        self.nearby
            .set_volume(state.nearby_volume(distance) * pulse(elapsed, 0.85));
        self.chase.set_volume(if state == AudioState::Chase {
            0.095 * pulse(elapsed, 0.30)
        } else {
            0.0
        });
    }

    fn play_spotted_stinger(&self) {
        let stinger = Sink::connect_new(self._stream.mixer());
        stinger.append(
            SineWave::new(132.0)
                .take_duration(Duration::from_millis(180))
                .amplify(0.16),
        );
        stinger.detach();
    }
}

fn pulse(elapsed: Duration, period_seconds: f32) -> f32 {
    let phase = elapsed.as_secs_f32() * TAU / period_seconds;
    0.35 + 0.65 * phase.sin().max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_normal_nearby_and_chase_audio() {
        assert_eq!(
            AudioState::from_monster(AiState::Wandering, 12.0),
            AudioState::Normal
        );
        assert_eq!(
            AudioState::from_monster(AiState::Searching, 4.0),
            AudioState::Nearby
        );
        assert_eq!(
            AudioState::from_monster(AiState::Chasing, 12.0),
            AudioState::Chase
        );
    }

    #[test]
    fn chase_end_returns_to_distance_based_audio() {
        assert_eq!(
            AudioState::from_monster(AiState::Searching, 4.0),
            AudioState::Nearby
        );
        assert_eq!(
            AudioState::from_monster(AiState::Searching, 10.0),
            AudioState::Normal
        );
    }

    #[test]
    fn non_playing_screens_stop_chase_audio() {
        assert_eq!(
            AudioState::for_gameplay(false, AiState::Chasing, 1.0),
            AudioState::Normal
        );
    }

    #[test]
    fn nearby_volume_increases_without_exposing_exact_distance() {
        assert!(AudioState::Nearby.nearby_volume(2.0) > AudioState::Nearby.nearby_volume(6.0));
        assert_eq!(AudioState::Normal.nearby_volume(2.0), 0.0);
    }
}
