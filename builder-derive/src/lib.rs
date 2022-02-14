use std::iter::once;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Ident, Visibility};

mod error;
mod fields;
mod generics;

use error::Error;
use fields::Fields;
use generics::Generics;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let builder = StructAttrs::try_from(&input).unwrap();
    let builder_with_callback = builder.builder_with_callback();
    quote! {
        #builder_with_callback

        #builder
    }
    .into()
}

struct StructAttrs<'a> {
    vis: &'a Visibility,
    ident: &'a Ident,
    builder_ident: Ident,
    callback: Ident,
    generics: Generics<'a>,
    fields: Fields<'a>,
}

impl<'a> StructAttrs<'a> {
    fn builder_generics(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.generics
            .lifetimes()
            .chain(self.generics.types())
            .chain(once(self.callback.to_token_stream()))
            .chain(self.fields.generics())
            .chain(self.generics.consts())
    }

    fn impl_generics(
        &'a self,
        other_types: impl IntoIterator<Item = TokenStream> + 'a,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        self.generics
            .impl_generics(once(self.callback.to_token_stream()).chain(other_types))
    }

    fn ty_generics(
        &'a self,
        other_types: impl IntoIterator<Item = TokenStream> + 'a,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        self.generics
            .ty_generics(once(self.callback.to_token_stream()).chain(other_types))
    }

    fn where_clause(&'a self) -> TokenStream {
        let where_predicates = self.generics.where_predicates();
        let built_type = self.ident;
        let ty_generics = self.generics.ty_generics(None);
        let callback = &self.callback;
        quote!(where #(#where_predicates,)* #callback: ::builder::Callback<#built_type <#(#ty_generics,)*>>)
    }

    fn builder_with_callback(&self) -> TokenStream {
        let callback = &self.callback;
        let impl_generics = self.impl_generics(None);
        let ident = &self.ident;
        let ty_generics = self.generics.ty_generics(None);
        let where_clause = self.where_clause();

        let builder_ident = &self.builder_ident;
        let builder_generics = self.ty_generics(self.fields.no_data_generics());
        quote! {
            #[automatically_derived]
            impl <#(#impl_generics,)*> builder::BuilderWithCallback<#callback> for #ident <#(#ty_generics,)*> #where_clause
            {
                type CallbackBuilder = #builder_ident <#(#builder_generics,)*>;

                fn builder_with_callback(callback: #callback) -> Self::CallbackBuilder {
                    #builder_ident::new(callback)
                }
            }
        }
    }

    fn default_constructor(&self) -> TokenStream {
        let callback = &self.callback;
        let impl_generics = self.impl_generics(None);
        let builder_ident = &self.builder_ident;
        let ty_generics = self.ty_generics(self.fields.no_data_generics());
        let where_clause = self.where_clause();

        let generic_fields = self.generics.default_constructors();
        let fields = self.fields.default_constructors();
        quote! {
            #[automatically_derived]
            impl <#(#impl_generics),*> #builder_ident <#(#ty_generics),*> #where_clause {
                fn new(callback: #callback) -> Self {
                    Self {
                        #(#generic_fields,)*
                        callback,
                        #(#fields,)*
                    }
                }
            }
        }
    }

    fn setters(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields.fields().enumerate().map(|(i, field)| {
            let impl_generics =
                self.impl_generics(self.fields.fields().enumerate().filter_map(|(j, field)| {
                    if i == j {
                        // this is the field we are writing the impl for; it is not generic here
                        None
                    } else {
                        Some(field.generic_ident.to_token_stream())
                    }
                }));
            let builder_ident = &self.builder_ident;
            let ty_generics =
                self.ty_generics(self.fields.fields().enumerate().map(|(j, field)| {
                    if i == j {
                        // this is the field we are writing the impl for; fill in the concrete
                        // `NoData` type
                        let ty = &field.ty;
                        quote!(::builder::NoData<#ty>)
                    } else {
                        field.generic_ident.to_token_stream()
                    }
                }));
            let setter = &field.setter;
            let ty = field.ty;
            let out_ty_generics =
                self.ty_generics(self.fields.fields().enumerate().map(|(j, field)| {
                    if i == j {
                        // this is the field we are writing the impl for; fill in the concrete type
                        ty.to_token_stream()
                    } else {
                        field.generic_ident.to_token_stream()
                    }
                }));
            let generic_fields = self.generics.default_constructors();
            let fields = self.fields.fields().enumerate().map(|(j, field)| {
                let field_ident = &field.field_ident;
                if i == j {
                    // this is the field we are writing the impl for; fill in `value`
                    quote!(#field_ident: value)
                } else {
                    // otherwise propagate self value
                    quote!(#field_ident: self.#field_ident)
                }
            });
            quote! {
                #[automatically_derived]
                impl <#(#impl_generics),*> #builder_ident <#(#ty_generics),*> {
                    fn #setter(self, value: #ty) -> #builder_ident <#(#out_ty_generics),*> {
                        #builder_ident {
                            #(#generic_fields,)*
                            callback: self.callback,
                            #(#fields,)*
                        }
                    }
                }
            }
        })
    }

    fn build(&self) -> TokenStream {
        let callback = &self.callback;
        let impl_generics = self.impl_generics(None);
        let builder_ident = &self.builder_ident;
        let builder_ty_generics = self.ty_generics(self.fields.completed_generics());
        let where_clause = self.where_clause();

        let ident = self.ident;

        let fields = match &self.fields {
            Fields::Named(fields) => {
                let fields = fields.iter().map(|field| {
                    let ident = field.ident;
                    let field_ident = &field.field.field_ident;
                    quote!(#ident: self.#field_ident)
                });
                quote!({ #(#fields),* })
            }
            Fields::Unnamed(fields) => {
                let fields = fields.iter().map(|field| {
                    let field_ident = &field.field.field_ident;
                    quote!(self.#field_ident)
                });
                quote!((#(#fields),*))
            }
            Fields::Unit => quote!(),
        };
        quote! {
            #[automatically_derived]
            impl <#(#impl_generics),*> #builder_ident <#(#builder_ty_generics),*> #where_clause {
                fn build(self) -> #callback::Output {
                    self.callback.callback(#ident #fields)
                }
            }
        }
    }
}

impl<'a> ToTokens for StructAttrs<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = self.vis;
        let builder_ident = &self.builder_ident;
        let builder_generics = self.builder_generics();

        let generic_markers = self.generics.field_definitions();
        let callback = &self.callback;
        let fields = self.fields.field_definitions();
        let default_constructor = self.default_constructor();
        let setters = self.setters();
        let build = self.build();
        let stream = quote! {
            #[automatically_derived]
            #vis struct #builder_ident <#(#builder_generics),*> {
                #(#generic_markers,)*
                callback: #callback,
                #(#fields,)*
            }

            #default_constructor

            #(#setters)*

            #build
        };
        tokens.extend(stream)
    }
}

impl<'a> TryFrom<&'a DeriveInput> for StructAttrs<'a> {
    type Error = Error;

    fn try_from(input: &'a DeriveInput) -> Result<Self, Self::Error> {
        let builder_ident = quote::format_ident!("{}Builder", &input.ident);
        match &input.data {
            syn::Data::Struct(data) => Ok(StructAttrs {
                vis: &input.vis,
                ident: &input.ident,
                builder_ident,
                callback: syn::parse_quote!(__Callback),
                generics: (&input.generics).into(),
                fields: (&data.fields).try_into()?,
            }),
            syn::Data::Enum(_) => Err(Error::InvalidShape("struct", "enum")),
            syn::Data::Union(_) => Err(Error::InvalidShape("struct", "union")),
        }
    }
}
