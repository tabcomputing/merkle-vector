# merkle-vector

A persistent vector for content-addressed systems: a 32-way vector trie
(Bagwell's, as in Clojure) with the properties
[merkle-champ](https://github.com/tabcomputing/merkle-champ) gives its map.

- **Canonical shape.** A vector's structure depends only on its length. The
  last 1 to 32 elements sit in a tail, the rest in full leaves of 32 under the
  fewest levels of branches that hold them, so pushes, pops, updates and
  rebuilding from scratch all give the same tree.
- **Lazily cached Merkle identities.** `vector.identity()` is a SHA-256
  identity of the contents. Each node computes its identity on first request
  and caches it; versions share unchanged nodes, so after a write only the
  changed path is rehashed.
- **Shared value encodings.** Elements are encoded with merkle-champ's
  `Identify`, so a value has the same encoding in a map and in a vector, and
  vectors and maps nest in each other.

```rust
use merkle_vector::Vector;

let v1: Vector<u64> = (0..100).collect();
let v2 = v1.update(5, 50);          // v1 is unchanged
assert_eq!(v1.get(5), Some(&5));
assert_eq!(v2.get(5), Some(&50));

let mut v3: Vector<u64> = (0..150).collect();
for _ in 0..50 { v3.pop(); }
assert_eq!(v1.identity(), v3.identity());   // history does not matter
```

`get`, `set` and `update` take O(log₃₂ n) steps, effectively constant; `push`
and `pop` are amortised O(1); cloning is O(1). `concat` and `slice` copy, O(n):
the March language, which this crate serves, plans lazy views for those.

The identity format is specified in [FORMAT.md](FORMAT.md) and checked against
an independent computation of it in `tests/identity.rs`.

Status: early. Not yet here: storage (nodes as their identity preimages, as in
merkle-champ), structural diff, and a builder that fills leaves in place.

Licensed under either of MIT or Apache-2.0, at your option.
