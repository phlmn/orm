extern crate postgres;

extern crate log;

use std::collections::HashMap;

use itertools::Itertools;
use postgres::{
    rows::Row,
    transaction::Transaction,
    types::{FromSql, ToSql},
    GenericConnection,
};

pub trait Entity: std::fmt::Debug + Sized {
    /// Partial variant of this entity, containing at least the fields that are
    /// necessary for insertion.
    type Partial: std::fmt::Debug + From<Self>;

    const TABLE_NAME: &'static str;
    const FIELD_INFOS: &'static [&'static FieldInfo];

    fn from_row(row: Row) -> Self;
    fn to_map(&self) -> HashMap<String, &dyn ToSql>;
}

#[derive(Debug, Clone, Copy)]
pub struct FieldInfo {
    pub name: &'static str,
    pub primary_key: bool,
    pub generated: bool,
}

#[derive(Debug)]
pub struct ToOne<Id, E>
where
    E: Entity,
{
    pub id: Id,
    pub entity: Option<E>,
}

impl<T: ToSql, E: Entity> ToSql for ToOne<T, E> {
    fn to_sql(
        &self,
        ty: &postgres::types::Type,
        out: &mut Vec<u8>,
    ) -> Result<postgres::types::IsNull, Box<dyn std::error::Error + 'static + Send + Sync>> {
        self.id.to_sql(ty, out)
    }

    fn accepts(ty: &postgres::types::Type) -> bool {
        T::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &postgres::types::Type,
        out: &mut Vec<u8>,
    ) -> Result<postgres::types::IsNull, Box<dyn std::error::Error + 'static + Send + Sync>> {
        self.id.to_sql_checked(ty, out)
    }
}

impl<T: FromSql, E: Entity> FromSql for ToOne<T, E> {
    fn from_sql(
        ty: &postgres::types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let id = T::from_sql(ty, raw)?;

        Ok(ToOne { id, entity: None })
    }

    fn accepts(ty: &postgres::types::Type) -> bool {
        T::accepts(ty)
    }

    fn from_sql_null(
        ty: &postgres::types::Type,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let id = T::from_sql_null(ty)?;

        Ok(ToOne { id, entity: None })
    }

    fn from_sql_nullable(
        ty: &postgres::types::Type,
        raw: Option<&[u8]>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let id = T::from_sql_nullable(ty, raw)?;

        Ok(ToOne { id, entity: None })
    }
}

pub struct Client<Conn>
where
    Conn: GenericConnection,
{
    conn: Conn,
}

pub trait TransactionalClient {
    fn commit(self) -> postgres::Result<()>;
}

impl<'a> TransactionalClient for Client<Transaction<'a>> {
    fn commit(self) -> postgres::Result<()> {
        self.conn.commit()
    }
}

impl<Conn> Client<Conn>
where
    Conn: GenericConnection,
{
    pub fn new(conn: Conn) -> Client<Conn> {
        Client { conn }
    }

    pub fn conn(&mut self) -> &Conn {
        &self.conn
    }

    pub fn transaction(&self) -> postgres::Result<Client<Transaction>> {
        let transaction = self.conn.transaction()?;

        Ok(Client { conn: transaction })
    }

    pub fn query<T: Entity>(
        &mut self,
        q: &str,
        params: &[&dyn ToSql]
    ) -> Vec<T> {
        let expanded_query = q.replace(
            "@select",
            format!("SELECT {}.* FROM {}", T::TABLE_NAME, T::TABLE_NAME).as_str(),
        );

        // if let Some(rel_info) = T::RELATION_INFO.get(relation) {
        //     let join_stmt = format!("JOIN {} ON {}.{} = {}.{}", rel_info.TABLE_NAME, T::TABLE_NAME, rel_info.FROM_COLUMN, rel_info.TO_TABLE, rel_info.TO_COLUMN);
        // }

        let res = self.conn.query(expanded_query.as_str(), params).unwrap();

        res.iter().map(T::from_row).collect()
    }

    fn build_insert<T: Entity>(&mut self) -> String {
        let fields_to_insert: Vec<_> = T::FIELD_INFOS
            .iter()
            .filter(|field| !field.generated)
            .collect();

        let rows = fields_to_insert
            .iter()
            .map(|field| quote_sql_ident(field.name))
            .join(", ");

        let placeholders = (1..=fields_to_insert.len())
            .map(|i| format!("${}", i))
            .join(", ");

        let query = format!(
            "INSERT INTO {} ({}) VALUES({})",
            T::TABLE_NAME,
            rows,
            placeholders
        );

        query
    }

    pub fn insert<T: Entity>(&mut self, entity: T) -> postgres::Result<u64> {
        let fields_to_insert = T::FIELD_INFOS.iter().filter(|field| !field.generated);

        let map = entity.to_map();

        let values: Vec<&dyn ToSql> = fields_to_insert
            .map(|field| *map.get(field.name).unwrap())
            .collect();

        let expanded_query = self.build_insert::<T>();

        let stmt = self.conn.prepare_cached(expanded_query.as_str())?;

        stmt.execute(values.as_slice())
    }
}

fn quote_sql_ident(i: &str) -> String {
    format!("\"{}\"", i)

    // TODO if supporting mysql (lol)
    // format!("`{}`", i)
}
