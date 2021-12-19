mod convert;
mod utils;

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, ResultExt};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};

use convert::{convert_data_type, template_from, template_into};
use utils::concat_tokens;

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
