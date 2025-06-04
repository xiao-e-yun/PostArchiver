macro_rules! as_column {
    ($row: expr, $col: expr) => {
        $row.get($col)?
    };
    (json, $row: expr, $col: expr) => {{
        let value: String = $row.get($col)?;
        serde_json::from_str(&value).unwrap()
    }};
}

macro_rules! as_table {
    ($($name:ident { $($field:ident: $col:expr $(=> $mode: ident)?),* $(,)? })+) => {
        $(impl $name {
            pub fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self {
                    $(
                        $field: crate::utils::macros::as_column!(
                            $($mode ,)?
                            row,
                            $col
                        ),
                    )*
                })
            }
        })+
    };
}

pub(crate) use {as_column, as_table};
