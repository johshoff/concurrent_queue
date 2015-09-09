Concurrent queue
----------------

Based on the paper [Fast Concurrent Queues for x86
Processors](http://www.cs.technion.ac.il/~mad/publications/ppopp2013-x86queues.pdf)
by Adam Morrison and Yehuda Afek.

Only runs on *nightly* since I'm using both `asm!` and `repr(simd)`. The former
is needed for atomic primitives in x86_64 and the latter for 16-byte alignment
of structs, which is needed for using `CMPXCHG16B`.

To run tests:

    cargo test

TODO
----

- don't leak memory in LCRQ (by e.g. using hazard pointers). I tried using
  [crossbeam](https://github.com/aturon/crossbeam) for this, but it doesn't
  seem to fit the use case exactly.
- use compiler intrinsic versions of `compare_and_swap`, `compare_and_swap_2`,
  `test_and_set` and `fetch_and_add` if possible
- see `TODO`s in source code
- store pointers instead of `u64`s

