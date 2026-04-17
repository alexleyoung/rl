use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::{note::Note, resource::Resource};

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct ResourceDto {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub added_at: i64,
    pub last_read_at: Option<i64>,
    pub status: String,
    pub tags: Vec<String>,
}

impl ResourceDto {
    pub fn from_parts(r: Resource, tags: Vec<String>) -> Self {
        Self {
            id: r.id,
            kind: r.kind,
            title: r.title,
            author: r.author,
            url: r.url,
            file_path: r.file_path,
            added_at: r.added_at,
            last_read_at: r.last_read_at,
            status: r.status,
            tags,
        }
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct ResourceInputDto {
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct QuickSetDto {
    pub field: String,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct SetTagsDto {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct NoteDto {
    pub id: i64,
    pub resource_id: i64,
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub updated_at: i64,
}

impl From<Note> for NoteDto {
    fn from(n: Note) -> Self {
        Self {
            id: n.id,
            resource_id: n.resource_id,
            title: n.title,
            body_md: n.body_md,
            body_html: n.body_html,
            updated_at: n.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct NoteInputDto {
    pub title: String,
    pub body_md: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct ResourceDetailDto {
    pub resource: ResourceDto,
    pub notes: Vec<NoteDto>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct NoteLocationDto {
    pub resource_id: i64,
    pub note_id: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct TagDto {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryDto {
    pub q: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct SearchHitDto {
    pub source_kind: String,
    pub source_id: i64,
    pub resource_id: i64,
    pub note_id: Option<i64>,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct SearchResponseDto {
    pub query: String,
    pub hits: Vec<SearchHitDto>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct ApiErrorDto {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct ExtractInputDto {
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Default, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct MetadataDto {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct ReadingContentDto {
    pub content_html: String,
    pub source_type: String,
    pub word_count: i64,
    pub status: String,
}
