### Task

Find all pairs of points closer than `0.05` euclidean distance apart in 3D.

### Performance

About 420us on my 24-thread 5900x. Limited by BTree insertion/deletion/lookup.

### Explanation

We find which of the 3 dimensions the points are most closely spaced in. We can reflect the points
through any plane and still have the same close pairs. Doing this, we can have the dimensions x,y,z
be sorted in order of increasing point closeness.

In the input file, x is already the most spread out dimension, followed by y. Hence we actually do
not need to reflect anything.

We give the (array-of-struct) points unique ids (from input order, but in a simulation these could
be anything) and sort them by x. We then scan through them, keeping the most recent slice in an
ordered tree and do range lookups before each insertion. For uniformly spaced points this would
result in a time complexity of `O(n (log n + n^1/3)) = O(n^4/3)`.

The outer scan is parallelized: Each cpu thread scans through a subinterval of x-values. The scaling
is sublinear:
- partly because each thread must start by constructing the ordered tree corresponding
to its starting x-value
- and partly because we need to flatten an uneven nest list of close-pair-answers

The first parallelization overhead is inherent, the second could be avoided by for example executing
a callback function for each close pair.

### Attempted approaches that did not help

- Struct-of-array layout (does not match access patterns)
- Rebuilding stdlib `BTreeSet` with `B != 6` (`B = 6` seems like a very good default actually)
- AVX2 SIMD pair distance computation (maybe bottlenecked by branch before answer insertion?)

### How to run

`nix develop` followed by `cargo run --release`. Cargo could also be acquired from another package
manager. `nix run` also works, but causes a 1-2% slowdown for some reason.

The input file is included at compile time for convenience. One can change
```
let xyzi = &parse_input(DATA);
```
to
```
let xyzi = &parse_input(std::io::stdin().lock());
```
in `src/main.rs` to read from stdin instead.
