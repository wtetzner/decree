use std::collections::HashSet;

use syn::{DeriveInput, Type, DataEnum};
use proc_macro2::TokenStream as TokenStreamInternal;
use quote::quote;

use crate::bitpattern::{find_bit_pattern, RawMappingValue};

pub fn where_clause(ast: &DeriveInput, constraints: &TokenStreamInternal) -> syn::Result<TokenStreamInternal> {
    let types = collect_types(ast)?;
    let constraints = type_constraints(&types, constraints);
    let where_clause = match &ast.generics.where_clause {
        Some(clause) => quote! {
            #clause, #(#constraints),*
        },
        None => quote! {
            where #(#constraints),*
        },
    };
    Ok(where_clause)
}

pub fn collect_types(ast: &DeriveInput) -> syn::Result<HashSet<Type>> {
    let mut types = HashSet::new();
    match &ast.data {
        syn::Data::Struct(data_struct) => {
            let bitpattern = find_bit_pattern(&ast.attrs)?.expect("Expected bitpattern to exist");
            let values = bitpattern.referenced_values();
            let mut index = 0;
            for field in &data_struct.fields {
                if let Some(ident) = &field.ident {
                    let name = RawMappingValue::Name(ident.to_string());
                    if values.contains(&name) {
                        types.insert(field.ty.clone());
                    }
                } else {
                    let name = RawMappingValue::Index(index);
                    if values.contains(&name) {
                        types.insert(field.ty.clone());
                    }
                }
                index += 1;
            }
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                let bitpattern = find_bit_pattern(&variant.attrs)?.expect("Expected bitpattern to exist");
                let values = bitpattern.referenced_values();
                let mut index = 0;
                for field in &variant.fields {
                    if let Some(ident) = &field.ident {
                        let name = RawMappingValue::Name(ident.to_string());
                        if values.contains(&name) {
                            types.insert(field.ty.clone());
                        }
                    } else {
                        let name = RawMappingValue::Index(index);
                        if values.contains(&name) {
                            types.insert(field.ty.clone());
                        }
                    }
                    index += 1;
                }
            }
        },
        syn::Data::Union(_) => unimplemented!(),
    };
    Ok(types)
}

pub fn type_constraints(types: &HashSet<Type>, constraints: &TokenStreamInternal) -> Vec<TokenStreamInternal> {
    let mut results = vec![];
    for ty in types {
        results.push(quote! {
            #ty: #constraints
        });
    }
    results
}

pub fn generics(ast: &DeriveInput) -> TokenStreamInternal {
    let generics = &ast.generics;
    quote!{ #generics }
}

pub fn generics_names(ast: &DeriveInput) -> TokenStreamInternal {
    let mut vals = vec![];
    for lifetime in ast.generics.lifetimes() {
        let lifetime_name = &lifetime.lifetime;
        vals.push(quote! { #lifetime_name });
    }
    for type_param in ast.generics.type_params() {
        let ident = &type_param.ident;
        vals.push(quote! { #ident });
    }
    for const_param in ast.generics.const_params() {
        let ident = &const_param.ident;
        vals.push(quote! { #ident });
    }

    if vals.is_empty() {
        quote! {}
    } else {
        quote! {
            <#(#vals),*>
        }
    }
}
