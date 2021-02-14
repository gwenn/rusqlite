//#![feature(proc_macro_diagnostic)]

use proc_macro2::Span;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse, Error, LitStr, Result, Token};

use fallible_iterator::FallibleIterator;
use sqlite3_parser::lexer::sql::Parser;

struct Input(String);

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Input(
            Punctuated::<LitStr, Token![+]>::parse_separated_nonempty(input)?
                .iter()
                .map(LitStr::value)
                .collect(),
        ))
    }
}

#[proc_macro]
pub fn validate(input: TokenStream) -> TokenStream {
    match parse::<Input>(input.clone()) {
        Ok(Input(sql)) => {
            let mut parser = Parser::new(sql.as_bytes());
            loop {
                match parser.next() {
                    Ok(None) => break,
                    Err(err) => {
                        return Error::new(Span::call_site(), err).to_compile_error().into();
                    }
                    _ => {}
                }
            }
        }
        _ => {
            //Span::call_site().warning("cannot validate input").emit();
        }
    }
    input
}
