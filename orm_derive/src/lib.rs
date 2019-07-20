extern crate proc_macro;

use std::convert::{TryFrom, TryInto as _};

use failure::{bail, Error, Fallible};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod field_info;

use field_info::FieldInfo;

const ATTRIBUTE_SCOPE: &'static str = "orm";

#[proc_macro_derive(Entity, attributes(orm))]
pub fn derive_entity_fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let entity: Entity = input.try_into().unwrap();

    let entity_name = &entity.type_name;
    let partial_entity_name = Ident::new(&format!("Partial{}", entity.type_name), Span::call_site());

    let module_name = format!("_impl_entity_for_{}", entity.type_name).to_lowercase();
    let module_ident = Ident::new(&module_name, Span::call_site());

    let field_infos: Vec<TokenStream> = entity
        .field_infos
        .iter()
        .map(|info| {
            let info_name = &info.name.to_string();
            let info_primary_key = &info.primary_key;
            let info_generated = &info.generated;

            quote! {
                orm::FieldInfo {
                    name: #info_name,
                    primary_key: #info_primary_key,
                    generated: #info_generated,
                }
            }
        })
        .collect();

    let field_constructors: Vec<TokenStream> = entity
        .field_infos
        .iter()
        .map(|info| {
            let info_name = &info.name.to_string();
            let info_ident = &info.name;
            quote! { #info_ident: row.get(#info_name) }
        })
        .collect();

    let map_statements: Vec<TokenStream> = entity
        .field_infos
        .iter()
        .map(|info| {
            let info_name = &info.name.to_string();
            let info_ident = &info.name;
            quote! { map.insert(String::from(#info_name), &self.#info_ident); }
        })
        .collect();

    let table_name = entity.table_name;

    let expanded = quote! {
        fn #module_ident() {
            use std::collections::HashMap;
            use orm::{self, FieldInfo};
            use postgres::rows::Row;
            use postgres::types::ToSql;

            impl orm::Entity for #entity_name {
                type Partial = #partial_entity_name;

                const TABLE_NAME: &'static str = #table_name;
                const FIELD_INFOS: &'static [&'static FieldInfo] = &[
                    #(&#field_infos),*
                ];

                fn from_row(row: Row) -> Self {
                    #entity_name {
                        #(#field_constructors),*
                    }
                }

                fn to_map(&self) -> HashMap<String, &dyn ToSql> {
                    let mut map: HashMap<String, &dyn ToSql> = HashMap::new();
                    #(#map_statements)*
                    map
                }
            }

            #[derive(Debug)]
            pub struct #partial_entity_name {
                // TODO
            }

            impl From<#entity_name> for #partial_entity_name {
                fn from(entity: #entity_name) -> Self {
                    // TODO
                    unimplemented!()
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

struct Entity {
    type_name: Ident,
    table_name: String,
    field_infos: Vec<FieldInfo>,
}

impl TryFrom<DeriveInput> for Entity {
    type Error = Error;

    fn try_from(input: DeriveInput) -> Fallible<Self> {
        let fields = match input.data {
            syn::Data::Struct(s) => match s.fields {
                syn::Fields::Named(f) => f.named,
                _ => bail!("derive(Entity) can only be applied to structs with named fields"),
            },
            _ => bail!("derive(Entity) can only be applied to structs"),
        };

        let table_name = input.ident.to_string();

        Ok(Self {
            type_name: input.ident,
            // TODO: #[orm(table_name = "")] attribute
            table_name,
            field_infos: fields
                .into_pairs()
                .map(syn::punctuated::Pair::into_value)
                .map(FieldInfo::try_from)
                .collect::<Fallible<_>>()?,
        })
    }
}
