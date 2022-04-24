# cache-up

## Summury

Cache toys.

## Example

### Use primitive

```rust
impl Key for i64 {}
impl Value for i64 {}

let mut cache_up = CacheUp::<i64, i64>::new();
let (result, _) = cache_up.execute(1, || 2 + 2);
assert_eq!(result, &4);

let (result, _) = cache_up.execute(1, || 5 + 5);
assert_eq!(result, &4);

let (result, _) = cache_up.execute(2, || 5 + 5);
assert_eq!(result, &10);

let (result, _) = cache_up.execute(2, || 6 + 6);
assert_eq!(result, &10);
```

### Use enum

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
enum Test {
    A,
    B,
    C(String),
}
impl Value for Test {}
impl Key for String {}

let mut cache_up = CacheUp::<String, Test>::new();
let (result, _) = cache_up.execute("aaa".to_string(), || Test::A);
assert_eq!(result, &Test::A);

let (result, _) = cache_up.execute("aaa".to_string(), || Test::B);
assert_eq!(result, &Test::A);

let (result, _) = cache_up.execute("bbb".to_string(), || Test::B);
assert_eq!(result, &Test::B);

let (result, _) = cache_up.execute("ccc".to_string(), || Test::C("inner_ccc".to_string()));
assert_eq!(result, &Test::C("inner_ccc".to_string()));
```

### Use with option

```rust
let mut cache_up = CacheUp::<i64, i64>::new();
let cache_opt = CacheOption::new().add_policy(|_, _, _| true);
let (result, _) = cache_up.execute_with_option(1, || 2 + 2, cache_opt);
assert_eq!(result, &4);

let (result, _) = cache_up.execute(1, || 5 + 5);
assert_eq!(result, &10);

let mut cache_up = CacheUp::<i64, i64>::new();
let cache_opt = CacheOption::new().add_policy(|_, _, _| false);
let (result, _) = cache_up.execute_with_option(1, || 2 + 2, cache_opt);
assert_eq!(result, &4);

let (result, _) = cache_up.execute(1, || 5 + 5);
assert_eq!(result, &4);
```

## Contributors

- [ktanaka101](https://github.com/ktanaka101) - creator, maintainer

## License

MIT
