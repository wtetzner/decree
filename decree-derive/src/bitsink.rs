
use std::collections::HashMap;

use proc_macro2::{TokenStream as TokenStreamInternal, Ident};
use syn::{DeriveInput, spanned::Spanned, Expr, DataEnum};
use quote::quote;

use crate::bitpattern::{Range, Token, RawMappingValue, Literal, BitPattern, compute_mapping, find_bit_pattern};

const GENERIC_FAILURE: &str = r#"#[derive(BitSink)] expects an attribute of the form #[bitpattern("11010[a:0-2]0110", a=foo)]"#;

pub fn expand_bit_sink(ast: &DeriveInput) -> syn::Result<TokenStreamInternal> {
    Err(syn::Error::new(ast.span(), "#[derive(BitSink)] is not yet implemented."))
}


// fn write_source_block(field_bindings: &TokenStreamInternal, pattern: &BitPattern, mapping: &HashMap<RawMappingValue, Expr>) -> syn::Result<TokenStreamInternal> {
//     let writes = {
//         let mut writes = Vec::new();
//         let mut token_start = 0;
//         for token in pattern.tokens.iter().rev() {
//             match token {
//                 Token::Bits(Literal { bytes, bits }) => {
//                     let num_bytes = bytes.len();
//                     let byte_array_tokens = byte_array_tokens(bytes);
//                     let token_end = token_start + bits;
//                     let tokens = quote! {
//                         if start < #token_end && end >= #token_start {
//                             static source_bytes: [u8; #num_bytes] = #byte_array_tokens;
//                             let source = ::decree::LittleEndian::<&[u8]>::with_bits(source_bytes.as_ref(), #bits)?;
//                             let written = source.write(sink, start - #token_start, usize::min(#bits, len), pos)?;
//                             start += written;
//                             pos += written;
//                             bits_written += written;
//                             len -= written;
//                         }
//                     };
//                     writes.push(tokens);
//                     token_start += bits;
//                 },
//                 Token::Range(Range { name, start: range_start, len }) => {
//                     let token_end = token_start + len;
//                     let source_expr = {
//                         let name_value = RawMappingValue::Name(name.to_string());
//                         if mapping.contains_key(&name_value) {
//                             &mapping[&name_value]
//                         } else {
//                             let index: usize = name.parse().unwrap();
//                             &mapping[&RawMappingValue::Index(index)]
//                         }
//                     };
//                     let tokens = quote! {
//                         if start < #token_end && end >= #token_start {
//                             let source = {
//                                 #field_bindings
//                                 #source_expr
//                             };
//                             let written = source.write(
//                                 sink,
//                                 start - #token_start + #range_start,
//                                 usize::min(#len, len),
//                                 pos
//                             )?;
//                             start += written;
//                             pos += written;
//                             bits_written += written;
//                             len -= written;
//                         }
//                     };
//                     writes.push(tokens);
//                     token_start += len;
//                 },
//                 Token::Error => unimplemented!(),
//             }
//         }
//         writes
//     };
//     let tokens = quote! {
//         let end = start + len;
//         let mut bits_written = 0;
//         let mut start = start;
//         let mut len = len;
//         let mut pos = pos;

//         #(#writes)*
        
//         Ok(bits_written)
//     };
//     Ok(tokens)
// }
