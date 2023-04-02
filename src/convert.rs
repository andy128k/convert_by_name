use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error as SynError, Fields, FieldsNamed,
    FieldsUnnamed, Generics, Ident, Path, Result as SynResult,
};

use crate::utils::concat_tokens;

pub enum ConvertData {
    Struct(DataStruct),
    Enum(DataEnum),
}

pub enum ConvertOpts {
    From(Path),
    Into(Path),
}

pub struct Convert {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ConvertData,
    pub opts: Vec<ConvertOpts>,
}

impl ConvertData {
    pub fn from_data(data: Data) -> SynResult<Self> {
        match data {
            Data::Struct(s) => Ok(ConvertData::Struct(s)),
            Data::Enum(e) => Ok(ConvertData::Enum(e)),
            Data::Union(u) => Err(SynError::new(
                u.union_token.span,
                "Deriving of ConvertByName is not supported for union types.",
            )),
        }
    }
}

impl ConvertOpts {
    pub fn from_attribute(attr: &Attribute) -> SynResult<Option<Self>> {
        if attr.path().is_ident("from") {
            let src_type = attr.parse_args::<Path>()?;
            Ok(Some(ConvertOpts::From(src_type)))
        } else if attr.path().is_ident("into") {
            let dst_type = attr.parse_args::<Path>()?;
            Ok(Some(ConvertOpts::Into(dst_type)))
        } else {
            Ok(None)
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
}

impl Convert {
    pub fn new(input: DeriveInput) -> SynResult<Self> {
        let mut all_opts = vec![];
        for attr in &input.attrs {
            if let Some(opts) = ConvertOpts::from_attribute(attr)? {
                all_opts.push(opts);
            }
        }
        if all_opts.is_empty() {
            return Err(SynError::new(
                input.ident.span(),
                "Deriving of ConvertByName requires at least one `from`/`into` attribute.",
            ));
        }
        Ok(Self {
            ident: input.ident,
            generics: input.generics,
            data: ConvertData::from_data(input.data)?,
            opts: all_opts,
        })
    }

    fn generate(&self, opts: &ConvertOpts) -> TokenStream {
        let src_type = opts.src_type();
        let dst_type = opts.dst_type();

        let body = match self.data {
            ConvertData::Struct(ref d) => convert_struct(d, src_type, dst_type),
            ConvertData::Enum(ref d) => convert_enum(d, src_type, dst_type),
        };

        match opts {
            ConvertOpts::From(src_type) => {
                template_from(&self.ident, &self.generics, src_type, body)
            }
            ConvertOpts::Into(dst_type) => {
                template_into(&self.ident, &self.generics, dst_type, body)
            }
        }
    }

    pub fn generate_all(&self) -> TokenStream {
        self.opts
            .iter()
            .map(|opts| self.generate(opts))
            .fold(quote!(), concat_tokens)
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
