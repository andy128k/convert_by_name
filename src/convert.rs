use proc_macro2::TokenStream;
use proc_macro_error::{abort_call_site, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    Attribute, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Path,
};

pub enum Convert {
    From(Path),
    Into(Path),
}

impl Convert {
    pub fn from_attribute(attr: &Attribute) -> Option<Self> {
        if attr.path.is_ident("from") {
            let src_type = attr.parse_args::<Path>().unwrap_or_abort();
            Some(Convert::From(src_type))
        } else if attr.path.is_ident("into") {
            let dst_type = attr.parse_args::<Path>().unwrap_or_abort();
            Some(Convert::Into(dst_type))
        } else {
            None
        }
    }

    pub fn src_type(&self) -> TokenStream {
        match self {
            Self::From(source) => source.to_token_stream(),
            Self::Into(_) => quote!(Self),
        }
    }

    pub fn dst_type(&self) -> TokenStream {
        match self {
            Self::From(_) => quote!(Self),
            Self::Into(destination) => destination.to_token_stream(),
        }
    }

    pub fn template(&self, ident: &Ident, generics: &Generics, body: TokenStream) -> TokenStream {
        match self {
            Self::From(src_type) => template_from(ident, generics, src_type, body),
            Self::Into(dst_type) => template_into(ident, generics, dst_type, body),
        }
    }

    pub fn generate(&self, input: &DeriveInput) -> TokenStream {
        let body = convert_data_type(&input.data, self.src_type(), self.dst_type());
        self.template(&input.ident, &input.generics, body)
    }
}

pub fn convert_data_type(
    data: &Data,
    src_type: impl ToTokens,
    dst_type: impl ToTokens,
) -> TokenStream {
    match data {
        Data::Struct(ref data_struct) => convert_struct(data_struct, &src_type, dst_type),
        Data::Enum(ref data_enum) => convert_enum(data_enum, &src_type, dst_type),
        Data::Union(..) => {
            abort_call_site!("Deriving convert by name is not supported for union types.");
        }
    }
}

fn convert_struct(
    data_struct: &syn::DataStruct,
    src_type: impl ToTokens,
    dst_type: impl ToTokens,
) -> TokenStream {
    let ConvertParts {
        destruct,
        construct,
    } = fields_convert_parts(&data_struct.fields);

    quote! {
        let #src_type #destruct = value;
        #dst_type #construct
    }
}

fn convert_enum(
    data_enum: &syn::DataEnum,
    src_type: impl ToTokens,
    dst_type: impl ToTokens,
) -> TokenStream {
    let variants: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let parts = fields_convert_parts(&variant.fields);
            let destruct = &parts.destruct;
            let construct = &parts.construct;
            quote! {
                #src_type::#ident #destruct => #dst_type::#ident #construct
            }
        })
        .collect();

    quote! {
        match value {#(
            #variants
        ),*}
    }
}

struct ConvertParts {
    destruct: TokenStream,
    construct: TokenStream,
}

fn fields_convert_parts(fields: &syn::Fields) -> ConvertParts {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let field_ident: Vec<&Ident> = named
                .pairs()
                .map(|pair| {
                    let field = pair.into_value();
                    let ident = field.ident.as_ref().expect("Field ident is specified");
                    ident
                })
                .collect();

            ConvertParts {
                destruct: quote! {
                    {#(
                        #field_ident
                    ),*}
                },
                construct: quote! {
                    {#(
                        #field_ident: std::convert::Into::into(#field_ident)
                    ),*}
                },
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let field_ident: Vec<Ident> = (0..unnamed.len())
                .map(|i| format_ident!("_{}", i))
                .collect();

            ConvertParts {
                destruct: quote! {
                    (#(
                        #field_ident
                    ),*)
                },
                construct: quote! {
                    (#(
                        std::convert::Into::into(#field_ident)
                    ),*)
                },
            }
        }
        Fields::Unit => ConvertParts {
            destruct: quote! {},
            construct: quote! {},
        },
    }
}

pub fn template_from(
    ident: &Ident,
    generics: &Generics,
    src_type: &Path,
    body: TokenStream,
) -> TokenStream {
    quote! {
        impl #generics std::convert::From<#src_type> for #ident #generics {
            fn from(value: #src_type) -> Self {
                #body
            }
        }
    }
}

pub fn template_into(
    ident: &Ident,
    generics: &Generics,
    dst_type: &Path,
    body: TokenStream,
) -> TokenStream {
    quote! {
        #[allow(clippy::from_over_into)]
        impl #generics std::convert::Into<#dst_type> for #ident #generics {
            fn into(self) -> #dst_type {
                let value = self;
                #body
            }
        }
    }
}
