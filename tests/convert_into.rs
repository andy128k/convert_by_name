use convert_by_name::ConvertByName;

#[test]
fn test_into_struct() {
    #[derive(ConvertByName)]
    #[into(Vec2D)]
    struct Point2D {
        x: i32,
        y: i32,
    }

    #[derive(PartialEq, Debug)]
    struct Vec2D {
        x: i32,
        y: i32,
    }

    let v: Vec2D = Point2D { x: 3, y: 4 }.into();
    assert_eq!(v, Vec2D { x: 3, y: 4 });
}

#[test]
fn test_into_tuple_struct() {
    #[derive(ConvertByName)]
    #[into(Vec2D)]
    struct Point2D(i32, i32);

    #[derive(PartialEq, Debug)]
    struct Vec2D(i32, i32);

    let v: Vec2D = Point2D(3, 4).into();
    assert_eq!(v, Vec2D(3, 4));
}

#[test]
fn test_into_nested_conversions() {
    #[derive(ConvertByName)]
    #[into(Vec2D)]
    struct Point2D(i32, i32);

    #[derive(PartialEq, Debug)]
    struct Vec2D(f64, f64);

    let v: Vec2D = Point2D(3, 4).into();
    assert_eq!(v, Vec2D(3.0, 4.0));
}

#[test]
fn test_into_plain_enum() {
    #[derive(ConvertByName)]
    #[into(ColorDst)]
    enum ColorSrc {
        Red,
        Green,
        Blue,
    }

    #[derive(PartialEq, Debug)]
    enum ColorDst {
        Red,
        Green,
        Blue,
    }

    let red: ColorDst = ColorSrc::Red.into();
    assert_eq!(red, ColorDst::Red);

    let green: ColorDst = ColorSrc::Green.into();
    assert_eq!(green, ColorDst::Green);

    let blue: ColorDst = ColorSrc::Blue.into();
    assert_eq!(blue, ColorDst::Blue);
}

#[test]
fn test_into_enum() {
    #[derive(ConvertByName)]
    #[into(ColorDst)]
    enum ColorSrc {
        Red(i32),
        Green { level: i32 },
        Blue,
    }

    #[derive(PartialEq, Debug)]
    enum ColorDst {
        Red(f64),
        Green { level: i32 },
        Blue,
    }

    let red: ColorDst = ColorSrc::Red(4).into();
    assert_eq!(red, ColorDst::Red(4.0));

    let green: ColorDst = ColorSrc::Green { level: 12 }.into();
    assert_eq!(green, ColorDst::Green { level: 12 });

    let blue: ColorDst = ColorSrc::Blue.into();
    assert_eq!(blue, ColorDst::Blue);
}
