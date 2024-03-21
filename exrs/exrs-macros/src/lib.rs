#![allow(clippy::needless_doctest_main, clippy::tabs_in_doc_comments)]

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// An easy way to generate a macro error message. It takes a start span, end span, and error message,
/// and can be converted into a `TokenStream` with `Into<TokenStream>`. When a macro returns this as
/// a `TokenStream`, it shows the given error message at the given span.
struct Error<'a> {
	pub msg: &'a str,
	pub start: Span,
	pub end: Span,
}

/// Automatically adds a `VARIANTS` constant to an enum, which is an
/// array containing all of the enum's variants. For example:
///
/// ```rust
/// #[variants]
/// #[derive(Debug)]
/// pub enum MyEnum {
/// 	VariantOne,
/// 	VariantTwo,
/// 	VariantThree
/// }
///
/// fn main() {
/// 	println!("{:?}", MyEnum::VARIANTS); // [VariantOne, VariantTwo, VariantThree]
/// }
/// ```
#[proc_macro_attribute]
pub fn variants(_: TokenStream, input: TokenStream) -> TokenStream {
	let mut source = input.clone();
	let mut tokens = input.into_iter();

	let enum_token = loop {
		match tokens.next() {
			Some(token) => {
				if token.to_string() == "enum" {
					break token;
				}
			}
			None => {
				return Error {
					msg: "`variants` only works with enums",
					start: Span::call_site(),
					end: Span::call_site(),
				}
				.into()
			}
		}
	};

	let Some(TokenTree::Ident(enum_name_token)) = tokens.next() else {
		return Error {
			msg: "Expected an enum name",
			start: enum_token.span(),
			end: enum_token.span(),
		}
		.into();
	};
	let Some(TokenTree::Group(enum_declaration_token)) = tokens.next() else {
		return Error {
			msg: "Expected `{` after enum name",
			start: enum_token.span(),
			end: enum_name_token.span(),
		}
		.into();
	};

	let mut variants = Vec::new();
	let mut enum_declaration = enum_declaration_token.stream().into_iter();

	let first_variant = loop {
		// Skip past any attributes
		match enum_declaration.next() {
			Some(TokenTree::Ident(token)) => {
				break token;
			}
			Some(_) => {}
			None => {
				return Error {
					msg: "Expected an enum variant",
					start: enum_declaration_token.span_open(),
					end: enum_declaration_token.span_close(),
				}
				.into()
			}
		}
	};
	variants.push(first_variant);
	loop {
		match enum_declaration.next() {
			Some(TokenTree::Punct(token)) => {
				if token.as_char() == ',' {
					loop {
						match enum_declaration.next() {
							Some(TokenTree::Ident(variant)) => {
								variants.push(variant);
								break;
							}
							Some(_) => {}
							None => break,
						}
					}
				}
			}
			Some(_) => {}
			None => break,
		}
	}

	let num_variants = variants.len();
	let mut variants_formatted = String::new();
	for variant in variants {
		variants_formatted += &format!("Self::{variant},");
	}

	let macro_output: TokenStream = format!(
		"
		impl {enum_name_token} {{
			pub const VARIANTS: [Self; {num_variants}] = [{variants_formatted}];
		}}
		"
	)
	.parse()
	.unwrap();

	source.extend(macro_output);
	source
}

impl From<Error<'_>> for TokenStream {
	fn from(value: Error) -> Self {
		TokenStream::from_iter(vec![
			TokenTree::Punct({
				let mut punct = Punct::new(':', Spacing::Joint);
				punct.set_span(value.start);
				punct
			}),
			TokenTree::Punct({
				let mut punct = Punct::new(':', Spacing::Alone);
				punct.set_span(value.start);
				punct
			}),
			TokenTree::Ident(Ident::new("core", value.start)),
			TokenTree::Punct(Punct::new(':', Spacing::Joint)),
			TokenTree::Punct(Punct::new(':', Spacing::Alone)),
			TokenTree::Ident(Ident::new("compile_error", value.start)),
			TokenTree::Punct(Punct::new('!', Spacing::Alone)),
			TokenTree::Group({
				let mut group = Group::new(
					Delimiter::Brace,
					TokenStream::from_iter(vec![TokenTree::Literal(Literal::string(value.msg))]),
				);
				group.set_span(value.end);
				group
			}),
		])
	}
}
