use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type};

use crate::error::Error;

pub enum Fields<'a> {
    Named(Vec<NamedField<'a>>),
    Unnamed(Vec<UnnamedField<'a>>),
    Unit,
}

impl<'a> Fields<'a> {
    pub fn fields(&'a self) -> impl Iterator<Item = &'a Field<'a>> + 'a {
        match self {
            Fields::Named(fields) => Iter::Named(fields.iter().map(|f| &f.field)),
            Fields::Unnamed(fields) => Iter::Unnamed(fields.iter().map(|f| &f.field)),
            Fields::Unit => Iter::Unit,
        }
    }

    pub fn fields_except<T>(
        &'a self,
        except_idx: usize,
        mut usual: impl FnMut(&'a Field<'a>) -> T + 'a,
        mut exception: impl FnMut(&'a Field<'a>) -> T + 'a,
    ) -> impl Iterator<Item = T> + 'a {
        self.fields().enumerate().map(move |(i, field)| {
            if i == except_idx {
                exception(field)
            } else {
                usual(field)
            }
        })
    }

    pub fn field_definitions(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields().map(
            |Field {
                 field_ident,
                 generic_ident,
                 ..
             }| { quote!(#field_ident: #generic_ident) },
        )
    }

    pub fn default_constructors(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields()
            .map(|Field { field_ident, .. }| quote!(#field_ident: ::builder::NoData::new()))
    }

    pub fn generics(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields()
            .map(|field| field.generic_ident.to_token_stream())
    }

    pub fn no_data_generics(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields()
            .map(|Field { ty, .. }| quote!(::builder::NoData<#ty>))
    }

    pub fn completed_generics(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields().map(|field| field.ty.to_token_stream())
    }
}

impl<'a> TryFrom<&'a syn::Fields> for Fields<'a> {
    type Error = Error;
    fn try_from(fields: &'a syn::Fields) -> Result<Self, Self::Error> {
        match fields {
            syn::Fields::Named(fields) => {
                let mut errors = Vec::new();
                let fields = fields
                    .named
                    .iter()
                    .filter_map(|field| match field.try_into() {
                        Ok(field) => Some(field),
                        Err(error) => {
                            errors.push(error);
                            None
                        }
                    })
                    .collect();
                if errors.is_empty() {
                    Ok(Fields::Named(fields))
                } else {
                    Err(Error::Multiple(errors))
                }
            }
            syn::Fields::Unnamed(fields) => {
                let mut errors = Vec::new();
                let fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|field| match field.try_into() {
                        Ok(field) => Some(field),
                        Err(error) => {
                            errors.push(error);
                            None
                        }
                    })
                    .collect();
                if errors.is_empty() {
                    Ok(Fields::Unnamed(fields))
                } else {
                    Err(Error::Multiple(errors))
                }
            }
            syn::Fields::Unit => Ok(Fields::Unit),
        }
    }
}

enum Iter<Named, Unnamed> {
    Named(Named),
    Unnamed(Unnamed),
    Unit,
}

impl<Named, Unnamed, T> Iterator for Iter<Named, Unnamed>
where
    Named: Iterator<Item = T>,
    Unnamed: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self {
            Iter::Named(ref mut it) => it.next(),
            Iter::Unnamed(ref mut it) => it.next(),
            Iter::Unit => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Named(it) => it.size_hint(),
            Iter::Unnamed(it) => it.size_hint(),
            Iter::Unit => (0, Some(0)),
        }
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, T) -> B,
    {
        match self {
            Iter::Named(it) => it.fold(init, f),
            Iter::Unnamed(it) => it.fold(init, f),
            Iter::Unit => init,
        }
    }
}

pub struct NamedField<'a> {
    pub ident: &'a Ident,
    pub field: Field<'a>,
}

impl<'a> TryFrom<&'a syn::Field> for NamedField<'a> {
    type Error = Error;

    fn try_from(field: &'a syn::Field) -> Result<Self, Self::Error> {
        let ident = field.ident.as_ref().ok_or(Error::MissingIdent)?;
        Ok(NamedField {
            ident,
            field: Field::new(ident, field),
        })
    }
}

pub struct UnnamedField<'a> {
    pub idx: usize,
    pub field: Field<'a>,
}

impl<'a> TryFrom<(usize, &'a syn::Field)> for UnnamedField<'a> {
    type Error = Error;

    fn try_from((idx, field): (usize, &'a syn::Field)) -> Result<Self, Self::Error> {
        if field.ident.is_some() {
            return Err(Error::UnexpectedIdent);
        }
        Ok(UnnamedField {
            idx,
            field: Field::new(&idx, field),
        })
    }
}

pub struct Field<'a> {
    pub field_ident: Ident,
    pub setter: Ident,
    pub builder: Ident,
    pub generic_ident: Ident,
    pub ty: &'a Type,
}

impl<'a> Field<'a> {
    fn new<S: ToString>(suffix: &S, field: &'a syn::Field) -> Self {
        let suffix = suffix.to_string();
        let snake_suffix = suffix.to_case(Case::Snake);
        let camel_suffix = suffix.to_case(Case::UpperCamel);
        Field {
            field_ident: format_ident!("field_{}", snake_suffix),
            setter: format_ident!("set_{}", snake_suffix),
            builder: format_ident!("build_{}", snake_suffix),
            generic_ident: format_ident!("__Field{}", camel_suffix),
            ty: &field.ty,
        }
    }
}
