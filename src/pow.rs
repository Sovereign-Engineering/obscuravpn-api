#[cfg(feature = "client")]
use std::ops::ControlFlow;

use argon2::{Algorithm, Argon2, Block, Params, Version};
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

const PARAMS: Params = match Params::new(16 * 1024, 1, 1, Some(32)) {
    Ok(params) => params,
    Err(_) => panic!("invalid proof-of-work argon2 parameters"),
};

#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PowDigest(#[serde_as(as = "Hex")] pub [u8; 32]);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PowParams {
    pub nonce: String,
    pub threshold: PowDigest,
    pub puzzles: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PowOutput {
    pub nonce: String,
    pub solutions: Vec<String>,
}

impl PowParams {
    #[cfg(feature = "client")]
    pub async fn solve(&self) -> Result<PowOutput, argon2::Error> {
        let mut solutions = Vec::with_capacity(self.puzzles.into());
        for puzzle in 0..self.puzzles {
            let mut candidate = 0u64;
            let solution = loop {
                let nonce = self.nonce.clone();
                let threshold = self.threshold;
                let found = tokio::task::spawn_blocking(move || solve_chunk(&nonce, threshold, puzzle, candidate))
                    .await
                    .expect("proof-of-work chunk panicked")?;
                match found {
                    ControlFlow::Break(solution) => break solution,
                    ControlFlow::Continue(next) => candidate = next,
                }
            };
            solutions.push(solution);
        }
        Ok(PowOutput {
            solutions,
            nonce: self.nonce.clone(),
        })
    }
}

impl PowOutput {
    #[must_use]
    pub fn verify(self, puzzles: u16, threshold: PowDigest) -> bool {
        let true = self.solutions.len() == usize::from(puzzles) else {
            return false;
        };
        let Ok((hasher, mut memory)) =
            prepare(&self.nonce).inspect_err(|err| tracing::warn!(message_id = "Hq2vK8tL", %err, "proof-of-work verification setup failed"))
        else {
            return false;
        };
        (0..puzzles).zip(&self.solutions).all(|(puzzle, solution)| {
            solution_hash(&hasher, &mut memory, u64::from(puzzle), solution.as_bytes())
                .inspect_err(|err| tracing::warn!(message_id = "Rj5pX1zD", %err, "proof-of-work verification hashing failed"))
                .is_ok_and(|digest| digest <= threshold)
        })
    }
}

fn prepare(nonce: &str) -> Result<(Argon2<'_>, Vec<Block>), argon2::Error> {
    let hasher = Argon2::new_with_secret(nonce.as_bytes(), Algorithm::Argon2id, Version::V0x13, PARAMS)?;
    let memory = vec![Block::default(); PARAMS.block_count()];
    Ok((hasher, memory))
}

fn solution_hash(hasher: &Argon2, memory: &mut [Block], salt: u64, solution: &[u8]) -> Result<PowDigest, argon2::Error> {
    let mut out = [0u8; 32];
    hasher.hash_password_into_with_memory(solution, &salt.to_be_bytes(), &mut out, memory)?;
    Ok(PowDigest(out))
}

#[cfg(feature = "client")]
fn solve_chunk(nonce: &str, threshold: PowDigest, puzzle: u16, start: u64) -> Result<ControlFlow<String, u64>, argon2::Error> {
    const CHUNK: u32 = 100;
    let (hasher, mut memory) = prepare(nonce)?;
    let salt = u64::from(puzzle);
    for offset in 0..CHUNK {
        let candidate = start.wrapping_add(u64::from(offset));
        let mut cand_hex = [0u8; 16];
        hex::encode_to_slice(candidate.to_be_bytes(), &mut cand_hex).unwrap();
        if solution_hash(&hasher, &mut memory, salt, &cand_hex)? <= threshold {
            return Ok(ControlFlow::Break(std::str::from_utf8(&cand_hex).unwrap().to_owned()));
        }
    }
    Ok(ControlFlow::Continue(start.wrapping_add(u64::from(CHUNK))))
}

#[cfg(all(test, feature = "client"))]
#[tokio::test]
async fn solve_then_verify() {
    let mut bytes = [0xff; 32];
    bytes[0] = 0x7f;
    let challenge = PowParams {
        nonce: "something".into(),
        threshold: PowDigest(bytes),
        puzzles: 3,
    };
    let output = challenge.solve().await.unwrap();
    assert_eq!(output.solutions.len(), 3);

    assert!(output.clone().verify(challenge.puzzles, challenge.threshold));
    assert!(output.clone().verify(challenge.puzzles, PowDigest([0xff; 32])));
    assert!(!output.clone().verify(challenge.puzzles, PowDigest([0x00; 32])));
    assert!(!output.clone().verify(challenge.puzzles + 1, challenge.threshold));
    assert!(!output.verify(challenge.puzzles - 1, challenge.threshold));
}
