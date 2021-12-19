use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn concat_tokens(t1: impl ToTokens, t2: impl ToTokens) -> TokenStream {
    quote! {
        #t1
        #t2
    }
}
