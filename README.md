# Packed Colony

[![Tests](https://github.com/logankaser/packed-colony/actions/workflows/ci.yml/badge.svg)](https://github.com/logankaser/packed-colony/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/packed-colony)](https://crates.io/crates/packed-colony)
[![Docs](https://img.shields.io/docsrs/packed-colony)](https://docs.rs/packed-colony)

Cache-friendly packed associative data structure.  
O(1) lookup and deletion, O(1) insetion (amortized).  
Ideal iteration, data is tightly packed in one allocation.

```rust
use packed_colony::Colony;
let mut library = Colony::new();
let book1 = library.insert("Foucault's Pendulum");
println!("{}", library[book1]);
```
Suitable for real-time systems such as storing game entities,
drawables in rendering engines or similar. Provides ideal iteration
performance while allowing for associative lookup of specific elements.
Unlike the indexes of a `Vec`, the ids of a `Colony` remain stable as
elements are added or removed.
### Advantages
* Very fast lookup (lookup is two array accesses)
* Underlying `Vec<T>` storage is accessible (as `elements`) and tightly-packed:
```rust
use packed_colony::Colony;
let mut scores = Colony::new();
for x in 1..100 {
  scores.insert(x);
}
for score in scores.elements {
  println!("{}", score);
}
```
* Underlying `Vec` acts like a slab or pool allocator, amortising allocation cost
* Faster than a `HashMap` for lookup and Iteration
### Disadvantages
* User does not pick the keys
* Keys may be re-used, meaning in:
```rust
use packed_colony::Colony;
let mut world = Colony::new();
let omega = world.insert("omega");
let star = world.insert("star");
world.remove(omega);
let gamma = world.insert("gamma");
````
`omega` and `gamma` may be the same.
* elements are not pointer-stable
## Implementation Notes
The Colony internally uses two lookup tables,
`id_to_index` and `index_to_id`.
A lookup is as simple as `elements[id_to_index[id]]`.
During removal, the removed element is swapped for the last
element in members, and the lookup tables are updated.
This naturally keeps all the data tightly packed.
