
use std::collections::HashMap;

use proc_macro2::{TokenStream as TokenStreamInternal, Ident};
use syn::{DeriveInput, spanned::Spanned, Expr, DataEnum};
use quote::quote;

use crate::bitpattern::{Range, Token, RawMappingValue, Literal, BitPattern, compute_mapping, find_bit_pattern};

const GENERIC_FAILURE: &str = r#"#[derive(BitSink)] expects an attribute of the form #[bitpattern("11010[a:0-2]0110", a=foo)]"#;

pub fn expand_bit_sink(ast: &DeriveInput) -> syn::Result<TokenStreamInternal> {
    Err(syn::Error::new(ast.span(), "#[derive(BitSink)] is not yet implemented."))
}
