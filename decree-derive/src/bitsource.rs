
use std::collections::HashMap;

use proc_macro2::{TokenStream as TokenStreamInternal, Ident};
use syn::{DeriveInput, spanned::Spanned, Expr, DataEnum};
use quote::quote;

use crate::{bitpattern::{Range, Token, RawMappingValue, Literal, BitPattern, compute_mapping, find_bit_pattern}, common::{collect_types, type_constraints, generics, generics_names, where_clause}};

const GENERIC_FAILURE: &str = r#"#[derive(BitSource)] expects an attribute of the form #[bitpattern("11010[a:0-2]0110", a=foo)]"#;

pub fn expand_bit_source(ast: &DeriveInput) -> syn::Result<TokenStreamInternal> {
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
                            bindings.push(quote!{ let #new_ident = &source.#ident;});
                        } else {
                            let ident = Ident::new_raw(&format!("_____{}", index), field.span());
                            let index = syn::Index::from(index);
                            bindings.push(quote! { let #ident = &source.#index; });
                        };
                        index += 1;
                    }
                    quote! {
                        #(#bindings)*
                    }
                };
                let block = write_source_block(&field_bindings, &pattern, &mapping)?;
                let body = quote! {
                    ::decree::check_input_range(#type_name, start, len, 0, ::decree::BitSource::size(source))?;
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
                                ::decree::check_input_range(#type_name, start, len, 0, ::decree::BitSource::size(source))?;
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
                match source {
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
    let generics = generics(ast);
    let generics_names = generics_names(ast);
    let where_clause = where_clause(ast, &quote! { ::decree::BitSource })?;
    Ok(quote! {
        impl #generics ::decree::BitSource for #type_ident #generics_names #where_clause {
            fn write(&self, sink: &mut (impl ::decree::BitSink + ?Sized), start: usize, len: usize, pos: usize) -> core::result::Result<usize, ::decree::Error> {
                if len == 0 {
                    return Ok(0);
                }
                #[inline]
                fn write_inner #generics (source: &#type_ident #generics_names, sink: &mut (impl ::decree::BitSink + ?Sized), start: usize, len: usize, pos: usize) -> core::result::Result<usize, ::decree::Error> #where_clause {
                    #body
                }
                write_inner(self, sink, start, len, pos)
                    .map_err(|err| err.write_failed(
                        format!(
                            "Failed to write [{}, {}] from {} to {} in the output (len={}).",
                            start,
                            start + len - 1,
                            #type_name,
                            pos,
                            sink.size().map(|s| s.to_string()).unwrap_or("-".to_string())
                        )
                    ))
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
                            let written = source.write(sink, start - #token_start, usize::min(#bits, len), pos)?;
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
                            let written = source.write(
                                sink,
                                start - #token_start + #range_start,
                                usize::min(#len, len),
                                pos
                            )?;
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
