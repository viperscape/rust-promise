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

fn main () {
    let p: Promise<int> = Promise::new();
    let p2 = p.clone();
    spawn(proc() {
        println!("task2: promise result {}", (p.apply(|x| *x).unwrap())); //waits on spawned thread
    });
    println!("task1: promise delivered {}",p2.deliver(1));

    let p: Promise<int> = Promise::new();
    let p2 = p.clone();
    spawn(proc() {
        println!("task2: promise delivered {}",p2.deliver(5));
    });

    println!("task1: promise result {}", (p.apply(|x| *x-1).unwrap())); //waits on spawned thread
}

//task1: promise delivered true
//task2: promise result 1

//task2: promise delivered true
//task1: promise result 4

```
