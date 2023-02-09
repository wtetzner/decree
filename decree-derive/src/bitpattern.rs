use std::collections::{HashSet, HashMap};

use logos::{Logos, Lexer};

use proc_macro::TokenStream as TokenStreamExternal;
use proc_macro2::{Ident, TokenTree, Span};
use syn::{spanned::Spanned, Attribute, Expr, Fields, Lit, ExprLit, ExprPath};
use quote::{quote, quote_spanned};

pub fn compute_mapping(fields: &Fields, pattern: &BitPattern) -> syn::Result<HashMap<RawMappingValue, Expr>> {
    let mut existing_fields = HashSet::new();
    let mut mapping = HashMap::new();
    let mut index = 0;
    for field in fields {
        if let Some(ident) = &field.ident {
            let name = RawMappingValue::Name(ident.to_string());
            existing_fields.insert(name.clone());
            let expr: Expr = {
                let ident = Ident::new_raw(&format!("____{}", ident), ident.span());
                let tokens: TokenStreamExternal = quote! { #ident }.into();
                syn::parse(tokens)?
            };
            mapping.insert(name, expr);
        } else {
            let name = RawMappingValue::Index(index);
            existing_fields.insert(name.clone());
            let expr: Expr = {
                let ident = Ident::new_raw(&format!("_____{}", index), field.span());
                let tokens: TokenStreamExternal = quote! { #ident }.into();
                syn::parse(tokens)?
            };
            mapping.insert(name, expr);
        }
        index += 1;
    }
    for (new_name, field_name) in pattern.renames.iter() {
        let field_value: RawMappingValue = RawMappingValue::from(field_name.clone());
        if !existing_fields.contains(&field_value) {
            return Err(syn::Error::new(field_name.span(), format!("\"{}\" isn't a valid field ({} = {})", field_name, new_name, field_name)));
        }
    }
    for (new_name, field_name) in pattern.renames.iter() {
        let name = RawMappingValue::Name(new_name.to_string());
        if !mapping.contains_key(&name) {
            mapping.insert(name, field_name.clone().into());
        } else {
            let value = &mapping[&name];
            let existing = quote! { #value };
            return Err(syn::Error::new(new_name.span(), format!("\"{}\" is already mapped to \".{}\"", new_name, existing)));
        }
    }
    Ok(mapping)
}

pub fn find_bit_pattern(attrs: &[Attribute]) -> syn::Result<Option<BitPattern>> {
    for attr in attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident.to_string() == "bitpattern" {
                if let Some(TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() {
                    let tokens: TokenStreamExternal = group.stream().into();
                    let pattern: BitPattern = syn::parse(tokens)?;
                    return Ok(Some(pattern));
                }
            }
        }
    }
    Ok(None)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Range {
    pub name: String,
    pub start: usize,
    pub len: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Literal {
    pub bytes: Vec<u8>,
    pub bits: usize,
}

#[derive(Logos, Debug, PartialEq, Eq)]
pub enum Token {
    #[regex("[01]+", |lex| parse_literal(lex.slice()))]
    Bits(Literal),

    #[regex(r#"\[([a-zA-Z0-9_]+):\d+(-\d+)?\]"#, parse_range)]
    Range(Range),

    #[error]
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum RawMappingValue {
    Index(usize),
    Name(String),
}

impl std::fmt::Display for RawMappingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawMappingValue::Index(index) => write!(f, "{}", index),
            RawMappingValue::Name(name) => write!(f, "{}", name),
        }
    }
}

impl From<MappingValue> for RawMappingValue {
    fn from(value: MappingValue) -> Self {
        match value {
            MappingValue::Index(_, value) => RawMappingValue::Index(value),
            MappingValue::Ident(value) => RawMappingValue::Name(value.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MappingValue {
    Index(Span, usize),
    Ident(Ident),
}

impl From<MappingValue> for Expr {
    fn from(value: MappingValue) -> Self {
        match value {
            MappingValue::Index(span, index) => {
                let ident = Ident::new_raw(&format!("_____{}", index), span.clone());
                let tokens: TokenStreamExternal = quote_spanned!(span => #ident).into();
                syn::parse(tokens).unwrap()
            },
            MappingValue::Ident(ident) => {
                let ident = Ident::new_raw(&format!("____{}", ident), ident.span());
                let tokens: TokenStreamExternal = quote!(#ident).into();
                syn::parse(tokens).unwrap()
            },
        }
    }
}

impl std::fmt::Display for MappingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingValue::Index(_, index) => write!(f, "{}", index),
            MappingValue::Ident(ident) => write!(f, "{}", ident),
        }
    }
}

impl MappingValue {
    pub fn span(&self) -> Span {
        match self {
            MappingValue::Index(span, _) => span.clone(),
            MappingValue::Ident(ident) => ident.span(),
        }
    }
}

impl syn::parse::Parse for MappingValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        let span = expr.span();
        match expr {
            Expr::Lit(ExprLit { lit, .. }) => {
                let index: usize = match lit {
                    Lit::Int(lit_int) => lit_int.base10_parse()?,
                    _ => return Err(syn::Error::new(span, "Expected a field index or name")),
                };
                Ok(MappingValue::Index(span, index))
            },
            Expr::Path(ExprPath { path, .. }) => {
                if let Some(ident) = path.get_ident() {
                    Ok(MappingValue::Ident(ident.clone()))
                } else {
                    Err(syn::Error::new(span, "Expected a field index or name"))
                }
            },
            _ => return Err(syn::Error::new(span, "Expected a field index or name")),
        }
    }
}

#[derive(Debug)]
pub struct BitPattern {
    pub tokens: Vec<Token>,
    pub renames: Vec<(Ident, MappingValue)>,
}

impl BitPattern {
    pub fn bits(&self) -> usize {
        let mut bits = 0;
        for token in &self.tokens {
            match token {
                Token::Bits(literal) => bits += literal.bits,
                Token::Range(range) => bits += range.len,
                Token::Error => {},
            }
        }
        bits
    }

    pub fn referenced_values(&self) -> HashSet<RawMappingValue> {
        let mut values = HashSet::new();
        let renames = {
            let mut renames = HashMap::new();
            for (ident, value) in &self.renames {
                renames.insert(ident.to_string(), value.clone());
            }
            renames
        };
        for token in &self.tokens {
            if let Token::Range(Range { name, .. }) = token {
                if let Ok(index) = name.parse::<usize>() {
                    values.insert(RawMappingValue::Index(index));
                } else {
                    if let Some(mapping_value) = renames.get(name) {
                        let mapping_value: RawMappingValue = mapping_value.clone().into();
                        values.insert(mapping_value);
                    } else {
                        values.insert(RawMappingValue::Name(name.to_string()));
                    }
                }
            }
        }
        values
    }
}

impl syn::parse::Parse for BitPattern {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        fn parse_mapping(input: &mut syn::parse::ParseStream) -> syn::Result<(Ident, MappingValue)> {
            let ident: Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let value: MappingValue = input.parse()?;
            Ok((ident, value))
        }
        let mut input = input;
        let expr: Expr = input.parse()?;
        let pattern = string_lit(&expr, "a bit pattern")?;
        let mut pattern = parse_bit_pattern(expr.span(), &pattern)?;
        while !input.is_empty() {
            let _comma: syn::Token![,] = input.parse()?;
            pattern.renames.push(parse_mapping(&mut input)?);
        }
        Ok(pattern)
    }
}

pub fn string_lit(lit: &Expr, kind: &str) -> syn::Result<String> {
    match lit {
        Expr::Lit(ExprLit { lit: Lit::Str(string), ..}) => Ok(string.value()),
        _ => Err(syn::Error::new(lit.span(), format!("Expected a string literal expressing {}.", kind))),
    }
}

pub fn parse_bit_pattern(span: Span, pattern: &str) -> syn::Result<BitPattern> {
    let mut lexer = Token::lexer(pattern);
    let mut results = vec![];
    while let Some(token) = lexer.next() {
        if token == Token::Error {
            let tspan = lexer.span();
            let start = tspan.start;
            let end = tspan.end;
            return Err(syn::Error::new(span, &format!("Unexpected token at {}-{}: {}", tspan.start, tspan.end, &pattern[start..end])));
        }
        results.push(token);
    }
    Ok(BitPattern { tokens: results, renames: vec![] })
}

pub fn parse_range(lexer: &mut Lexer<Token>) -> Range {
    let slice: &str = lexer.slice();
    let mut colon = None;
    let mut hyphen: Option<usize> = None;
    let mut index = 0;
    for chr in slice.chars() {
        if chr == ':' {
            colon = Some(index);
        }
        if colon.is_some() && chr == '-' {
            hyphen = Some(index);
        }
        index += 1;
    }
    let colon = colon.unwrap();
    if let Some(hyphen) = hyphen {
        let start: usize = slice[(colon + 1)..hyphen].parse().unwrap();
        let end: usize = slice[(hyphen + 1)..(slice.len() - 1)].parse().unwrap();
        Range {
            name: slice[1..colon].to_string(),
            start,
            len: end + 1 - start,
        }
    } else {
        let start: usize = slice[(colon + 1)..(slice.len() - 1)].parse().unwrap();
        Range {
            name: slice[1..colon].to_string(),
            start,
            len: 1,
        }
    }
}

pub fn parse_literal(string: &str) -> Literal {
    Literal {
        bytes: parse_bits(string),
        bits: string.len(),
    }
}

pub fn parse_bits(bits: &str) -> Vec<u8> {
    let mut results = vec![];
    let mut len = bits.len();
    while len > 8 {
        let end = &bits[(len - 8)..];
        results.push(parse_bits_to_byte(end));
        len -= 8;
    }
    if len > 0 {
        results.push(parse_bits_to_byte(&bits[..len]));
    }
    results
}

pub fn parse_bits_to_byte(bits: &str) -> u8 {
    let mut result = 0;
    for bit in bits.chars() {
        result = result << 1;
        if bit == '1' {
            result |= 1;
        }
    }
    result
}
