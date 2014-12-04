rust-promise
============

```
let p: Promise<int> = Promise::new();
let p2 = p.clone();
spawn(proc() {
  assert_eq!(p2.deliver(1),true);
});
assert_eq!(p.apply(|x| *x).unwrap(),1); //waits on spawned thread
```
