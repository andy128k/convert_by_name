use convert_by_name::ByNameFrom;

#[test]
fn test_from_struct() {
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

    assert_eq!(Vec2D::from(Point2D { x: 3, y: 4 }), Vec2D { x: 3, y: 4 });
}

#[test]
fn test_from_tuple_struct() {
    struct Point2D(i32, i32);

    #[derive(PartialEq, Debug, ByNameFrom)]
    #[by_name_from(Point2D)]
    struct Vec2D(i32, i32);

    assert_eq!(Vec2D::from(Point2D(3, 4)), Vec2D(3, 4));
}

#[test]
fn test_from_nested_conversions() {
    struct Point2D(i32, i32);

    #[derive(PartialEq, Debug, ByNameFrom)]
    #[by_name_from(Point2D)]
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

    #[derive(PartialEq, Debug, ByNameFrom)]
    #[by_name_from(ColorSrc)]
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

    #[derive(PartialEq, Debug, ByNameFrom)]
    #[by_name_from(ColorSrc)]
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
