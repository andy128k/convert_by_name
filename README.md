# Convert by name

Procedural macros to derive `std::convert::From` and `std::convert::Into` implementations based on field/variant names.
The crate supports `struct`s and `enum`s only. `union`s are not supported.

## Examples

### Deriving `From`

```rust
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Debug, ByNameFrom)]
#[by_name_from(Point2D)]
struct Vec2D {
    x: i32,
    y: i32,
}

let point = Point2D { x: 3, y: 4 };
let vector = Vec2D::from(point); // `from` is derived
assert_eq!(vector, Vec2D { x: 3, y: 4 });
```

### Deriving `Into`

```rust
#[derive(ByNameInto)]
#[by_name_into(Vec2D)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Debug)]
struct Vec2D {
    x: i32,
    y: i32,
}

let point = Point2D { x: 3, y: 4 };
let vector: Vec2D = point.into();
assert_eq!(vector, Vec2D { x: 3, y: 4 });
```