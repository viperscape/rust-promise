rust-promise
============

for now use the git dependency in your Cargo.toml:
``` toml
[dependencies.promise]

git = "https://github.com/viperscape/rust-promise.git"
```

example usage:
``` rust
extern crate promise;
use promise::Promise;
use std::rand;

fn main () {
    let (pt,pr) = Promise::new();
    let bd = vec![rand::random::<u64>();1000];
    pt.deliver(bd);
    let v = pr.with(|x| x[999]); //copy value, returns inside of Result
    println!("{:?}",v); //Ok(3654177790282180513)
}
```
