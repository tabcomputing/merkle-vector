//! A vector against a `Vec` model, through random and boundary-crossing
//! operations, with the canonical shape checked along the way.
use merkle_vector::Vector;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn agrees(v: &Vector<u64>, model: &[u64]) {
    assert_eq!(v.len(), model.len());
    assert!(v.iter().eq(model.iter()), "elements differ");
    v.check_invariants().unwrap();
}

#[test]
fn grows_and_shrinks_through_every_level() {
    // Past 32 (the first leaf), 1,056 (a second branch level) and 32,800 (a
    // third), then all the way back down.
    let mut v = Vector::new();
    let mut model = Vec::new();
    for i in 0..40_000u64 {
        v.push(i);
        model.push(i);
        if i % 997 == 0 || (i as usize + 1).is_power_of_two() || (i + 1) % 32 <= 1 && i < 2_200 {
            agrees(&v, &model);
        }
    }
    agrees(&v, &model);
    for i in (0..40_000usize).rev() {
        assert_eq!(v.pop(), model.pop());
        if i % 991 == 0 || i.is_power_of_two() || i % 32 <= 1 && i < 2_200 {
            agrees(&v, &model);
        }
    }
    assert_eq!(v.pop(), None);
    agrees(&v, &model);
}

#[test]
fn random_operations_match_a_vec() {
    let mut rng = Rng(2026);
    let mut v: Vector<u64> = Vector::new();
    let mut model: Vec<u64> = Vec::new();
    for step in 0..60_000 {
        match rng.below(10) {
            0..=4 => {
                let x = rng.next();
                v.push(x);
                model.push(x);
            }
            5 | 6 => assert_eq!(v.pop(), model.pop()),
            7 | 8 if !model.is_empty() => {
                let i = rng.below(model.len());
                let x = rng.next();
                assert_eq!(v.set(i, x), Ok(model[i]));
                model[i] = x;
            }
            _ if !model.is_empty() => {
                let i = rng.below(model.len());
                assert_eq!(v.get(i), Some(&model[i]));
            }
            _ => {}
        }
        if step % 1_000 == 0 {
            agrees(&v, &model);
        }
    }
    agrees(&v, &model);
    assert_eq!(v.get(model.len()), None);
    assert_eq!(v.set(model.len(), 0), Err(0));
}

#[test]
fn old_versions_are_unchanged() {
    let base: Vector<u64> = (0..5_000).collect();
    let mut a = base.clone();
    a.set(17, 99).unwrap();
    a.set(4_990, 99).unwrap();
    let b = base.update(2_000, 7).pushed(1).pushed(2);
    let mut c = base.clone();
    for _ in 0..1_000 {
        c.pop();
    }
    assert!(base.iter().copied().eq(0..5_000));
    assert_eq!(a.get(17), Some(&99));
    assert_eq!(b.get(2_000), Some(&7));
    assert_eq!(b.len(), 5_002);
    assert_eq!(c.len(), 4_000);
    for v in [&base, &a, &b, &c] {
        v.check_invariants().unwrap();
    }
}

#[test]
fn concat_and_slice_copy_into_canonical_vectors() {
    let a: Vector<u64> = (0..1_000).collect();
    let b: Vector<u64> = (1_000..2_345).collect();
    let ab = a.concat(&b);
    assert!(ab.iter().copied().eq(0..2_345));
    ab.check_invariants().unwrap();
    let s = ab.slice(37..1_500);
    assert!(s.iter().copied().eq(37..1_500));
    s.check_invariants().unwrap();
    assert_eq!(ab.slice(5..5).len(), 0);
    assert_eq!(ab.first(), Some(&0));
    assert_eq!(ab.last(), Some(&2_344));
}
