### Context

This was a (recruiting) challenge from the company Industrial Path Solutions from November 2022.

I think I wrote a quite good solution, given my hardware (many-core avx2+fma CPU).

https://github.com/industrialpathsolutions/student-challenge-particle-simulation

### Task

Find all pairs of points in `positions.xyz` closer than `0.05` euclidean distance apart in 3D.

### Performance

About 375us using AVX2+FMA on my 24-thread 5900x. About 420us using alternative non-SIMD algorithm
that is limited by BTree insertion/deletion/lookup. I have not tried AVX512 since I lack hardware
support myself.

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

The solver API of
```
pub fn solve_threaded<SubscanSolver>(
    xyzi: &mut [(f32, f32, f32, u16)],
    (x_soa, y_soa, z_soa, i_soa): (&mut Vec<f32>, &mut Vec<f32>, &mut Vec<f32>, &mut Vec<u16>),
    parallel: NonZeroUsize,
    ret: &mut Vec<MaybeUninit<(u16, u16)>>,
)
```
might seem a little weird, but it is motivated:
- We pass `xyzi` mutably since we want to sort it without allocating. In the benchmark we still
    include copying from an immutable `xyzi` to this mutable buffer before every call, but in a real
    application we could skip the copy and sort an already almost-sorted array giving a further ~30%
    speedup.
- The BTree solution is fastest with a AoS layout, while the SIMD solution is fastest with a SoA
    layout. Since the SoA must be sorted by `x`, we start by sorting the AoS input and copying to
    some SoA arrays. We pass mutable Vec:s to minimize allocation and deallocation.
- We pass `parallel` since syscalling for it every call is significant overhead.
- We pass `ret` as an out parameter to re-use the allocation from earlier calls.

### Attempted approaches that did not help

- ~~Struct-of-array layout (does not match access patterns)~~ (useful in SIMD solution)
- Rebuilding stdlib `BTreeSet` with `B != 6` (`B = 6` seems like a very good default actually)
- ~~AVX2 SIMD pair distance computation (maybe bottlenecked by branch before answer insertion?)~~
    (useful after realizing that most points with close x-values are not close in euclidian distance
    and can be early-exited.

### How to run

Either `nix run` or `nix develop` followed by `cargo run --release`. Cargo could also be acquired
from another package manager.

The input file is included at compile time for convenience. One can change
```
let xyzi = &parse_input(DATA);
```
to
```
let xyzi = &parse_input(std::io::stdin().lock());
```
in `src/main.rs` to read from stdin instead.
