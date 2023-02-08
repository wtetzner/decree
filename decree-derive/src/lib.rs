use std::collections::{HashSet, HashMap};

use logos::{Logos, Lexer};

use proc_macro::{TokenStream as TokenStreamExternal};
use proc_macro2::{TokenStream as TokenStreamInternal, Ident, TokenTree, Span};
use syn::{DeriveInput, spanned::Spanned, Attribute, Expr, Fields, Lit, ExprLit, ExprPath, DataEnum};
use quote::{quote, quote_spanned};
use syn::parse_macro_input;

const GENERIC_FAILURE: &str = r#"#[derive(BitSource)] expects an attribute of the form #[bitpattern("11010[a:0-2]0110", a=foo)]"#;

#[proc_macro_derive(BitSource, attributes(bitpattern))]
pub fn bit_source(input: TokenStreamExternal) -> TokenStreamExternal {
    let ast: DeriveInput = parse_macro_input!(input);
    expand_bit_source(&ast)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand_bit_source(ast: &DeriveInput) -> syn::Result<TokenStreamInternal> {
    let type_ident = &ast.ident;
    let type_name = ast.ident.to_string();
    let (body, size_body) = match &ast.data {
        syn::Data::Struct(data_struct) => {
            if let Some(pattern) = find_bit_pattern(&ast.attrs)? {
                let fields = &data_struct.fields;
                let mapping = compute_mapping(fields, &pattern)?;
                let field_bindings = {
                    let mut bindings = vec![];
                    let mut index = 0;
                    for field in fields {
                        if let Some(ident) = &field.ident {
                            let new_ident = Ident::new_raw(&format!("____{}", ident), ident.span());
                            bindings.push(quote!{ let #new_ident = &self.#ident;});
                        } else {
                            let ident = Ident::new_raw(&format!("_____{}", index), field.span());
                            let index = syn::Index::from(index);
                            bindings.push(quote! { let #ident = &self.#index; });
                        };
                        index += 1;
                    }
                    quote! {
                        #(#bindings)*
                    }
                };
                let block = write_source_block(&field_bindings, &pattern, &mapping)?;
                let body = quote! {
                    ::decree::check_input_range(#type_name, start, len, 0, ::decree::BitSource::size(self))?;
                    #block
                };
                let pattern_bits = pattern.bits();
                let size_body = quote! {
                    #pattern_bits
                };
                (body, size_body)
            } else {
                return Err(syn::Error::new(ast.ident.span(), GENERIC_FAILURE))
            }
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let (matches, sizes) = {
                let mut sizes = vec![];
                let mut matches = vec![];
                for variant in variants {
                    if let Some(pattern) = find_bit_pattern(&variant.attrs)? {
                        let enum_type = &ast.ident;
                        let variant_ident = &variant.ident;
                        let mut indexed = false;
                        let bindings = {
                            let mut bindings = vec![];
                            let mut index = 0;
                            for field in &variant.fields { 
                                if field.ident.is_none() {
                                    indexed = true;
                                }
                                if let Some(ident) = &field.ident {
                                    let new_ident = Ident::new_raw(&format!("____{}", ident), ident.span());
                                    bindings.push(quote! { #ident: #new_ident });
                                } else {
                                    let new_ident = Ident::new_raw(&format!("_____{}", index), field.span());
                                    bindings.push(quote! { #new_ident });
                                }
                                index += 1;
                            }
                            bindings
                        };
                        let bindings = if bindings.is_empty() {
                            quote! {}
                        } else if indexed {
                            quote! {
                                (#(#bindings),*)
                            }
                        } else {
                            quote! {
                                { #(#bindings),* }
                            }
                        };
                        let mapping = compute_mapping(&variant.fields, &pattern)?;
                        let block = write_source_block(&quote!{}, &pattern, &mapping)?;
                        let tokens = quote! {
                            #enum_type::#variant_ident #bindings => {
                                #block
                            }
                        };
                        let bit_size = pattern.bits();
                        let size_tokens = quote! {
                            #enum_type::#variant_ident #bindings => {
                                #bit_size
                            }
                        };
                        matches.push(tokens);
                        sizes.push(size_tokens);
                    } else {
                        return Err(syn::Error::new(variant.span(), GENERIC_FAILURE))
                    }
                }
                (matches, sizes)
            };
            let body = quote! {
                match self {
                    #(#matches),*
                }
            };
            let size_body = quote! {
                match self {
                    #(#sizes),*
                }
            };
            (body, size_body)
        },
        syn::Data::Union(_) => return Err(syn::Error::new(ast.span(), "#[derive(BitSource)] is not supported for untagged unions.")),
    };
    Ok(quote! {
        impl ::decree::BitSource for #type_ident {
            fn write(&self, sink: &mut (impl ::decree::BitSink + ?Sized), start: usize, len: usize, pos: usize) -> core::result::Result<usize, ::decree::Error> {
                #body
            }

            fn size(&self) -> usize {
                #size_body
            }
        }
    })
}

fn write_source_block(field_bindings: &TokenStreamInternal, pattern: &BitPattern, mapping: &HashMap<RawMappingValue, Expr>) -> syn::Result<TokenStreamInternal> {
    let writes = {
        let mut writes = Vec::new();
        let mut token_start = 0;
        for token in pattern.tokens.iter().rev() {
            match token {
                Token::Bits(Literal { bytes, bits }) => {
                    let num_bytes = bytes.len();
                    let byte_array_tokens = byte_array_tokens(bytes);
                    let token_end = token_start + bits;
                    let tokens = quote! {
                        if start < #token_end && end >= #token_start {
                            static source_bytes: [u8; #num_bytes] = #byte_array_tokens;
                            let source = ::decree::LittleEndian::<&[u8]>::with_bits(source_bytes.as_ref(), #bits)?;
                            println!("{:?}.write(sink, start={}, len={}, pos={})", source, start - #token_start, usize::min(#bits, len), pos);
                            let written = source.write(sink, start - #token_start, usize::min(#bits, len), pos)?;
                            println!("  written = {}", written);
                            start += written;
                            pos += written;
                            bits_written += written;
                            len -= written;
                        }
                    };
                    writes.push(tokens);
                    token_start += bits;
                },
                Token::Range(Range { name, start: range_start, len }) => {
                    let token_end = token_start + len;
                    let source_expr = {
                        let name_value = RawMappingValue::Name(name.to_string());
                        if mapping.contains_key(&name_value) {
                            &mapping[&name_value]
                        } else {
                            let index: usize = name.parse().unwrap();
                            &mapping[&RawMappingValue::Index(index)]
                        }
                    };
                    let tokens = quote! {
                        if start < #token_end && end >= #token_start {
                            let source = {
                                #field_bindings
                                #source_expr
                            };
                            println!("{:0b}.write(sink, start={}, len={}, pos={})", source, start - #token_start + #range_start, usize::min(#len, len), pos);
                            let written = source.write(
                                sink,
                                start - #token_start + #range_start,
                                usize::min(#len, len),
                                pos
                            )?;
                            println!("  written = {}", written);
                            start += written;
                            pos += written;
                            bits_written += written;
                            len -= written;
                        }
                    };
                    writes.push(tokens);
                    token_start += len;
                },
                Token::Error => unimplemented!(),
            }
        }
        writes
    };
    let tokens = quote! {
        let end = start + len;
        let mut bits_written = 0;
        let mut start = start;
        let mut len = len;
        let mut pos = pos;

        #(#writes)*
        
        Ok(bits_written)
    };
    Ok(tokens)
}

fn byte_array_tokens(bytes: &[u8]) -> TokenStreamInternal {
    let byte_tokens = {
        let mut tokens = vec![];
        for byte in bytes {
            tokens.push(quote! { #byte });
        }
        tokens
    };
    quote! {
        [#(#byte_tokens),*]
    }
}

fn compute_mapping(fields: &Fields, pattern: &BitPattern) -> syn::Result<HashMap<RawMappingValue, Expr>> {
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

fn find_bit_pattern(attrs: &[Attribute]) -> syn::Result<Option<BitPattern>> {
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
struct Range {
    name: String,
    start: usize,
    len: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Literal {
    bytes: Vec<u8>,
    bits: usize,
}

#[derive(Logos, Debug, PartialEq, Eq)]
enum Token {
    #[regex("[01]+", |lex| parse_literal(lex.slice()))]
    Bits(Literal),

    #[regex(r#"\[([a-zA-Z0-9_]+):\d+(-\d+)?\]"#, parse_range)]
    Range(Range),

    #[error]
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum RawMappingValue {
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
enum MappingValue {
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
struct BitPattern {
    tokens: Vec<Token>,
    renames: Vec<(Ident, MappingValue)>,
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

    pub fn bytes(&self) -> usize {
        let bits = self.bits();
        let mut bytes = bits / 8;
        if bits % 8 != 0 {
            bytes += 1;
        }
        bytes
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

fn string_lit(lit: &Expr, kind: &str) -> syn::Result<String> {
    match lit {
        Expr::Lit(ExprLit { lit: Lit::Str(string), ..}) => Ok(string.value()),
        _ => Err(syn::Error::new(lit.span(), format!("Expected a string literal expressing {}.", kind))),
    }
}

fn parse_bit_pattern(span: Span, pattern: &str) -> syn::Result<BitPattern> {
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

fn parse_range(lexer: &mut Lexer<Token>) -> Range {
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

fn parse_literal(string: &str) -> Literal {
    Literal {
        bytes: parse_bits(string),
        bits: string.len(),
    }
}

fn parse_bits(bits: &str) -> Vec<u8> {
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

fn parse_bits_to_byte(bits: &str) -> u8 {
    let mut result = 0;
    for bit in bits.chars() {
        result = result << 1;
        if bit == '1' {
            result |= 1;
        }
    }
    result
}
