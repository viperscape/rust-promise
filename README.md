promiser
============

``` toml
promiser = "0.0.5"
```

example usage:
``` rust
extern crate promiser;
use promiser::Promise;
use std::rand;

fn main () {
    let (pt,pr) = Promise::new();
    let bd = vec![rand::random::<u64>();1000];
    pt.deliver(bd);
    let v = pr.with(|x| x[999]); //copy value, returns inside of Result
    println!("{:?}",v); //Ok(3654177790282180513)
}
```

#### benchmarks ####
These represent some basic benchmarking, real speeds could vary significantly. 
```
test latch::tests::bench_latch            ... bench:        45 ns/iter (+/- 0)
test promise::tests::bench_channel_linear ... bench:       758 ns/iter (+/- 7)
test promise::tests::bench_promise_linear ... bench:         1 ns/iter (+/- 0)
```
