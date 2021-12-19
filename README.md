# Convert by name

[![Crates.io Status](https://img.shields.io/crates/v/convert_by_name.svg)](https://crates.io/crates/convert_by_name)
[![docs.rs](https://img.shields.io/docsrs/convert_by_name)](https://docs.rs/convert_by_name)

Procedural macros to derive `std::convert::From` and `std::convert::Into` implementations based on field/variant names.
The crate supports `struct`s and `enum`s only. `union`s are not supported.

## Examples

### Deriving `From`

```rust
#[derive(PartialEq, Debug)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Debug, ConvertByName)]
#[from(Point2D)]
#[into(Point2D)]
struct Vec2D {
    x: i32,
    y: i32,
}

let point = Point2D { x: 3, y: 4 };

let vector: Vec2D = point.into();
assert_eq!(vector, Vec2D { x: 3, y: 4 });

let point2: Point2D = vector.into();
assert_eq!(point2, Point2D { x: 3, y: 4 });
```
