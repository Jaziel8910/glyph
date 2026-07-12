//! Shamir Secret Sharing over GF(2^8) (AES reduction polynomial).
//!
//! Splits a secret (the 32-byte seed) into `shares` parts such that any
//! `threshold` of them reconstruct it, but `threshold-1` reveal nothing. Each
//! share is `x || y` (33 bytes) and is encoded to a recovery phrase by the
//! caller, so shares travel as ordinary Glyph phrases.

use rand::RngCore;

const PRIME_POLY: u8 = 0x1b; // AES reduction polynomial (x^8 + x^4 + x^3 + x^2 + 1) low byte

fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a = (a << 1) & 0xff;
        if hi != 0 {
            a ^= PRIME_POLY;
        }
        b >>= 1;
    }
    p
}

fn gf_inv(a: u8) -> u8 {
    // a^(254) in GF(2^8), a != 0
    let mut r = 1u8;
    let e = a;
    for _ in 0..254 {
        r = gf_mul(r, e);
    }
    r
}

/// Split `secret` into `shares` parts; `threshold` required to reconstruct.
/// Share `x` values are `1..=shares` (distinct, non-zero).
pub fn split(
    secret: &[u8],
    shares: u8,
    threshold: u8,
    rng: &mut impl RngCore,
) -> anyhow::Result<Vec<(u8, Vec<u8>)>> {
    if threshold < 2 {
        anyhow::bail!("threshold debe ser >= 2");
    }
    if threshold > shares {
        anyhow::bail!("threshold no puede superar shares");
    }
    if shares == 0 {
        anyhow::bail!("shares debe ser >= 1");
    }
    let n = secret.len();
    let mut out: Vec<(u8, Vec<u8>)> = (1..=shares).map(|x| (x, vec![0u8; n])).collect();

    for (i, &s) in secret.iter().enumerate() {
        // polynomial coefficients: c0 = secret byte, c1..c_{t-1} random
        let mut coeffs = vec![s];
        for _ in 1..threshold {
            coeffs.push(rng.next_u32() as u8);
        }
        for (x, share) in out.iter_mut() {
            let mut y = 0u8;
            let mut term = 1u8;
            for &c in &coeffs {
                y ^= gf_mul(c, term);
                term = gf_mul(term, *x);
            }
            share[i] = y;
        }
    }
    Ok(out)
}

/// Reconstruct the secret from any `threshold` shares.
pub fn combine(shares: &[(u8, Vec<u8>)]) -> anyhow::Result<Vec<u8>> {
    let k = shares.len();
    if k == 0 {
        anyhow::bail!("no hay shares para combinar");
    }
    let n = shares[0].1.len();
    for (_, v) in shares {
        if v.len() != n {
            anyhow::bail!("los shares tienen distinto tamaño");
        }
    }
    let xs: Vec<u8> = shares.iter().map(|(x, _)| *x).collect();
    let mut secret = vec![0u8; n];
    for (i, slot) in secret.iter_mut().enumerate() {
        let mut acc = 0u8;
        for j in 0..k {
            let xj = xs[j];
            let mut num = 1u8;
            let mut den = 1u8;
            for m in 0..k {
                if m == j {
                    continue;
                }
                let xm = xs[m];
                num = gf_mul(num, xm); // (0 - xm) == xm in GF(2^8)
                den = gf_mul(den, xj ^ xm);
            }
            let lag = gf_mul(num, gf_inv(den));
            acc ^= gf_mul(shares[j].1[i], lag);
        }
        *slot = acc;
    }
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> [u8; 32] {
        [0x42u8; 32]
    }

    #[test]
    fn threshold_recovers() {
        let s = secret();
        let mut rng = rand::thread_rng();
        let shares = split(&s, 5, 3, &mut rng).unwrap();
        let recovered = combine(&shares[..3]).unwrap();
        assert_eq!(recovered, s.to_vec());
    }

    #[test]
    fn any_subset_of_threshold_recovers() {
        let s = secret();
        let mut rng = rand::thread_rng();
        let shares = split(&s, 7, 4, &mut rng).unwrap();
        // pick a non-contiguous subset
        let subset = vec![shares[0].clone(), shares[2].clone(), shares[4].clone(), shares[6].clone()];
        assert_eq!(combine(&subset).unwrap(), s.to_vec());
    }

    #[test]
    fn shares_below_threshold_fail() {
        let s = secret();
        let mut rng = rand::thread_rng();
        let shares = split(&s, 5, 3, &mut rng).unwrap();
        // 2 shares should NOT equal the secret (with overwhelming probability)
        let recovered = combine(&shares[..2]).unwrap();
        assert_ne!(recovered, s.to_vec());
    }
}
