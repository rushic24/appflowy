use diesel::sql_types::Integer;

use flowy_database::schema::view_table;
use lib_infra::timestamp;

use crate::{
    entities::{
        trash::{Trash, TrashType},
        view::{RepeatedView, UpdateViewParams, View, ViewType},
    },
    sql_tables::app::AppTable,
};

#[derive(PartialEq, Clone, Debug, Queryable, Identifiable, Insertable, Associations)]
#[belongs_to(AppTable, foreign_key = "belong_to_id")]
#[table_name = "view_table"]
pub(crate) struct ViewTable {
    pub id: String,
    pub belong_to_id: String,
    pub name: String,
    pub desc: String,
    pub modified_time: i64,
    pub create_time: i64,
    pub thumbnail: String,
    pub view_type: ViewTableType,
    pub version: i64,
    pub is_trash: bool,
}

impl ViewTable {
    pub fn new(view: View) -> Self {
        let view_type = match view.view_type {
            ViewType::Blank => ViewTableType::Docs,
            ViewType::Doc => ViewTableType::Docs,
        };

        ViewTable {
            id: view.id,
            belong_to_id: view.belong_to_id,
            name: view.name,
            desc: view.desc,
            modified_time: view.modified_time,
            create_time: view.create_time,
            // TODO: thumbnail
            thumbnail: "".to_owned(),
            view_type,
            version: 0,
            is_trash: false,
        }
    }
}

impl std::convert::From<ViewTable> for View {
    fn from(table: ViewTable) -> Self {
        let view_type = match table.view_type {
            ViewTableType::Docs => ViewType::Doc,
        };

        View {
            id: table.id,
            belong_to_id: table.belong_to_id,
            name: table.name,
            desc: table.desc,
            view_type,
            belongings: RepeatedView::default(),
            modified_time: table.modified_time,
            version: table.version,
            create_time: table.create_time,
        }
    }
}

impl std::convert::From<ViewTable> for Trash {
    fn from(table: ViewTable) -> Self {
        Trash {
            id: table.id,
            name: table.name,
            modified_time: table.modified_time,
            create_time: table.create_time,
            ty: TrashType::View,
        }
    }
}

#[derive(AsChangeset, Identifiable, Clone, Default, Debug)]
#[table_name = "view_table"]
pub(crate) struct ViewTableChangeset {
    pub id: String,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub thumbnail: Option<String>,
    pub modified_time: i64,
}

impl ViewTableChangeset {
    pub(crate) fn new(params: UpdateViewParams) -> Self {
        ViewTableChangeset {
            id: params.view_id,
            name: params.name,
            desc: params.desc,
            thumbnail: params.thumbnail,
            modified_time: timestamp(),
        }
    }

    pub(crate) fn from_table(table: ViewTable) -> Self {
        ViewTableChangeset {
            id: table.id,
            name: Some(table.name),
            desc: Some(table.desc),
            thumbnail: Some(table.thumbnail),
            modified_time: table.modified_time,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, FromSqlRow, AsExpression)]
#[repr(i32)]
#[sql_type = "Integer"]
pub enum ViewTableType {
    Docs = 0,
}

impl std::default::Default for ViewTableType {
    fn default() -> Self { ViewTableType::Docs }
}

impl std::convert::From<i32> for ViewTableType {
    fn from(value: i32) -> Self {
        match value {
            0 => ViewTableType::Docs,
            o => {
                log::error!("Unsupported view type {}, fallback to ViewType::Docs", o);
                ViewTableType::Docs
            },
        }
    }
}

impl ViewTableType {
    pub fn value(&self) -> i32 { *self as i32 }
}

impl_sql_integer_expression!(ViewTableType);
