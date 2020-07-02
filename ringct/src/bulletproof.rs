// Needed because most cryptographic code relies on non snake case names and single letter variables
#![allow(non_snake_case)]
#![allow(clippy::many_single_char_names)]

//! # Bulletproofs
//!
//! Zero knowledge range proofs

use std::borrow::Borrow;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crypto::{
    curve25519_dalek::traits::{Identity, IsIdentity, MultiscalarMul},
    ecc::{hash_to_point, hash_to_scalar, Point, Scalar, BASEPOINT_TABLE},
    CNFastHash, Digest,
};

use crate::{Commitment, AMOUNT_BASEPOINT, AMOUNT_BASEPOINT_TABLE};

/// Errors returned by Bulletproof operations
#[derive(Fail, Debug)]
pub enum Error {
    /// Returned when the number of values to be proved in a proof is more than M_MAX
    #[fail(display = "Too many values to be proven")]
    TooManyValues,

    /// Returned when the challenge scalars are not reduced
    #[fail(display = "Input scalars not reduced")]
    ScalarsNotReduced,

    /// Returned when the proof is empty
    #[fail(display = "Proof is empty")]
    EmptyProof,

    /// Returned when the proof does not contain any commitments
    #[fail(display = "Proof has no commitments")]
    NoCommitments,

    /// Returned when the proof is inconsistent
    #[fail(display = "Proof is inconsistent")]
    InconsistentProof,

    /// Returned when the proof is too large
    #[fail(display = "Proof is too large")]
    TooLargeProof,

    /// Returned when the proof fails a check
    #[fail(display = "Proof failed a check")]
    InvalidProof,
}

/// A bulletproof
///
/// Contains commitments to proved values and responses to challenges based on those values
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bulletproof {
    /// Commitments to proved values
    pub V: Vec<Point>,
    pub A: Point,
    pub S: Point,
    pub T_1: Point,
    pub T_2: Point,
    pub tau_x: Scalar,
    pub mu: Scalar,
    pub L: Vec<Point>,
    pub R: Vec<Point>,
    pub a: Scalar,
    pub b: Scalar,
    pub t: Scalar,
}

/// Maximum number of bits `N` of the value
///
/// The input value is then proved to be within `[0,2^n]`
const N_BITS: usize = 64;

/// Maximum number of values proved by a given bulletproof
const M_MAX: usize = 16;

/// Maintains the non-interactive transcript of messages used
/// in the zero knowledge proof
struct Transcript {
    hasher: CNFastHash,
    value: Scalar,
}

impl Transcript {
    /// Create a new transcript with the given value
    pub fn new(initial_value: Scalar) -> Transcript {
        let mut t = Transcript {
            hasher: CNFastHash::new(),
            value: initial_value,
        };
        // Prefill the hasher with the current transcript value
        t.hasher.input(t.value.as_bytes());
        t
    }

    /// Append the given curve points to the transcript
    pub fn extend_with_points(&mut self, points: &[Point]) {
        points
            .iter()
            .for_each(|point| self.hasher.input(point.compress().as_bytes()))
    }

    /// Append the given scalars to the transcript
    pub fn extend_with_scalars(&mut self, scalars: &[Scalar]) {
        scalars
            .iter()
            .for_each(|scalar| self.hasher.input(scalar.as_bytes()))
    }

    /// Reset the transcript with the given value
    pub fn reset_state(&mut self, value: Scalar) {
        self.hasher.reset();
        self.value = value;
        self.hasher.input(self.value.as_bytes());
    }

    /// Get the next challenge value from the transcript
    pub fn get_current_state(&mut self) -> Scalar {
        self.value = hash_to_scalar(self.hasher.result_reset());
        self.hasher.input(self.value.as_bytes());
        self.value
    }
}

/// Generates a vector of powers of the given Scalar upto n
///
/// `[1, a, a^2, ..., a^n]`
fn power_vector(a: Scalar, n: usize) -> Vec<Scalar> {
    let mut vec = Vec::with_capacity(n);

    if n == 0 {
        return vec;
    }
    vec.push(Scalar::one());
    if n == 1 {
        return vec;
    }
    vec.push(a);

    for i in 2..n {
        vec.push(vec[i - 1] * a);
    }

    vec
}

/// Gets the sum of all powers of the given Scalar upto n
pub fn power_sum(a: Scalar, n: usize) -> Scalar {
    if n == 0 {
        return Scalar::zero();
    }

    let mut acc = Scalar::one();
    if n == 1 {
        return acc;
    }

    let mut prev = a;
    for i in 1..n {
        if i > 1 {
            prev *= a;
        }
        acc += prev;
    }

    acc
}

/// Computes the inner product of the given Scalar arrays
///
/// `IP = a_1*b_1 + a_2*b_2 + ... + a_n*b_n`
fn inner_product(
    a: impl IntoIterator<Item = impl Borrow<Scalar>>,
    b: impl IntoIterator<Item = impl Borrow<Scalar>>,
) -> Scalar {
    a.into_iter().zip(b).fold(Scalar::zero(), |sum, (a, b)| {
        sum + (a.borrow() * b.borrow())
    })
}

/// Returns a basepoint derived from the given basepoint and index
fn get_power(base: Point, index: u64) -> Point {
    let mut hasher = CNFastHash::new();

    hasher.input(base.compress().as_bytes());
    hasher.input(b"bulletproof");
    hasher.input(varint::serialize(index));

    let hash = hasher.result();
    // Double hash
    hash_to_point(CNFastHash::digest(&hash))
}

lazy_static! {
    /// Inverse of 8
    static ref INV_EIGHT: Scalar = Scalar::from(8u64).invert();

    /// Power vector of the Scalar 2
    static ref TWO_POWERS: Vec<Scalar> = power_vector(Scalar::from(2u64), N_BITS);

    /// Inner product of the power vectors of 1 and 2
    static ref ONE_TWO_INNER_PRODUCT: Scalar = inner_product((0..N_BITS).map(|_| Scalar::one()), TWO_POWERS.iter());

    /// Commitment basepoints Hi
    static ref H_I: Vec<Point> = (0..(N_BITS * M_MAX))
        .map(|i| 2 * i)
        .map(|i| get_power(*AMOUNT_BASEPOINT, i as u64))
        .collect();

    /// Commitment basepoints Gi
    static ref G_I: Vec<Point> = (0..(N_BITS * M_MAX))
        .map(|i| (2 * i) + 1)
        .map(|i| get_power(*AMOUNT_BASEPOINT, i as u64))
        .collect();
}

/// Prove the given values and Scalars to be within [0, 2^64-1] using a Bulletproof
///
/// # Returns
/// 1. A bulletproof that proves the given values to be within the 64-bit range
/// 2. A set of mask values used in the commitments
pub fn prove_multiple(values: &[u64]) -> Result<(Bulletproof, Vec<Scalar>), Error> {
    // Make sure we're not proving too many values
    if values.len() > M_MAX {
        return Err(Error::TooManyValues);
    }

    // Number of values to be proved
    let mut M = 0;
    // Find log2(M)
    let mut logM = 0;
    while (M < values.len()) && (M <= M_MAX) {
        logM += 1;
        M = 1 << logM;
    }

    let MN = N_BITS * M;

    // Keep the current length of values before converting to an iterator
    let current_length = values.len();
    // Convert the values to `Scalar`s
    let value_mask_pairs = values
        .iter()
        .map(
            |value| (Scalar::from(*value), Scalar::random(&mut rand::rngs::OsRng)), // NOTE: This collect is needed because we generate random masks here. If
                                                                                    // we did not collect, each cloned iterator gets a different mask value
        )
        .collect::<Vec<_>>();

    // Compute the value commitments
    let V = value_mask_pairs
        .iter()
        .map(|(value, mask)| {
            // V = mG + vH
            (Commitment {
                mask: *mask,
                value: *value,
            })
            .into_public()
        })
        .map(|V| *INV_EIGHT * V);

    // Extend the given values to match the next power of 2 at M
    // and decompose each value into its binary digits
    //
    // aL contains the binary representation
    let bin_decomp = value_mask_pairs
        .iter()
        .map(|(value, _)| *value)
        .chain((current_length..M).map(|_| Scalar::zero()))
        .flat_map(|value| {
            let mut v = [0; N_BITS];
            for (n, byte) in value.as_bytes().iter().take(N_BITS / 8).enumerate() {
                for i in 0..8 {
                    v[(n * 8) + i] = if (byte & (1 << i)) == 0 { 0u8 } else { 1u8 };
                }
            }
            v.to_vec()
        })
        .collect::<Vec<_>>();

    let aL = bin_decomp.iter().map(|x| Scalar::from(*x as u64));

    // aR is aL - 1 (scalar arithmetic)
    let aR = aL.clone().map(|val| val - Scalar::one());

    // Repeat in a loop if it so happens that the challenges equal zero
    loop {
        let mut hasher = CNFastHash::new();

        // Begin the transcript with a hash of all value commitments
        V.clone()
            .for_each(|x| hasher.input(x.compress().to_bytes()));
        let mut transcript = Transcript::new(hash_to_scalar(hasher.result()));

        // Generate the blinded Pedersen Commitments to aL and aR
        // alpha & A
        let alpha = Scalar::random(&mut rand::rngs::OsRng);
        let vec_exp = Point::multiscalar_mul(
            aL.clone().interleave(aR.clone()),
            G_I.iter().interleave(H_I.iter()).take(2 * bin_decomp.len()),
        );

        // A = VE + alphaG
        // Inverse 8 to adjust for cofactor-8
        let A = *INV_EIGHT * (vec_exp + (&alpha * &BASEPOINT_TABLE));

        // S
        let sL = (0..MN)
            .map(|_| Scalar::random(&mut rand::rngs::OsRng))
            .collect::<Vec<_>>()
            .into_iter();
        let sR = (0..MN)
            .map(|_| Scalar::random(&mut rand::rngs::OsRng))
            .collect::<Vec<_>>()
            .into_iter();
        let rho = Scalar::random(&mut rand::rngs::OsRng);
        let vec_exp = Point::multiscalar_mul(
            sL.clone().interleave(sR.clone()),
            G_I.iter().interleave(H_I.iter()).take(2 * MN),
        );

        // A = VE + rhoG
        // Inverse 8 to adjust for cofactor-8
        let S = *INV_EIGHT * (vec_exp + (&rho * &BASEPOINT_TABLE));

        // Compute the challenges y, z
        transcript.extend_with_points(&[A, S]);
        let y = transcript.get_current_state();
        if y == Scalar::zero() {
            println!("y is 0, retrying...");
            continue;
        }

        let z = hash_to_scalar(CNFastHash::digest(y.as_bytes()));
        if z == Scalar::zero() {
            println!("z is 0, retrying...");
            continue;
        }
        transcript.reset_state(z);

        // Polynomial Construction
        let l_0 = aL.clone().map(|aL| aL - z);
        let l_1 = sL;

        let z_pow = power_vector(z, M + 2);

        let mut zero_twos = (0..(MN)).map(|_| Scalar::zero()).collect::<Vec<_>>();
        for i in 0..(MN) {
            for j in 1..=M {
                if (i >= (j - 1) * N_BITS) && (i < (j * N_BITS)) {
                    // TODO: Add assertions, replace with an iterator chained version
                    // zt += (z^n) * 2^n
                    zero_twos[i] += z_pow[j + 1] * TWO_POWERS[i - (j - 1) * N_BITS];
                }
            }
        }

        let y_pow = power_vector(y, MN);
        let r_0 = aR
            .clone()
            // aR[i] + z
            .map(|a| a + z)
            // Hadamard(r0, y_pow)
            .zip(&y_pow)
            .map(|(r, y)| r * y)
            // r0 + zero_twos
            .zip(zero_twos)
            .map(|(r, zT)| r + zT);

        // Hadamard(yMN, sR)
        let r_1 = sR.zip(&y_pow).map(|(s, y)| s * y);

        let t_1 = inner_product(l_0.clone(), r_1.clone()) + inner_product(l_1.clone(), r_0.clone());
        let t_2 = inner_product(l_1.clone(), r_1.clone());

        let tau_1 = Scalar::random(&mut rand::rngs::OsRng);
        let tau_2 = Scalar::random(&mut rand::rngs::OsRng);

        let T_1 = ((&tau_1 * &BASEPOINT_TABLE) + (&t_1 * &*AMOUNT_BASEPOINT_TABLE)) * *INV_EIGHT;
        let T_2 = ((&tau_2 * &BASEPOINT_TABLE) + (&t_2 * &*AMOUNT_BASEPOINT_TABLE)) * *INV_EIGHT;

        transcript.extend_with_scalars(&[z]);
        transcript.extend_with_points(&[T_1, T_2]);

        let x = transcript.get_current_state();
        if x == Scalar::zero() {
            println!("x is 0, retrying...");
            continue;
        }

        let tau_x = value_mask_pairs
            .iter()
            .zip(&z_pow[2..])
            .fold((tau_1 * x) + (tau_2 * x * x), |tau_x, ((_, mask), z_n)| {
                tau_x + (z_n * mask)
            });

        let mu = alpha + (x * rho);

        let l = l_0.clone().zip(l_1).map(|(l, l_1)| l + (l_1 * x));
        let r = r_0.clone().zip(r_1).map(|(r, r_1)| r + (r_1 * x));

        let t = inner_product(l.clone(), r.clone());

        transcript.extend_with_scalars(&[x, tau_x, mu, t]);
        let x_ip = transcript.get_current_state();
        if x_ip == Scalar::zero() {
            println!("x_ip is 0, retrying...");
            continue;
        }

        let mut n_prime = MN;
        let y_inv_pow = power_vector(y.invert(), n_prime);

        let mut a_prime = l.collect::<Vec<_>>();
        let mut b_prime = r.collect::<Vec<_>>();

        let mut G_prime = G_I[..MN].to_vec();
        let mut H_prime = H_I
            .iter()
            .zip(y_inv_pow)
            .map(|(h_p, y_inv_pow)| h_p * y_inv_pow)
            .take(MN)
            .collect::<Vec<_>>();

        let mut L = Vec::new();
        let mut R = Vec::new();

        let mut w = Vec::new();

        while n_prime > 1 {
            n_prime /= 2;

            let c_L = inner_product(&a_prime[..n_prime], &b_prime[n_prime..]);
            let c_R = inner_product(&a_prime[n_prime..], &b_prime[..n_prime]);

            let L_i = Point::multiscalar_mul(
                a_prime[..n_prime]
                    .iter()
                    .interleave(b_prime[n_prime..].iter()),
                G_prime[n_prime..]
                    .iter()
                    .interleave(H_prime[..n_prime].iter()),
            );
            let L_i = (L_i + (&(c_L * x_ip) * &*AMOUNT_BASEPOINT_TABLE)) * *INV_EIGHT;

            let R_i = Point::multiscalar_mul(
                a_prime[n_prime..]
                    .iter()
                    .interleave(b_prime[..n_prime].iter()),
                G_prime[..n_prime]
                    .iter()
                    .interleave(H_prime[n_prime..].iter()),
            );
            let R_i = (R_i + (&(c_R * x_ip) * &*AMOUNT_BASEPOINT_TABLE)) * *INV_EIGHT;

            L.push(L_i);
            R.push(R_i);

            transcript.extend_with_points(&[L_i, R_i]);

            let w_i = transcript.get_current_state();
            if w_i == Scalar::zero() {
                println!("w_i is 0, retrying...");
                continue;
            }
            w.push(w_i);

            let w_inv = w_i.invert();

            G_prime = G_prime[..n_prime]
                .iter()
                .map(|g_prime| w_inv * g_prime)
                .zip(G_prime[n_prime..].iter().map(|g_prime| w_i * g_prime))
                // Hadamard
                .map(|(g_1, g_2)| g_1 + g_2)
                .collect();

            H_prime = H_prime[0..n_prime]
                .iter()
                .map(|h_prime| w_i * h_prime)
                .zip(
                    H_prime[n_prime..H_prime.len()]
                        .iter()
                        .map(|h_prime| w_inv * h_prime),
                )
                // Hadamard
                .map(|(h_1, h_2)| h_1 + h_2)
                .collect();

            a_prime = a_prime[0..n_prime]
                .iter()
                .map(|a_prime| w_i * a_prime)
                .zip(
                    a_prime[n_prime..a_prime.len()]
                        .iter()
                        .map(|a_prime| w_inv * a_prime),
                )
                .map(|(a_1, a_2)| a_1 + a_2)
                .collect();

            b_prime = b_prime[0..n_prime]
                .iter()
                .map(|b_prime| w_inv * b_prime)
                .zip(
                    b_prime[n_prime..b_prime.len()]
                        .iter()
                        .map(|b_prime| w_i * b_prime),
                )
                .map(|(a_1, a_2)| a_1 + a_2)
                .collect();
        }

        return Ok((
            Bulletproof {
                V: V.collect(),
                A,
                S,
                T_1,
                T_2,
                tau_x,
                mu,
                L,
                R,
                a: a_prime[0],
                b: b_prime[0],
                t,
            },
            value_mask_pairs.iter().map(|(_, mask)| *mask).collect(),
        ));
    }
}

/// Checks a set of bulletproofs for validity
pub fn verify_multiple(proofs: &[impl Borrow<Bulletproof>]) -> Result<(), Error> {
    let mut max_length = 0;
    for proof in proofs {
        let proof = proof.borrow();
        // Sanity checks
        if (proof.tau_x.reduce() != proof.tau_x)
            || (proof.mu.reduce() != proof.mu)
            || (proof.a.reduce() != proof.a)
            || (proof.b.reduce() != proof.b)
            || (proof.t.reduce() != proof.t)
        {
            return Err(Error::ScalarsNotReduced);
        }

        if proof.L.is_empty() {
            return Err(Error::EmptyProof);
        }
        if proof.V.is_empty() {
            return Err(Error::NoCommitments);
        }
        if proof.L.len() != proof.R.len() {
            return Err(Error::InconsistentProof);
        }
        max_length = std::cmp::max(max_length, proof.L.len());
    }

    if max_length >= 32 {
        return Err(Error::TooLargeProof);
    }

    let maxMN = 1u64 << max_length;

    // Setup weighted aggregates
    let mut Z0 = Point::identity();
    let mut z1 = Scalar::zero();
    let mut Z2 = Point::identity();
    let mut z3 = Scalar::zero();
    let mut z4 = (0..maxMN).map(|_| Scalar::zero()).collect::<Vec<_>>();
    let mut z5 = (0..maxMN).map(|_| Scalar::zero()).collect::<Vec<_>>();
    let mut Y2 = Point::identity();
    let mut Y3 = Point::identity();
    let mut Y4 = Point::identity();
    let mut y0 = Scalar::zero();
    let mut y1 = Scalar::zero();

    for proof in proofs {
        let proof = proof.borrow();

        let mut M = 0;
        // Find log2(M)
        let mut logM = 0;
        while (M < proof.V.len()) && (M <= M_MAX) {
            logM += 1;
            M = 1 << logM;
        }
        if proof.L.len() != 6 + logM {
            return Err(Error::InconsistentProof);
        }

        let MN = N_BITS * M;

        let weight = Scalar::random(&mut rand::rngs::OsRng);

        // Replay the transcript
        let mut hasher = CNFastHash::new();
        proof
            .V
            .iter()
            .for_each(|x| hasher.input(x.compress().to_bytes()));

        let mut transcript = Transcript::new(hash_to_scalar(hasher.result()));

        // Challenge y
        // Insert A and S
        transcript.extend_with_points(&[proof.A, proof.S]);
        let y = transcript.get_current_state();
        if y == Scalar::zero() {
            return Err(Error::InconsistentProof);
        }

        // Challenge z
        let z = hash_to_scalar(CNFastHash::digest(y.as_bytes()));
        if z == Scalar::zero() {
            return Err(Error::InconsistentProof);
        }
        transcript.reset_state(z);

        transcript.extend_with_scalars(&[z]);
        transcript.extend_with_points(&[proof.T_1, proof.T_2]);

        let x = transcript.get_current_state();
        if x == Scalar::zero() {
            return Err(Error::InconsistentProof);
        }

        transcript.extend_with_scalars(&[x, proof.tau_x, proof.mu, proof.t]);
        let x_ip = transcript.get_current_state();
        if x_ip == Scalar::zero() {
            return Err(Error::InconsistentProof);
        }

        // Multiply some points to account for cofactor-8
        let V = proof
            .V
            .iter()
            .map(|V| V.mul_by_cofactor())
            .collect::<Vec<_>>();
        let L = proof
            .L
            .iter()
            .map(|L| L.mul_by_cofactor())
            .collect::<Vec<_>>();
        let R = proof
            .R
            .iter()
            .map(|R| R.mul_by_cofactor())
            .collect::<Vec<_>>();
        let T_1 = proof.T_1.mul_by_cofactor();
        let T_2 = proof.T_2.mul_by_cofactor();
        let A = proof.A.mul_by_cofactor();
        let S = proof.S.mul_by_cofactor();

        y0 += weight * proof.tau_x;

        let z_pow = power_vector(z, M + 3);

        let ip1y = power_sum(y, MN);
        let mut k = Scalar::zero() - (z_pow[2] * ip1y);
        for j in 1..=M {
            k -= z_pow[j + 2] * (*ONE_TWO_INNER_PRODUCT);
        }

        y1 += weight * (proof.t - (k + (z * ip1y)));
        Y2 += weight * Point::multiscalar_mul(&z_pow[2..(2 + V.len())], V);

        Y3 += (weight * x) * T_1;
        Y4 += (weight * (x * x)) * T_2;

        Z0 += weight * (A + (x * S));

        // log(64) (= 6) + log(M)
        let rounds = 6 + logM;
        let w = (0..rounds)
            .map(|i| {
                transcript.extend_with_points(&[proof.L[i], proof.R[i]]);
                transcript.get_current_state()
            })
            .collect::<Vec<_>>();

        for w_i in &w {
            if *w_i == Scalar::zero() {
                return Err(Error::InconsistentProof);
            }
        }

        let mut y_pow = Scalar::one();
        let y_inv = y.invert();
        let mut y_inv_pow = Scalar::one();
        let w_inv = w.iter().map(|w_i| w_i.invert()).collect::<Vec<_>>();

        for i in 0..MN {
            let mut g = proof.a;
            let mut h = proof.b * y_inv_pow;

            for j in (0..rounds).rev() {
                let J = w.len() - j - 1;

                if (i & (1 << j)) == 0 {
                    g *= w_inv[J];
                    h *= w[J];
                } else {
                    g *= w[J];
                    h *= w_inv[J];
                }
            }

            g += z;

            let mut tmp = z_pow[2 + (i / N_BITS)] * TWO_POWERS[i % N_BITS];
            tmp += z * y_pow;
            h -= tmp * y_inv_pow;

            z4[i] += weight * g;
            z5[i] += weight * h;

            if i != (MN - 1) {
                y_inv_pow *= y_inv;
                y_pow *= y;
            }
        }

        z1 += weight * proof.mu;

        let acc = Point::multiscalar_mul(
            w.iter()
                .map(|w| w * w)
                .interleave(w_inv.iter().map(|w_inv| w_inv * w_inv)),
            L.iter().interleave(R.iter()),
        );

        Z2 += weight * acc;
        let tmp = proof.t - (proof.a * proof.b);
        let tmp = x_ip * tmp;

        z3 += weight * tmp;
    }

    let check1 = (&y0 * &BASEPOINT_TABLE) + (&y1 * &*AMOUNT_BASEPOINT_TABLE) - Y2 - Y3 - Y4;

    if !check1.is_identity() {
        return Err(Error::InvalidProof);
    }

    let p = Point::multiscalar_mul(
        z5.iter().interleave(z4.iter()).map(|s| Scalar::zero() - s),
        (0..(2 * maxMN)).map(|i| get_power(*AMOUNT_BASEPOINT, i)),
    );

    let check2 = Point::vartime_double_scalar_mul_basepoint(
        &z3,
        &(&Scalar::one() * &*AMOUNT_BASEPOINT_TABLE),
        &(Scalar::zero() - z1),
    ) + Z0
        + Z2
        + p;

    if !check2.is_identity() {
        return Err(Error::InvalidProof);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::RngCore;

    use crypto::ecc::{CompressedPoint, ScalarExt};

    #[test]
    fn it_should_generate_proofs_correctly() {
        // 10 random amounts and masks
        let (proof, _) = super::prove_multiple(
            &(0..10)
                .map(|_| rand::rngs::OsRng.next_u64())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        super::verify_multiple(&[proof]).unwrap();
    }

    #[test]
    fn it_should_verify_mainnet_proofs_correctly() {
        // The following is from mainnet transaction <cf8e4ffccd7f3604b4ec4be689a7d3669a8ea8bfa5e40d7bacf44a864ee75365>
        // https://explorer.unprll.cash/tx/cf8e4ffccd7f3604b4ec4be689a7d3669a8ea8bfa5e40d7bacf44a864ee75365
        let b = Bulletproof {
            V: [
                "5324fa962edab083eef717f8dd9f2cced683671cf5f28081c83ee1171c054869",
                "62c50265df62e8b6c78a1e320366684ab5873565ce0e17eaa4e1a28bab9d70f7",
            ]
            .iter()
            .map(|x| hex::decode(x).unwrap())
            .map(|x| CompressedPoint::from_slice(&x).decompress().unwrap())
            // NOTE: Remember to multiply by eight inverse in actual code
            .map(|x| x * *INV_EIGHT)
            .collect(),
            A: CompressedPoint::from_slice(
                &hex::decode("85df863be3a385365b82cfbef09aaa87267522265e9dc7d8f5cf32440bcf3996")
                    .unwrap(),
            )
            .decompress()
            .unwrap(),
            S: CompressedPoint::from_slice(
                &hex::decode("51d1d9f2ba89de8cb5608c98c795cb6079a0b4aafb60ce5c444159d8edb8db6c")
                    .unwrap(),
            )
            .decompress()
            .unwrap(),
            T_1: CompressedPoint::from_slice(
                &hex::decode("d6939befc6a1d735fa4a13e0c4f69bc1e72bdacab6f60c260fa763c6f412f474")
                    .unwrap(),
            )
            .decompress()
            .unwrap(),
            T_2: CompressedPoint::from_slice(
                &hex::decode("21331553a5d2a385aeeec00d7f252b86bd6a676f63e21a16d4173f0f0ac795e3")
                    .unwrap(),
            )
            .decompress()
            .unwrap(),
            tau_x: Scalar::from_slice(
                &hex::decode("057d34ae685f3b753eba9be6bb3fb88fe2335aed10bbf027beac6d071593f600")
                    .unwrap(),
            ),
            mu: Scalar::from_slice(
                &hex::decode("b5b36890fe4006fedf8d5d8d5b7a33b71b60411d229c96d8fdec8c8db20b2902")
                    .unwrap(),
            ),
            L: [
                "6a5d60a0ece269606913ad09434be74852ef65c8248c111921cd5ca25eed2324",
                "148f1631c946ac671b66ff79ab02cdf44b13259d8173c4039fbf1f6d04342b42",
                "75d59916898656b9c13929f2abf386263b241c42a6c9bfaf404ebbdf20ee0e4d",
                "193a55a65d50a19f4f367873bccfb869bc32b0aca8982d5654dd8a10b14805d2",
                "50bf471652814b69b4f690f510fb0bd4cc0d1181ddd86805c82d8b6fd6b3b391",
                "20d6ebc217ac4ee405cdb23fe48f87ece14d1cb19845af38a054a8d2ae6aec95",
                "84b7729f5fe410c5dc00dcb1fbe1218f118e5d92eec81943b5546cac74653043",
            ]
            .iter()
            .map(|x| hex::decode(x).unwrap())
            .map(|x| CompressedPoint::from_slice(&x).decompress().unwrap())
            .collect(),
            R: [
                "dfc25c10bd846911a1ce4bbe5a7bb877757937a30781605117aebb4890f22936",
                "f2e77c4c16a18c46567a5f5d6e0d0765206cc435bbf59bcb66ae5e926d6844db",
                "2f877df20b0fe87e422da9c233c39890665a326f50c6861bbf17e61b068a6b4e",
                "980e92d495c02d86c92372efe423f0c9e29ef1e94bfa4ca6688bf17d045f0819",
                "e14916dff97b534d7c30d911bdf2e0586ed676f8be3b200ae18b264ecd0d94da",
                "fe8a374b1322d6bd325ec963b5e9e3a7f0151dd712c7c81d208c0c5429983dcd",
                "48901c3dd352422a24056e2a0f72e41a8c1eeee56fa600f83a1027d068e2c8e0",
            ]
            .iter()
            .map(|x| hex::decode(x).unwrap())
            .map(|x| CompressedPoint::from_slice(&x).decompress().unwrap())
            .collect(),
            a: Scalar::from_slice(
                &hex::decode("6d97e02c1942f18a900854d337d428b92416af2680335f8fc7fd003320a19700")
                    .unwrap(),
            ),
            b: Scalar::from_slice(
                &hex::decode("d0dea26ace229cace97f2f477f4cf770871d784720cac53ecb47ead8021a5b09")
                    .unwrap(),
            ),
            t: Scalar::from_slice(
                &hex::decode("bfa2af387659ddb7fb4418fb8094a99f394012c5fe300c7cf8bf15cc91fd2d04")
                    .unwrap(),
            ),
        };

        super::verify_multiple(&[&b]).unwrap()
    }
}
