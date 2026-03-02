use std::{collections::HashSet, ops::Deref, rc::Rc};

use chrono::{DateTime, Utc};
use rusqlite::ToSql;
use serde::Serialize;

use crate::query::RawSql;

#[derive(Debug)]
pub struct TextFilter {
    col: &'static str,
    text: String,
}

impl Deref for TextFilter {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl TextFilter {
    pub fn new(col: &'static str) -> Self {
        TextFilter {
            col,
            text: String::new(),
        }
    }

    pub fn contains(&mut self, text: &str) -> &mut Self {
        self.text = format!("%{}%", Self::safe_escape(text));
        self
    }

    pub fn starts_with(&mut self, text: &str) -> &mut Self {
        self.text = format!("{}%", Self::safe_escape(text));
        self
    }

    pub fn ends_with(&mut self, text: &str) -> &mut Self {
        self.text = format!("%{}", Self::safe_escape(text));
        self
    }

    pub fn equals(&mut self, text: &str) -> &mut Self {
        self.text = Self::safe_escape(text);
        self
    }

    pub fn like(&mut self, t: &str) -> &mut Self {
        self.text = t.to_string();
        self
    }

    fn safe_escape(text: &str) -> String {
        text.replace('%', "\\%").replace('_', "\\_")
    }

    pub fn build_sql<T>(&self, mut sql: RawSql<T>) -> RawSql<T> {
        if self.text.is_empty() {
            return sql;
        }

        let (wheres, params) = &mut sql.where_clause;
        wheres.push(format!("{} LIKE ?", self.col));
        params.push(Rc::new(self.text.clone()));
        sql
    }
}

#[derive(Debug)]
pub struct DateFilter {
    col: &'static str,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
}

impl DateFilter {
    pub fn new(col: &'static str) -> Self {
        DateFilter {
            col,
            before: None,
            after: None,
        }
    }

    pub fn before(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.before = Some(date);
        self
    }

    pub fn after(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.after = Some(date);
        self
    }

    pub fn equals(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.before = Some(date);
        self.after = Some(date);
        self
    }

    pub fn build_sql<T>(&self, mut sql: RawSql<T>) -> RawSql<T> {
        let (wheres, params) = &mut sql.where_clause;
        match (self.before, self.after) {
            (None, None) => {}
            (Some(before), None) => {
                wheres.push(format!("{} <= ?", self.col));
                params.push(Rc::new(before));
            }
            (None, Some(after)) => {
                wheres.push(format!("{} >= ?", self.col));
                params.push(Rc::new(after));
            }
            (Some(before), Some(after)) => {
                if before == after {
                    wheres.push(format!("{} = ?", self.col));
                    params.push(Rc::new(before));
                } else {
                    wheres.push(format!("{} BETWEEN ? AND ?", self.col));
                    params.push(Rc::new(after));
                    params.push(Rc::new(before));
                }
            }
        }
        sql
    }
}

#[derive(Debug)]
pub struct IdFilter<T> {
    col: &'static str,
    ids: HashSet<T>,
}

impl<T> Deref for IdFilter<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl<T: ToSql + Serialize + Clone + 'static> IdFilter<T>
where
    T: Eq + std::hash::Hash,
{
    pub fn new(col: &'static str) -> Self {
        IdFilter {
            col,
            ids: HashSet::new(),
        }
    }

    pub fn insert(&mut self, id: T) -> &mut Self {
        self.ids.insert(id);
        self
    }

    pub fn extend(&mut self, ids: impl IntoIterator<Item = T>) -> &mut Self {
        self.ids.extend(ids);
        self
    }

    pub fn build_sql<U>(&self, mut sql: RawSql<U>) -> RawSql<U> {
        let (wheres, params) = &mut sql.where_clause;
        match self.ids.len() {
            0 => {}
            1 => {
                wheres.push(format!("{} = ?", self.col));
                params.push(Rc::new(self.ids.iter().next().unwrap().clone()));
            }
            _ => {
                wheres.push(format!("{} IN (SELECT value FROM json_each(?))", self.col));
                let json_array = serde_json::to_string(&self.ids).unwrap();
                params.push(Rc::new(json_array));
            }
        }
        sql
    }
}

#[derive(Debug)]
pub struct RelationshipsFilter<T> {
    table: &'static str,
    col: &'static str,
    ids: HashSet<T>,
}

impl<T> Deref for RelationshipsFilter<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl<T> RelationshipsFilter<T> {
    pub fn new(table: &'static str, col: &'static str) -> Self {
        RelationshipsFilter {
            table,
            col,
            ids: HashSet::new(),
        }
    }
}

impl<T: ToSql + Serialize + Clone + 'static> RelationshipsFilter<T>
where
    T: Eq + std::hash::Hash,
{
    pub fn build_sql<U>(&self, mut sql: RawSql<U>) -> RawSql<U> {
        let (wheres, params) = &mut sql.where_clause;
        match self.len() {
            0 => {}
            1 => {
                wheres.push(format!(
                    "EXISTS (SELECT 1 FROM {} WHERE post = posts.id AND {} = ?)",
                    self.table, self.col
                ));
                params.push(Rc::new(self.iter().next().unwrap().clone()));
            }
            n => {
                wheres.push(format!(
                  "? == (SELECT COUNT(*) FROM {} WHERE post = posts.id AND {} IN (SELECT value FROM json_each(?)))",
                  self.table, self.col
                ));
                params.push(Rc::new(n));
                let json_array = serde_json::to_string(&self.ids).unwrap();
                params.push(Rc::new(json_array));
            }
        }
        sql
    }
}
