use convert_by_name::ConvertByName;

#[test]
fn test_from_struct() {
    struct Point2D {
        x: i32,
        y: i32,
    }

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(Point2D)]
    struct Vec2D {
        x: i32,
        y: i32,
    }

    assert_eq!(Vec2D::from(Point2D { x: 3, y: 4 }), Vec2D { x: 3, y: 4 });
}

#[test]
fn test_from_tuple_struct() {
    struct Point2D(i32, i32);

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(Point2D)]
    struct Vec2D(i32, i32);

    assert_eq!(Vec2D::from(Point2D(3, 4)), Vec2D(3, 4));
}

#[test]
fn test_from_nested_conversions() {
    struct Point2D(i32, i32);

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(Point2D)]
    struct Vec2D(f64, f64);

    assert_eq!(Vec2D::from(Point2D(3, 4)), Vec2D(3.0, 4.0));
}

#[test]
fn test_from_plain_enum() {
    enum ColorSrc {
        Red,
        Green,
        Blue,
    }

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(ColorSrc)]
    enum ColorDst {
        Red,
        Green,
        Blue,
    }

    assert_eq!(ColorDst::from(ColorSrc::Red), ColorDst::Red);
    assert_eq!(ColorDst::from(ColorSrc::Green), ColorDst::Green);
    assert_eq!(ColorDst::from(ColorSrc::Blue), ColorDst::Blue);
}

#[test]
fn test_from_enum() {
    enum ColorSrc {
        Red(i32),
        Green { level: i32 },
        Blue,
    }

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(ColorSrc)]
    enum ColorDst {
        Red(f64),
        Green { level: i32 },
        Blue,
    }

    assert_eq!(ColorDst::from(ColorSrc::Red(4)), ColorDst::Red(4.0));
    assert_eq!(
        ColorDst::from(ColorSrc::Green { level: 12 }),
        ColorDst::Green { level: 12 }
    );
    assert_eq!(ColorDst::from(ColorSrc::Blue), ColorDst::Blue);
}

#[test]
fn test_from_struct_generic() {
    struct Point2D<T: Copy> {
        x: T,
        y: T,
    }

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(Point2D::<T>)]
    struct Vec2D<T: Copy> {
        x: T,
        y: T,
    }

    assert_eq!(
        Vec2D::from(Point2D::<i32> { x: 3, y: 4 }),
        Vec2D::<i32> { x: 3, y: 4 }
    );

    assert_eq!(
        Vec2D::from(Point2D::<&str> {
            x: "three",
            y: "four"
        }),
        Vec2D::<&str> {
            x: "three",
            y: "four"
        }
    );
}

#[test]
fn test_from_struct_generic_with_lifetime() {
    struct Point2D<'a, T> {
        x: &'a T,
        y: &'a T,
    }

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(Point2D::<'a, T>)]
    struct Vec2D<'a, T> {
        x: &'a T,
        y: &'a T,
    }

    assert_eq!(
        Vec2D::from(Point2D::<i32> { x: &3, y: &4 }),
        Vec2D::<i32> { x: &3, y: &4 }
    );
}

#[test]
fn test_from_struct_generic_with_lifetime_str() {
    struct Person<'a> {
        first_name: &'a str,
        last_name: &'a str,
    }

    #[derive(PartialEq, Debug, ConvertByName)]
    #[from(Person::<'a>)]
    struct User {
        first_name: String,
        last_name: String,
    }

    assert_eq!(
        User::from(Person {
            first_name: "three",
            last_name: "four",
        }),
        User {
            first_name: "three".to_owned(),
            last_name: "four".to_owned(),
        }
    );
}
