// Needed because most cryptographic code relies on non snake case names
#![allow(non_snake_case)]

use failure::format_err;

use crypto::{
    curve25519_dalek::{
        traits::{
            Identity,
            IsIdentity,
            MultiscalarMul
        }
    },
    CNFastHash,
    Digest,
    ecc::{
        BASEPOINT_TABLE,
        Point,
        Scalar,
        hash_to_point,
        hash_to_scalar
    }
};

use crate::{
    MASK_BASEPOINT,
    MASK_BASEPOINT_TABLE,
};

pub struct Bulletproof {
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
    pub t: Scalar
}

/// Maximum number of bits `N` of the value
///
/// The input value is then proved to be within `[0,2^n]`
const N_BITS: usize = 64;

/// Maximum number of values proved by a given bulletproof
const M_MAX: usize = 16;

/// Generate a vector of Pedersen Commitments from a set of Scalars
// fn create_commitments(a: &mut dyn Iterator<Item = Scalar>, b: &mut dyn Iterator<Item = Scalar>) -> Point {
//     a.zip(b).map(|(l, r)| {
//         (&l * &BASEPOINT_TABLE) + (&r * &*MASK_BASEPOINT_TABLE)
//     }).fold(Point::default(), |sum, p| sum + p)
// }

struct Transcript {
    hasher: CNFastHash,
    value: Scalar
}

impl Transcript {
    pub fn new(initial_value: Scalar) -> Transcript {
        let mut t = Transcript {
            hasher: CNFastHash::new(),
            value: initial_value
        };
        // Prefill the hasher with the current transcript value
        t.hasher.input(t.value.as_bytes());
        t
    }

    pub fn extend_with_points(&mut self, points: &[Point]) {
        points.iter().for_each(|point| self.hasher.input(point.compress().as_bytes()))
    }

    pub fn extend_with_scalars(&mut self, scalars: &[Scalar]) {
        scalars.iter().for_each(|scalar| self.hasher.input(scalar.as_bytes()))
    }

    pub fn reset_state(&mut self, value: Scalar) {
        self.hasher.reset();
        self.value = value;
        self.hasher.input(self.value.as_bytes());
    }

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
fn inner_product(a: &[Scalar], b: &[Scalar]) -> Scalar {
    let mut res = Scalar::zero();
    for (a, b) in a.iter().zip(b) {
        res += a * b;
    }
    res
}

fn get_power(base: Point, index: u64) -> Point {
    let mut hasher = CNFastHash::new();

    hasher.input(base.compress().as_bytes());
    hasher.input(b"bulletproof");
    hasher.input(bincode_epee::serialize(&index).unwrap());

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
    static ref ONE_TWO_INNER_PRODUCT: Scalar = inner_product(&(0..N_BITS).map(|_| Scalar::one()).collect::<Vec<_>>(), &TWO_POWERS);
}

/// Checks a set of bulletproofs for validity
pub fn verify_multiple<'a>(proofs: &[&'a Bulletproof]) -> Result<(), failure::Error> {
    let mut max_length = 0;
    for proof in proofs {
        // Sanity checks
        if (proof.tau_x.reduce() != proof.tau_x)
            || (proof.mu.reduce() != proof.mu)
            || (proof.a.reduce() != proof.a)
            || (proof.b.reduce() != proof.b)
            || (proof.t.reduce() != proof.t)
        {
            return Err(format_err!("Input scalars not in range"));
        }

        if proof.L.is_empty() {
            return Err(format_err!("Proof is empty"));
        }
        if proof.V.is_empty() {
            return Err(format_err!("Proof does not have at least one commitment V"));
        }
        if proof.L.len() != proof.R.len() {
            return Err(format_err!("Proof does not have L.len() == R.len()"));
        }
        max_length = std::cmp::max(max_length, proof.L.len());
    }

    if max_length >= 32 {
        return Err(format_err!("Atleast one proof is too large"));
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
        let mut M = 0;
        // Find log2(M)
        let mut logM = 0;
        while (M < proof.V.len()) && (M <= M_MAX) {
            logM += 1;
            M = 1 << logM;
        }
        if proof.L.len() != 6 + logM {
            return Err(format_err!("Proof does not have the expected size"));
        }

        let MN = N_BITS * M;

        let weight = Scalar::random(&mut rand::rngs::OsRng);

        // Replay the transcript
        let mut hasher = CNFastHash::new();
        proof.V.iter().for_each(|x| hasher.input(x.compress().to_bytes()));

        let mut transcript = Transcript::new(hash_to_scalar(hasher.result()));

        // Challenge y
        // Insert A and S
        transcript.extend_with_points(&[proof.A, proof.S]);
        let y = transcript.get_current_state();
        if y == Scalar::zero() {
            return Err(format_err!("y == 0"));
        }

        // Challenge z
        let z = hash_to_scalar(CNFastHash::digest(y.as_bytes()));
        if z == Scalar::zero() {
            return Err(format_err!("z == 0"));
        }
        transcript.reset_state(z);

        transcript.extend_with_scalars(&[z]);
        transcript.extend_with_points(&[proof.T_1, proof.T_2]);

        let x = transcript.get_current_state();
        if x == Scalar::zero() {
            return Err(format_err!("x == 0"));
        }

        transcript.extend_with_scalars(&[x, proof.tau_x, proof.mu, proof.t]);
        let x_ip = transcript.get_current_state();
        if x_ip == Scalar::zero() {
            return Err(format_err!("x_ip == 0"));
        }

        // Multiply some points to account for cofactor-8
        let V = proof.V.iter().map(|V| V.mul_by_cofactor()).collect::<Vec<_>>();
        let L = proof.L.iter().map(|L| L.mul_by_cofactor()).collect::<Vec<_>>();
        let R = proof.R.iter().map(|R| R.mul_by_cofactor()).collect::<Vec<_>>();
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
        let w = (0..rounds).map(|i| {
            transcript.extend_with_points(&[proof.L[i], proof.R[i]]);
            transcript.get_current_state()
        }).collect::<Vec<_>>();

        for w_i in &w {
            if *w_i == Scalar::zero() {
                return Err(format_err!("w[i] == 0"));
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
            (0..(2*rounds)).map(|i| {
                if i % 2 == 0 {
                    w[i/2] * w[i/2]
                } else {
                    w_inv[i/2] * w_inv[i/2]
                }
            }),
            (0..(2*rounds)).map(|i| {
                if i % 2 == 0 {
                    L[i/2]
                } else {
                    R[i/2]
                }
            })
        );

        Z2 += weight * acc;
        let tmp = proof.t - (proof.a * proof.b);
        let tmp = x_ip * tmp;

        z3 += weight * tmp;
    }

    let check1 = (&y0 * &BASEPOINT_TABLE)
        + (&y1 * &*MASK_BASEPOINT_TABLE)
        - Y2
        - Y3
        - Y4;

    if !check1.is_identity() {
        return Err(format_err!("Check 1 failed"));
    }

    let p = Point::multiscalar_mul(
        (0..(2*maxMN)).map(|i| {
            let i = i as usize;
            if i % 2 == 0 {
                z5[i/2]
            } else {
                z4[i/2]
            }
        }).map(|s| Scalar::zero() - s),
        (0..(2*maxMN)).map(|i| {
            get_power(*MASK_BASEPOINT, i)
        })
    );

    let check2 = Point::vartime_double_scalar_mul_basepoint(
        &z3,
        &(&Scalar::one() * &*MASK_BASEPOINT_TABLE),
        &(Scalar::zero() - z1)
    )
        + Z0
        + Z2
        + p;

    if !check2.is_identity() {
        return Err(format_err!("Check 2 failed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::ecc::{
        CompressedPoint,
        ScalarExt
    };

    #[test]
    fn it_should_verify_correctly() {
        // The following is from mainnet transaction <cf8e4ffccd7f3604b4ec4be689a7d3669a8ea8bfa5e40d7bacf44a864ee75365>
        // https://explorer.unprll.cash/tx/cf8e4ffccd7f3604b4ec4be689a7d3669a8ea8bfa5e40d7bacf44a864ee75365
        let b = Bulletproof {
            V: [
                "5324fa962edab083eef717f8dd9f2cced683671cf5f28081c83ee1171c054869",
                "62c50265df62e8b6c78a1e320366684ab5873565ce0e17eaa4e1a28bab9d70f7"
            ].iter()
                .map(|x| hex::decode(x).unwrap())
                .map(|x| CompressedPoint::from_slice(&x).decompress().unwrap())
                // NOTE: Remember to multiply by eight inverse in actual code
                .map(|x| x * *INV_EIGHT)
                .collect(),
            A: CompressedPoint::from_slice(&hex::decode("85df863be3a385365b82cfbef09aaa87267522265e9dc7d8f5cf32440bcf3996").unwrap()).decompress().unwrap(),
            S: CompressedPoint::from_slice(&hex::decode("51d1d9f2ba89de8cb5608c98c795cb6079a0b4aafb60ce5c444159d8edb8db6c").unwrap()).decompress().unwrap(),
            T_1: CompressedPoint::from_slice(&hex::decode("d6939befc6a1d735fa4a13e0c4f69bc1e72bdacab6f60c260fa763c6f412f474").unwrap()).decompress().unwrap(),
            T_2: CompressedPoint::from_slice(&hex::decode("21331553a5d2a385aeeec00d7f252b86bd6a676f63e21a16d4173f0f0ac795e3").unwrap()).decompress().unwrap(),
            tau_x: Scalar::from_slice(&hex::decode("057d34ae685f3b753eba9be6bb3fb88fe2335aed10bbf027beac6d071593f600").unwrap()),
            mu: Scalar::from_slice(&hex::decode("b5b36890fe4006fedf8d5d8d5b7a33b71b60411d229c96d8fdec8c8db20b2902").unwrap()),
            L: [
                "6a5d60a0ece269606913ad09434be74852ef65c8248c111921cd5ca25eed2324",
                "148f1631c946ac671b66ff79ab02cdf44b13259d8173c4039fbf1f6d04342b42",
                "75d59916898656b9c13929f2abf386263b241c42a6c9bfaf404ebbdf20ee0e4d",
                "193a55a65d50a19f4f367873bccfb869bc32b0aca8982d5654dd8a10b14805d2",
                "50bf471652814b69b4f690f510fb0bd4cc0d1181ddd86805c82d8b6fd6b3b391",
                "20d6ebc217ac4ee405cdb23fe48f87ece14d1cb19845af38a054a8d2ae6aec95",
                "84b7729f5fe410c5dc00dcb1fbe1218f118e5d92eec81943b5546cac74653043"
            ].iter()
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
                "48901c3dd352422a24056e2a0f72e41a8c1eeee56fa600f83a1027d068e2c8e0"
            ].iter()
                .map(|x| hex::decode(x).unwrap())
                .map(|x| CompressedPoint::from_slice(&x).decompress().unwrap())
                .collect(),
            a: Scalar::from_slice(&hex::decode("6d97e02c1942f18a900854d337d428b92416af2680335f8fc7fd003320a19700").unwrap()),
            b: Scalar::from_slice(&hex::decode("d0dea26ace229cace97f2f477f4cf770871d784720cac53ecb47ead8021a5b09").unwrap()),
            t: Scalar::from_slice(&hex::decode("bfa2af387659ddb7fb4418fb8094a99f394012c5fe300c7cf8bf15cc91fd2d04").unwrap())
        };

        let res = super::verify_multiple(&[&b]);
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
