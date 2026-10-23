//! Robustness smoke for the shared interpreter: feeding adversarial or
//! grammar-shaped byte scripts to `interpret` must never panic (no panics,
//! no index-out-of-bounds, no unwrap on hostile input). It may only return
//! Ok(ExecutionResult) or Err. Deterministic xorshift32 so failures replay.

use neo_vm_rs::{OpCode, interpret};

struct XorShift32(u32);

impl XorShift32 {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn byte(&mut self) -> u8 {
        (self.next() & 0xff) as u8
    }
}

/// Scripts built from a small deterministic grammar that exercises real
/// execution paths (push + arithmetic/splice + ret) with random operands and
/// lengths, plus occasional malformed concat to hit fault decode paths.
fn grammar_script(rng: &mut XorShift32) -> Vec<u8> {
    let mut script = Vec::new();
    let steps = 1 + (rng.next() % 24) as usize;
    for _ in 0..steps {
        match rng.next() % 6 {
            0 => script.push(OpCode::PUSHINT8.byte()), // operand follows
            1 => script.push(OpCode::PUSHDATA1.byte()),
            2 => script.push(OpCode::PUSHT.byte()),
            3 => script.push(OpCode::PUSHF.byte()),
            4 => script.push(OpCode::ADD.byte()),
            _ => script.push(OpCode::RET.byte()),
        }
        // Fill in any operand bytes the decoder needs.
        match script.last().copied() {
            Some(op) if op == OpCode::PUSHINT8.byte() => {
                script.push(rng.byte());
                // Arithmetic opcodes often follow a push to keep the stack warm.
                if rng.next() % 2 == 0 {
                    script.push(OpCode::ADD.byte());
                }
            }
            Some(op) if op == OpCode::PUSHDATA1.byte() => {
                let len = (rng.next() % 16) as u8;
                script.push(len);
                for _ in 0..len {
                    script.push(rng.byte());
                }
            }
            _ => {}
        }
    }
    script.push(OpCode::RET.byte());
    script
}

#[test]
fn interpret_never_panics_on_bounded_random_scripts() {
    let mut rng = XorShift32(0x5eed_0b16);

    // Pure adversarial bytes: short random lengths so a stray loop burns at
    // most the shared max_instructions budget instead of making the test slow.
    for _ in 0..3_000 {
        let len = (rng.next() % 128) as usize;
        let script: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        let _ = interpret(&script);
    }

    // Grammar-shaped scripts that drive real execution paths.
    let mut halted = 0usize;
    let mut faulted = 0usize;
    for _ in 0..3_000 {
        let script = grammar_script(&mut rng);
        match interpret(&script) {
            Ok(outcome) => match outcome.state {
                neo_vm_rs::VmState::Halt => halted += 1,
                _ => faulted += 1,
            },
            Err(_) => faulted += 1,
        }
    }

    // Both outcomes represent genuinely exercised code paths, not accidents.
    assert!(
        halted > 500,
        "grammar corpus produced too few HALT runs: {halted}"
    );
    assert!(
        faulted > 500,
        "grammar corpus produced too few FAULT runs: {faulted}"
    );
    println!(
        "fuzz robustness: 6000 interpreted scripts, {halted} HALT / {faulted} FAULT/ERR, no panics"
    );
}
