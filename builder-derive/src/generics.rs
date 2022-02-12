use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{ConstParam, Ident, LifetimeDef, TypeParam};

pub struct Generics<'a> {
    lifetimes: Vec<&'a LifetimeDef>,
    types: Vec<Generic<'a>>,
    consts: Vec<&'a ConstParam>,
}

impl<'a> Generics<'a> {
    pub fn field_definitions(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.types.iter().map(
            |Generic {
                 field_name,
                 field_type,
                 ..
             }| quote!(#field_name: #field_type),
        )
    }

    pub fn default_constructors(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.types
            .iter()
            .map(|Generic { field_name, .. }| quote!(#field_name: ::core::marker::PhantomData))
    }

    pub fn lifetimes(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.lifetimes.iter().map(ToTokens::to_token_stream)
    }

    pub fn types(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.types.iter().map(|ty| ty.param.to_token_stream())
    }

    pub fn consts(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.consts.iter().map(ToTokens::to_token_stream)
    }
}

impl<'a> From<&'a syn::Generics> for Generics<'a> {
    fn from(generics: &'a syn::Generics) -> Self {
        Generics {
            lifetimes: generics.lifetimes().collect(),
            types: generics.type_params().map(Generic::from).collect(),
            consts: generics.const_params().collect(),
        }
    }
}

pub struct Generic<'a> {
    param: &'a TypeParam,
    field_type: TokenStream,
    field_name: Ident,
}

impl<'a> From<&'a syn::TypeParam> for Generic<'a> {
    fn from(param: &'a TypeParam) -> Self {
        let ident = &param.ident;
        Generic {
            param,
            field_type: quote!(::core::marker::PhantomData<#ident>),
            field_name: format_ident!("_generic_{}", ident.to_string().to_case(Case::Snake)),
        }
    }
}
