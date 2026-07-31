use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, ConstParam, Data, DataEnum, DataStruct, DeriveInput, Error as SynError, Fields,
    FieldsNamed, FieldsUnnamed, GenericArgument, GenericParam, Generics, Ident, LifetimeParam,
    Result as SynResult, Token, Type, TypeParam, token::Comma,
};

use crate::utils::concat_tokens;

pub enum ConvertData {
    Struct(DataStruct),
    Enum(DataEnum),
}

pub enum ConvertOpts {
    From(Type),
    Into(Type),
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
            let src_type = attr.parse_args::<Type>()?;
            Ok(Some(ConvertOpts::From(src_type)))
        } else if attr.path().is_ident("into") {
            let dst_type = attr.parse_args::<Type>()?;
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

    pub fn src_generics(&self) -> Vec<GenericParam> {
        match self {
            Self::From(source) => extract_generics_from_type(source),
            Self::Into(_) => vec![],
        }
    }

    pub fn dst_type(&self) -> TokenStream {
        match self {
            Self::From(_) => quote!(Self),
            Self::Into(destination) => destination.to_token_stream(),
        }
    }

    pub fn dst_geenerics(&self) -> Vec<GenericParam> {
        match self {
            Self::From(_) => vec![],
            Self::Into(destination) => extract_generics_from_type(destination),
        }
    }
}

/// Recursive function to extract all generic types from a Type
fn extract_generics_from_type(ty: &Type) -> Vec<GenericParam> {
    let mut generics = Vec::<GenericParam>::new();

    fn process(ty: &Type, out: &mut Vec<GenericParam>) {
        match ty {
            Type::Path(type_path) => {
                for segment in &type_path.path.segments {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        for arg in &args.args {
                            match arg {
                                GenericArgument::Lifetime(lifetime) => {
                                    out.push(GenericParam::Lifetime(LifetimeParam::new(
                                        lifetime.clone(),
                                    )));
                                }
                                GenericArgument::Type(ty) => {
                                    out.push(GenericParam::Type(TypeParam::from(ty.ident())));
                                    process(ty, out);
                                }
                                // GenericArgument::Const(expr) => {
                                //     out.push(GenericParam::Const(ConstParam::from(expr.clone())));
                                // }
                                GenericArgument::Const(..)
                                | GenericArgument::AssocType(..)
                                | GenericArgument::AssocConst(..)
                                | GenericArgument::Constraint(..) => {}
                            }
                        }
                    }
                }
            }
            Type::Tuple(tuple) => {
                for elem in &tuple.elems {
                    process(elem, out);
                }
            }
            Type::Slice(slice) => {
                process(&slice.elem, out);
            }
            Type::Array(arr) => {
                process(&arr.elem, out);
            }
            Type::Reference(refr) => {
                process(&refr.elem, out);
            }
            _ => {}
        }
    }

    process(ty, &mut generics);
    generics
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

        let src_type_generics = opts.src_generics();
        let dst_type_generics = opts.dst_geenerics();

        match opts {
            ConvertOpts::From(src_type) => template_from(
                &self.ident,
                &self.generics,
                src_type_generics,
                dst_type_generics,
                src_type,
                body,
            ),
            ConvertOpts::Into(dst_type) => template_into(
                &self.ident,
                &self.generics,
                src_type_generics,
                dst_type_generics,
                dst_type,
                body,
            ),
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
                        #field_ident: core::convert::Into::into(#field_ident)
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
                        core::convert::Into::into(#field_ident)
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
    src_type_generics: Vec<GenericParam>,
    dst_type_generics: Vec<GenericParam>,
    src_type: &Type,
    body: TokenStream,
) -> TokenStream {
    let mut generics = generics.clone();
    for t in src_type_generics {
        generics.params.push(t);
    }
    for t in dst_type_generics {
        generics.params.push(t);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics core::convert::From<#src_type> for #ident #ty_generics #where_clause {
            fn from(value: #src_type) -> Self {
                #body
            }
        }
    }
}

pub fn template_into(
    ident: &Ident,
    generics: &Generics,
    src_type_generics: Vec<GenericParam>,
    dst_type_generics: Vec<GenericParam>,
    dst_type: &Type,
    body: TokenStream,
) -> TokenStream {
    let mut generics = generics.clone();
    for t in src_type_generics {
        generics.params.push(t);
    }
    for t in dst_type_generics {
        generics.params.push(t);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[allow(clippy::from_over_into)]
        impl #impl_generics core::convert::Into<#dst_type> for #ident #ty_generics #where_clause {
            fn into(self) -> #dst_type {
                let value = self;
                #body
            }
        }
    }
}
