//! Replay and input logging data structures for deterministic headless verification.

use serde::{Deserialize, Serialize};

/// Current replay protocol format version.
pub const REPLAY_PROTOCOL_VERSION: u32 = 1;

/// A single discrete tick of recorded player input.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InputFrame {
    /// Sequential physics tick index.
    pub tick: u64,
    /// Motor throttle command clamped to [0.0, 1.0].
    pub throttle: f32,
    /// Rider pitch weight shift clamped to [-1.0, 1.0] (forward = negative/nose-down, backward = positive/nose-up).
    pub pitch_lean: f32,
    /// Rider roll weight shift clamped to [-1.0, 1.0] (left/heel = -1.0, right/toe = +1.0).
    pub roll_lean: f32,
}

impl InputFrame {
    pub fn new(tick: u64, throttle: f32, pitch_lean: f32, roll_lean: f32) -> Self {
        Self {
            tick,
            throttle: throttle.clamp(0.0, 1.0),
            pitch_lean: pitch_lean.clamp(-1.0, 1.0),
            roll_lean: roll_lean.clamp(-1.0, 1.0),
        }
    }
}

/// Metadata header for an authenticated efoil run replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayHeader {
    /// Protocol version for forwards/backwards compatibility.
    pub protocol_version: u32,
    /// Game build semver string (e.g. "0.1.0").
    pub game_version: String,
    /// Identifier for the course/level (e.g. "corsica_slalom_01").
    pub course_id: String,
    /// Player identifier / public key hash.
    pub player_id: String,
    /// Deterministic PRNG seed used for wave spectrum and spawn conditions.
    pub seed: u64,
    /// Component configuration ID or serialized setup (board, mast, wings, motor).
    pub efoil_setup_id: String,
    /// Reported finish time in seconds.
    pub reported_time_seconds: f64,
    /// Reported game mode (e.g. "slalom", "tricks", "pumping").
    pub game_mode: String,
}

/// A complete recording of a player's run for anti-cheat verification and ghost playback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputLog {
    /// Replay metadata.
    pub header: ReplayHeader,
    /// Chronological stream of physics inputs.
    pub frames: Vec<InputFrame>,
}

impl InputLog {
    pub fn new(header: ReplayHeader) -> Self {
        Self {
            header,
            frames: Vec::new(),
        }
    }

    /// Appends a new input frame to the log.
    pub fn push_frame(&mut self, frame: InputFrame) {
        self.frames.push(frame);
    }

    /// Serializes the replay to a compact binary format using bincode.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes an input log from compact binary bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_frame_clamping() {
        let frame = InputFrame::new(0, 1.5, -2.0, 3.0);
        assert_eq!(frame.throttle, 1.0);
        assert_eq!(frame.pitch_lean, -1.0);
        assert_eq!(frame.roll_lean, 1.0);
    }

    #[test]
    fn test_replay_binary_roundtrip() {
        let header = ReplayHeader {
            protocol_version: REPLAY_PROTOCOL_VERSION,
            game_version: "0.1.0".to_string(),
            course_id: "norway_fjord_speedrun".to_string(),
            player_id: "Ka10U_pilot".to_string(),
            seed: 1337420,
            efoil_setup_id: "plume_race_100_mast85_wing800".to_string(),
            reported_time_seconds: 42.195,
            game_mode: "slalom".to_string(),
        };

        let mut log = InputLog::new(header);
        for tick in 0..100 {
            let t = tick as f32 / 100.0;
            log.push_frame(InputFrame::new(tick, t, 0.1 * t.sin(), -0.2 * t.cos()));
        }

        let encoded = log.to_bytes().expect("Serialization failed");
        assert!(!encoded.is_empty());

        let decoded = InputLog::from_bytes(&encoded).expect("Deserialization failed");
        assert_eq!(log, decoded);
        assert_eq!(decoded.frames.len(), 100);
    }
}
