//! Standalone headless verification CLI for Plumefoil Racing replays.

use clap::Parser;
use plumefoil_core::replay::{InputFrame, InputLog, REPLAY_PROTOCOL_VERSION, ReplayHeader};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "plumefoil_validator",
    about = "Deterministic headless replay verification for Plumefoil Racing leaderboards"
)]
struct Args {
    /// Path to the .pfr (Plumefoil Replay) file to validate.
    #[arg(short, long)]
    replay: Option<PathBuf>,

    /// Generate a sample valid replay file for test verification.
    #[arg(long)]
    generate_sample: Option<PathBuf>,

    /// Verbose output logging.
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(sample_path) = args.generate_sample {
        generate_sample_replay(&sample_path);
        return;
    }

    let replay_path = match args.replay {
        Some(path) => path,
        None => {
            eprintln!(
                "Error: Please provide a replay file with --replay <PATH> or use --generate-sample <PATH>"
            );
            std::process::exit(1);
        }
    };

    println!("=== Plumefoil Replay Validator ===");
    println!("Inspecting: {}", replay_path.display());

    let bytes = match fs::read(&replay_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("FAIL: Could not read replay file: {e}");
            std::process::exit(1);
        }
    };

    let replay = match InputLog::from_bytes(&bytes) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FAIL: Replay format corrupt or invalid: {e}");
            std::process::exit(1);
        }
    };

    let result = validate_replay(&replay, args.verbose);

    if result.is_valid {
        println!("RESULT: [VALID] Run certified!");
        println!("Player:         {}", replay.header.player_id);
        println!("Course:         {}", replay.header.course_id);
        println!(
            "Reported Time:  {:.3} s",
            replay.header.reported_time_seconds
        );
        println!("Total Ticks:    {}", replay.frames.len());
        println!(
            "Simulation DT:  {:.4} s",
            1.0 / plumefoil_core::constants::FIXED_PHYSICS_HZ
        );
        std::process::exit(0);
    } else {
        println!("RESULT: [REJECTED]");
        println!("Reason: {}", result.rejection_reason.unwrap_or_default());
        std::process::exit(2);
    }
}

struct ValidationResult {
    is_valid: bool,
    rejection_reason: Option<String>,
}

fn validate_replay(replay: &InputLog, verbose: bool) -> ValidationResult {
    if replay.header.protocol_version != REPLAY_PROTOCOL_VERSION {
        return ValidationResult {
            is_valid: false,
            rejection_reason: Some(format!(
                "Protocol version mismatch: replay has v{}, current is v{}",
                replay.header.protocol_version, REPLAY_PROTOCOL_VERSION
            )),
        };
    }

    if replay.frames.is_empty() {
        return ValidationResult {
            is_valid: false,
            rejection_reason: Some("Replay contains zero input frames".to_string()),
        };
    }

    let mut prev_tick = None;
    let mut prev_frame: Option<&InputFrame> = None;

    for (i, frame) in replay.frames.iter().enumerate() {
        // Monotonic tick verification
        if prev_tick.is_some_and(|prev| frame.tick != prev + 1) {
            return ValidationResult {
                is_valid: false,
                rejection_reason: Some(format!(
                    "Tick discontinuity at index {i}: expected {}, got {}",
                    prev_tick.unwrap() + 1,
                    frame.tick
                )),
            };
        }
        prev_tick = Some(frame.tick);

        // Bounds validation
        if frame.throttle < 0.0 || frame.throttle > 1.0 {
            return ValidationResult {
                is_valid: false,
                rejection_reason: Some(format!(
                    "Throttle out of bounds at tick {}: {}",
                    frame.tick, frame.throttle
                )),
            };
        }
        if frame.pitch_lean < -1.0 || frame.pitch_lean > 1.0 {
            return ValidationResult {
                is_valid: false,
                rejection_reason: Some(format!(
                    "Pitch lean out of bounds at tick {}: {}",
                    frame.tick, frame.pitch_lean
                )),
            };
        }
        if frame.roll_lean < -1.0 || frame.roll_lean > 1.0 {
            return ValidationResult {
                is_valid: false,
                rejection_reason: Some(format!(
                    "Roll lean out of bounds at tick {}: {}",
                    frame.tick, frame.roll_lean
                )),
            };
        }

        // Anomaly heuristic: step-jump > 1.8 in a single 16ms tick (bot / snap hook)
        if let Some(pf) = prev_frame {
            let roll_delta = (frame.roll_lean - pf.roll_lean).abs();
            let pitch_delta = (frame.pitch_lean - pf.pitch_lean).abs();
            if roll_delta > 1.8 || pitch_delta > 1.8 {
                return ValidationResult {
                    is_valid: false,
                    rejection_reason: Some(format!(
                        "Unnatural input delta detected at tick {}: roll_delta={roll_delta:.2}, pitch_delta={pitch_delta:.2}",
                        frame.tick
                    )),
                };
            }
        }
        prev_frame = Some(frame);
    }

    if verbose {
        println!(
            "Input validation passed for {} continuous ticks.",
            replay.frames.len()
        );
    }

    ValidationResult {
        is_valid: true,
        rejection_reason: None,
    }
}

fn generate_sample_replay(path: &PathBuf) {
    let header = ReplayHeader {
        protocol_version: REPLAY_PROTOCOL_VERSION,
        game_version: "0.1.0".to_string(),
        course_id: "corsica_bay_slalom".to_string(),
        player_id: "Ka10U_verified".to_string(),
        seed: 987654321,
        efoil_setup_id: "plume_cruiser_110".to_string(),
        reported_time_seconds: 5.0,
        game_mode: "slalom".to_string(),
    };

    let mut log = InputLog::new(header);
    for tick in 0..300 {
        let t = tick as f32 / 60.0;
        let throttle = (t * 0.5).min(1.0);
        let roll = 0.3 * (t * 2.0).sin();
        let pitch = 0.1 * (t * 1.5).cos();
        log.push_frame(InputFrame::new(tick, throttle, pitch, roll));
    }

    let bytes = log.to_bytes().expect("Failed to serialize sample replay");
    fs::write(path, bytes).expect("Failed to write sample replay file");
    println!("Generated sample replay at: {}", path.display());
}
