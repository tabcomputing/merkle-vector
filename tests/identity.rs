//! Identities against an independent computation of FORMAT.md, which
//! chunks a plain slice rather than walking the vector's tree.
use merkle_vector::Vector;
use sha2::{Digest, Sha256};

/// merkle-champ's encoding of a `u64`: tag `u`, length 8, the bytes.
fn encode(x: u64) -> Vec<u8> {
    let mut e = vec![b'u'];
    e.extend(8u64.to_le_bytes());
    e.extend(x.to_le_bytes());
    e
}

fn leaf(xs: &[u64]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"merkle-vector/leaf/v1");
    h.update([xs.len() as u8]);
    for &x in xs {
        h.update(encode(x));
    }
    h.finalize().into()
}

fn branch(ids: &[[u8; 32]]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"merkle-vector/branch/v1");
    h.update([ids.len() as u8]);
    for id in ids {
        h.update(id);
    }
    h.finalize().into()
}

/// The tree over `xs`, a multiple of 32: leaves of 32, then branches of up to
/// 32, left-packed, until one root remains (at least one branch level).
fn tree(xs: &[u64]) -> [u8; 32] {
    let mut level: Vec<[u8; 32]> = xs.chunks(32).map(leaf).collect();
    loop {
        level = level.chunks(32).map(branch).collect();
        if level.len() == 1 {
            return level[0];
        }
    }
}

fn reference(xs: &[u64]) -> [u8; 32] {
    let len = xs.len();
    let tailoff = if len == 0 { 0 } else { (len - 1) / 32 * 32 };
    let mut h = Sha256::new();
    h.update(b"merkle-vector/vector/v1");
    h.update((len as u64).to_le_bytes());
    if tailoff > 0 {
        h.update(tree(&xs[..tailoff]));
    }
    if len > 0 {
        h.update(leaf(&xs[tailoff..]));
    }
    h.finalize().into()
}

#[test]
fn identities_follow_the_format() {
    for len in [
        0, 1, 31, 32, 33, 63, 64, 65, 1_023, 1_024, 1_055, 1_056, 1_057, 2_000, 32_800, 32_801,
        33_000,
    ] {
        let xs: Vec<u64> = (0..len as u64).map(|i| i * 7 + 3).collect();
        let v: Vector<u64> = xs.iter().copied().collect();
        assert_eq!(v.identity(), reference(&xs), "length {len}");
    }
}

#[test]
fn history_does_not_matter() {
    let direct: Vector<u64> = (0..1_500).collect();
    let mut grown: Vector<u64> = (0..2_100).collect();
    for _ in 0..600 {
        grown.pop();
    }
    let mut edited: Vector<u64> = (0..1_500).map(|_| 0).collect();
    for i in (0..1_500).rev() {
        edited.set(i, i as u64).unwrap();
    }
    assert_eq!(direct.identity(), grown.identity());
    assert_eq!(direct.identity(), edited.identity());
    // Cached identities are cleared by writes: changing and restoring an
    // element restores the identity.
    let mut v = direct.clone();
    let before = v.identity();
    v.set(700, 1).unwrap();
    assert_ne!(v.identity(), before);
    v.set(700, 700).unwrap();
    assert_eq!(v.identity(), before);
}

#[test]
fn contents_decide_identity() {
    let a: Vector<u64> = (0..100).collect();
    let b: Vector<u64> = (0..101).collect();
    let c = a.update(99, 100);
    assert_ne!(a.identity(), b.identity());
    assert_ne!(a.identity(), c.identity());
    assert_eq!(Vector::<u64>::new().identity(), reference(&[]));
}

#[test]
fn vectors_nest_through_identify() {
    use merkle_champ::{ChampMap, Identify};
    let inner: Vector<u64> = (0..40).collect();
    let outer: Vector<Vector<u64>> = vec![inner.clone(), Vector::new()].into_iter().collect();
    let again: Vector<Vector<u64>> = vec![(0..40).collect(), Vector::new()].into_iter().collect();
    assert_eq!(outer.identity(), again.identity());
    // In a merkle-champ map too: the encoding is the tag `v` and the identity.
    let mut encoding = Vec::new();
    inner.identify(&mut encoding);
    assert_eq!(encoding[0], b'v');
    assert_eq!(&encoding[9..], &inner.identity());
    let map: ChampMap<String, Vector<u64>> = [("xs".to_string(), inner)].into_iter().collect();
    let map2: ChampMap<String, Vector<u64>> = [("xs".to_string(), (0..40).collect())]
        .into_iter()
        .collect();
    assert_eq!(map.identity(), map2.identity());
}
