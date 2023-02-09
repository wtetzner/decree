use proc_macro::TokenStream as TokenStreamExternal;
use syn::DeriveInput;
use syn::parse_macro_input;

mod bitpattern;
mod bitsource;
mod bitsink;
mod common;

#[proc_macro_derive(BitSource, attributes(bitpattern))]
pub fn bit_source(input: TokenStreamExternal) -> TokenStreamExternal {
    let ast: DeriveInput = parse_macro_input!(input);
    bitsource::expand_bit_source(&ast)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(BitSink, attributes(bitpattern))]
pub fn bit_sink(input: TokenStreamExternal) -> TokenStreamExternal {
    let ast: DeriveInput = parse_macro_input!(input);
    bitsink::expand_bit_sink(&ast)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}


