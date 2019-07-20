use std::convert::TryFrom;

use failure::{bail, err_msg, Error, Fallible};
use proc_macro2::Ident;

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: Ident,
    pub primary_key: bool,
    pub generated: bool,
}

impl TryFrom<syn::Field> for FieldInfo {
    type Error = Error;

    fn try_from(field: syn::Field) -> Fallible<Self> {
        let name = field
            .ident
            .ok_or(err_msg("expected a named field"))?
            .clone();
        let mut primary_key = false;
        let mut generated = false;

        for attr in &field.attrs {
            if let syn::Meta::List(syn::MetaList { ident, nested, .. }) = attr.parse_meta()? {
                if ident == crate::ATTRIBUTE_SCOPE {
                    for nested_meta in nested.iter() {
                        if let syn::NestedMeta::Meta(syn::Meta::Word(nested_ident)) = nested_meta {
                            let attr: &str = &nested_ident.to_string();
                            match attr {
                                "generated" => generated = true,
                                "primary_key" => primary_key = true,
                                _ => {
                                    bail!("unknown attribute: {}", attr)
                                    // TODO: emit a warning about unknown attribute on nightly?
                                    // #[rustversion::nightly]
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(FieldInfo {
            name,
            primary_key,
            generated,
        })
    }
}
