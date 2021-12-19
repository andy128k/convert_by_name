//! Procedural macros to derive `std::convert::From` and `std::convert::Into` implementations based on field/variant names.

mod convert;
mod utils;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, ResultExt};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};

use convert::{convert_data_type, template_from, template_into};
use utils::concat_tokens;

/// Derive an implementaion of `std::convert::From`
///
/// ## Example
///
/// ```rust
/// use convert_by_name::ByNameFrom;
///
/// struct Point2D {
///     x: i32,
///     y: i32,
/// }
///
/// #[derive(PartialEq, Debug, ByNameFrom)]
/// #[by_name_from(Point2D)]
/// struct Vec2D {
///     x: i32,
///     y: i32,
/// }
///
/// let point = Point2D { x: 3, y: 4 };
/// let vector = Vec2D::from(point); // `from` is derived
/// assert_eq!(vector, Vec2D { x: 3, y: 4 });
/// ```
#[proc_macro_error]
#[proc_macro_derive(ByNameFrom, attributes(by_name_from))]
pub fn by_name_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("by_name_from"))
        .map(|attr| {
            let src_type = attr.parse_args::<Path>().unwrap_or_abort();
            let dst_type = quote! { Self };
            let src_value = quote! { value };
            let body = convert_data_type(&input.data, &src_type, &dst_type, &src_value);

            template_from(&input.ident, &input.generics, &src_type, body)
        })
        .fold(quote!(), concat_tokens)
        .into()
}

/// Derive an implementaion of `std::convert::Into`
///
/// ## Example
///
/// ```rust
/// use convert_by_name::ByNameInto;
///
/// #[derive(ByNameInto)]
/// #[by_name_into(Vec2D)]
/// struct Point2D {
///     x: i32,
///     y: i32,
/// }
///
/// #[derive(PartialEq, Debug)]
/// struct Vec2D {
///     x: i32,
///     y: i32,
/// }
///
/// let point = Point2D { x: 3, y: 4 };
/// let vector: Vec2D = point.into(); // `into` is derived
/// assert_eq!(vector, Vec2D { x: 3, y: 4 });
/// ```
#[proc_macro_error]
#[proc_macro_derive(ByNameInto, attributes(by_name_into))]
pub fn by_name_into(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("by_name_into"))
        .map(|attr| {
            let src_type = quote! { Self };
            let dst_type = attr.parse_args::<Path>().unwrap_or_abort();
            let src_value = quote! { self };
            let body = convert_data_type(&input.data, &src_type, &dst_type, &src_value);

            template_into(&input.ident, &input.generics, &dst_type, body)
        })
        .fold(quote!(), concat_tokens)
        .into()
}
