//! Procedural macro to derive `std::convert::From` and `std::convert::Into` implementations based on field/variant names.

mod convert;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use convert::Convert;

/// Derive an implementaion of `std::convert::From` and `std::convert::Into`.
///
/// ## Examples
///
/// Conversion of structures:
/// ```rust
/// # use convert_by_name::ConvertByName;
/// #[derive(PartialEq, Debug)]
/// struct Point2D {
///     x: i32,
///     y: i32,
/// }
///
/// #[derive(PartialEq, Debug, ConvertByName)]
/// #[from(Point2D)]
/// #[into(Point2D)]
/// struct Vec2D {
///     x: i32,
///     y: i32,
/// }
///
/// let point = Point2D { x: 3, y: 4 };
/// let vector: Vec2D = point.into();
/// assert_eq!(vector, Vec2D { x: 3, y: 4 });
///
/// let point2: Point2D = vector.into();
/// assert_eq!(point2, Point2D { x: 3, y: 4 });
/// ```
///
/// Conversion of enumerations:
/// ```rust
/// # use convert_by_name::ConvertByName;
/// pub mod module_a {
///     pub enum Color {
///         Red,
///         Green,
///         Blue,
///         Rgb { r: u8, g: u8, b: u8 },
///         AnsiValue(u8),
///     }
/// }
///
/// #[derive(PartialEq, Debug, ConvertByName)]
/// #[from(module_a::Color)]
/// #[into(module_a::Color)]
/// pub enum Color {
///     Red,
///     Green,
///     Blue,
///     Rgb { r: u8, g: u8, b: u8 },
///     AnsiValue(u8),
/// }
///
/// let c1 = module_a::Color::Green;
/// let c2: Color = c1.into();
/// assert_eq!(c2, Color::Green);
/// ```
#[proc_macro_derive(ConvertByName, attributes(from, into))]
pub fn convert_by_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    Convert::new(input)
        .map(|c| c.generate_all())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
