# merkle-vector identity format, version 1

This document specifies how a vector's identity is computed, so that another
implementation can reproduce it byte for byte. `tests/identity.rs` checks the
implementation against an independent computation of this specification. Any
change that alters an identity computed under this document is a new format
version and must change the domain strings.

## 1. Canonical shape

A vector of `len` elements is split by its length alone:

- **The tail** holds the last elements: indexes `tailoff` to `len - 1`, where
  `tailoff = ((len - 1) / 32) * 32` for `len > 0`, and `tailoff = 0` for the
  empty vector. A non-empty vector's tail holds 1 to 32 elements.
- **The tree** holds the first `tailoff` elements, a multiple of 32, and is
  absent when `tailoff` is 0. Its leaves hold exactly 32 elements each, in
  order. Above them are `h ≥ 1` levels of branches; a branch holds 1 to 32
  children, in order, and every branch except those on the rightmost path is
  full. `h` is the smallest value with `32^(h + 1) ≥ tailoff`.

The shape therefore depends only on `len`, never on how the vector was built:
pushes, pops, updates and rebuilding from scratch all give the same tree.

## 2. Node identity

All integers are little-endian. `||` is concatenation. SHA-256 throughout.
Elements are encoded with merkle-champ's `Identify` (merkle-champ FORMAT.md,
section 5).

```
leaf   = SHA-256( "merkle-vector/leaf/v1"   || count (u8) || Identify(e) for each element in order )
branch = SHA-256( "merkle-vector/branch/v1" || count (u8) || identity(child) for each child in order )
```

`count` is the number of elements (leaf) or children (branch), 1 to 32 (0 for
the empty vector's tail).

## 3. Vector identity

```
vector = SHA-256( "merkle-vector/vector/v1" || len (u64)
               || identity(tree root)   if tailoff > 0
               || identity(tail leaf)   if len > 0 )
```

Given `len`, which parts are present and where the tail begins are fixed, so
the encoding is unambiguous. The empty vector's identity is
`SHA-256("merkle-vector/vector/v1" || 0u64)`.

## 4. Nesting

A vector inside a merkle-champ map or another vector is encoded, through
`Identify`, as the tag `v` followed by a `u64` length 32 and its identity
(the same scheme as a nested map's `m`).

## 5. Not yet specified

Storage (nodes as their identity preimages, as merkle-champ FORMAT.md section
9) and structural diff come in later versions of the crate.
