use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod util;

#[proc_macro_derive(Val)]
pub fn derive_val(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let fields = util::get_fields(&input.data);
	let value = util::get_value_name(&input.ident);

	let (variants, fmts, impls) = {
		let mut defined = vec![];

		let mut variants = TokenStream::new();
		let mut fmts = TokenStream::new();
		let mut impls = TokenStream::new();

		for field in fields {
			let ty = &field.ty;

			let ident = util::get_type_ident(ty).unwrap();

			if defined.contains(&ident) {
				continue;
			} else {
				defined.push(ident.clone());
			}

			let variant = {
				let variant = ident.to_string();
				let variant = variant[0..1].to_uppercase() + &variant[1..];
				Ident::new(&variant, ident.span())
			};

			variants.extend(quote! {
				#variant(#ty),
			});

			fmts.extend(quote! {
				#value::#variant(v) => write!(f, "{}", v.to_string()),
			});

			impls.extend(quote!(
				impl From<#ty> for #value {
					fn from(value: #ty) -> Self {
						#value::#variant(value)
					}
				}
			))
		}

		(variants, fmts, impls)
	};

	let expanded = quote! {
		#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
		#[serde(untagged)]
		pub enum #value {
			#variants
		}

		impl std::fmt::Display for #value {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				match self {
					#fmts
				}
			}
		}

		#impls
	};

	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Iter)]
pub fn derive_iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = &input.ident;
	let data = input.data;

	let fields = util::get_fields(&data);
	let value = util::get_value_name(&input.ident);

	let arms = {
		let mut arms = TokenStream::new();

		for (index, field) in fields.iter().enumerate() {
			let ident = field.ident.as_ref().unwrap().to_string();

			arms.extend(quote! {
				#index => #ident,
			});
		}

		arms
	};

	let expanded = quote! {
		pub struct IntoIter<'a> {
			inner: &'a #name,
			index: usize,
		}

		impl<'a> Iterator for IntoIter<'a> {
			type Item = (&'a str, #value);

			fn next(&mut self) -> Option<Self::Item> {
				let index = match self.index {
					#arms
					_ => return None,
				};

				self.index += 1;

				Some((index, self.inner.get(index).unwrap()))
			}
		}

		impl<'a> IntoIterator for &'a #name {
			type Item = (&'a str, #value);
			type IntoIter = IntoIter<'a>;

			fn into_iter(self) -> Self::IntoIter {
				IntoIter {
					inner: self,
					index: 0,
				}
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Get)]
pub fn derive_get(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = &input.ident;
	let data = input.data;

	let fields = util::get_fields(&data);
	let value = util::get_value_name(&input.ident);

	let arms = {
		let mut arms = TokenStream::new();

		for field in fields {
			let ident = field.ident.as_ref().unwrap();
			let key = ident.to_string();

			arms.extend(quote! {
				#key => Some(self.#ident.clone().into()),
			});
		}

		arms
	};

	let expanded = quote! {
		impl #name {
			pub fn get(&self, key: &str) -> Option<#value> {
				match key {
					#arms
					_ => None,
				}
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Set)]
pub fn derive_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = &input.ident;
	let data = input.data;
	let fields = util::get_fields(&data);

	let arms = {
		let mut arms = TokenStream::new();

		for field in fields {
			let ident = field.ident.as_ref().unwrap();
			let key = ident.to_string();

			arms.extend(quote! {
				#key => self.#ident = value.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
			});
		}

		arms
	};

	let expanded = quote! {
		impl #name {
			pub fn set(&mut self, key: &str, value: &str) -> std::io::Result<()> {
				match key {
					#arms
					_ => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Field `{}` does not exist", key))),
				}

				Ok(())
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}
