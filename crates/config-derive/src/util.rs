use proc_macro2::{Ident, TokenTree};
use quote::{format_ident, ToTokens};
use syn::{Data, Field, Fields, Type};

pub fn get_fields(data: &Data) -> Vec<&Field> {
	match data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(named) => {
				let mut fields = vec![];

				'outer: for field in &named.named {
					match field.ident {
						Some(_) => {
							if let Some(attr) = field.attrs.first() {
								for tree in attr.meta.to_token_stream() {
									if let TokenTree::Ident(ident) = tree {
										if ident == "serde" {
											continue 'outer;
										}
									}
								}
							}

							fields.push(field);
						}
						None => unimplemented!("Tuples are not supported"),
					}
				}

				fields
			}
			_ => unimplemented!("Only named fields are supported"),
		},
		_ => {
			unimplemented!("Only flat structs are supported")
		}
	}
}

pub fn get_type_ident(ty: &Type) -> Option<Ident> {
	for tree in ty.to_token_stream() {
		if let TokenTree::Ident(ident) = tree {
			return Some(ident);
		}
	}

	None
}

pub fn get_value_name(name: &Ident) -> Ident {
	format_ident!("{}Value", name.to_string())
}
