Concurrent queue
----------------

**This is currently fast but buggy. Please don't actually use it.**

Based on the paper [Fast Concurrent Queues for x86
Processors](http://www.cs.technion.ac.il/~mad/publications/ppopp2013-x86queues.pdf)
by Adam Morrison and Yehuda Afek.

Only runs on *nightly* since I'm using both `asm!` and `repr(simd)`. The former
is needed for atomic primitives in x86_64 and the latter for 16-byte alignment
of structs, which is needed for using `CMPXCHG16B`.

To run tests:

    cargo test

Performance
-----------

Initial performance numbers are quite promising. On my laptop, sending
10,000,000 numbers from each of two threads to be consumed by another takes
1.2 seconds, while the same operation takes 2.4 seconds with `mpsc::channel`.

TODO
----

- don't leak memory in LCRQ (by e.g. using hazard pointers). I tried using
  [crossbeam](https://github.com/aturon/crossbeam) for this, but it doesn't
  seem to fit the use case exactly.
- use compiler intrinsic versions of `compare_and_swap`, `compare_and_swap_2`,
  `test_and_set` and `fetch_and_add` if possible
- see `TODO`s in source code
- store pointers instead of `u64`s

