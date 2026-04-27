use rusqlite::{params, types::Value, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::{BibEntry, parse_bib};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: i64,
    pub key: String,
    pub entry_type: String,
    pub title: Option<String>,
    pub authors: Option<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub doi: Option<String>,
    pub abstract_text: Option<String>,
    pub url: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub booktitle: Option<String>,
    pub edition: Option<String>,
    pub month: Option<String>,
    pub keywords: Option<String>,
    pub note: Option<String>,
    pub isbn: Option<String>,
    pub issn: Option<String>,
    pub number: Option<String>,
    pub institution: Option<String>,
    pub added_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemInput {
    pub key: String,
    pub entry_type: String,
    pub title: Option<String>,
    pub authors: Option<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub doi: Option<String>,
    pub abstract_text: Option<String>,
    pub url: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub booktitle: Option<String>,
    pub edition: Option<String>,
    pub month: Option<String>,
    pub keywords: Option<String>,
    pub note: Option<String>,
    pub isbn: Option<String>,
    pub issn: Option<String>,
    pub number: Option<String>,
    pub institution: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub text: Option<String>,
    pub entry_types: Option<Vec<String>>,
    pub year_min: Option<String>,
    pub year_max: Option<String>,
    pub has_doi: Option<bool>,
    pub collection_id: Option<i64>,
    pub tag_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub created_at: i64,
    pub item_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

// ── Attachment / Annotation types ────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id:        i64,
    pub item_id:   i64,
    pub file_name: String,
    pub file_path: String,
    pub added_at:  i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub id:            i64,
    pub item_id:       i64,
    pub attachment_id: i64,
    pub page:          i64,
    pub ann_type:      String,
    pub color:         String,
    pub selected_text: Option<String>,
    pub note_text:     Option<String>,
    pub position_json: String,
    pub created_at:    i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationWithSource {
    pub id:                  i64,
    pub item_id:             i64,
    pub attachment_id:       i64,
    pub page:                i64,
    pub ann_type:            String,
    pub color:               String,
    pub selected_text:       Option<String>,
    pub note_text:           Option<String>,
    pub created_at:          i64,
    pub item_title:          Option<String>,
    pub item_authors:        Option<String>,
    pub item_year:           Option<String>,
    pub item_key:            String,
    pub attachment_file_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationInput {
    pub item_id:       i64,
    pub attachment_id: i64,
    pub page:          i64,
    pub ann_type:      String,
    pub color:         String,
    pub selected_text: Option<String>,
    pub note_text:     Option<String>,
    pub position_json: String,
}

// ── Path helpers ──────────────────────────────────────────────────────────────

fn library_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".quire")
        .join("library.db")
}

fn bib_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".quire")
        .join("references.bib")
}

fn attachments_dir(item_id: i64) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".quire")
        .join("attachments")
        .join(item_id.to_string())
}

// ── DB helpers ────────────────────────────────────────────────────────────────

fn open_conn() -> Result<Connection, String> {
    let conn = Connection::open(library_db_path()).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS items (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            key           TEXT    NOT NULL UNIQUE,
            entry_type    TEXT    NOT NULL DEFAULT 'misc',
            title         TEXT,
            authors       TEXT,
            year          TEXT,
            journal       TEXT,
            doi           TEXT,
            abstract_text TEXT,
            url           TEXT,
            volume        TEXT,
            issue         TEXT,
            pages         TEXT,
            publisher     TEXT,
            booktitle     TEXT,
            edition       TEXT,
            month         TEXT,
            keywords      TEXT,
            note          TEXT,
            isbn          TEXT,
            issn          TEXT,
            number        TEXT,
            institution   TEXT,
            added_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS collections (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL,
            parent_id  INTEGER REFERENCES collections(id) ON DELETE SET NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS collection_items (
            collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            item_id       INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            PRIMARY KEY (collection_id, item_id)
        );

        CREATE TABLE IF NOT EXISTS tags (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT    NOT NULL UNIQUE,
            color TEXT    NOT NULL DEFAULT '#888888'
        );

        CREATE TABLE IF NOT EXISTS item_tags (
            item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
            PRIMARY KEY (item_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS trash (
            item_id    INTEGER NOT NULL PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            deleted_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS attachments (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id   INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            file_name TEXT    NOT NULL,
            file_path TEXT    NOT NULL,
            added_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS annotations (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id        INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            attachment_id  INTEGER NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
            page           INTEGER NOT NULL,
            ann_type       TEXT    NOT NULL DEFAULT 'highlight',
            color          TEXT    NOT NULL DEFAULT '#FACC15',
            selected_text  TEXT,
            note_text      TEXT,
            position_json  TEXT    NOT NULL DEFAULT '{}',
            created_at     INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );",
    )
    .map_err(|e| e.to_string())
}

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<LibraryItem> {
    let tag_csv: String = row.get(25)?;
    let tags: Vec<String> = tag_csv
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    Ok(LibraryItem {
        id:            row.get(0)?,
        key:           row.get(1)?,
        entry_type:    row.get(2)?,
        title:         row.get(3)?,
        authors:       row.get(4)?,
        year:          row.get(5)?,
        journal:       row.get(6)?,
        doi:           row.get(7)?,
        abstract_text: row.get(8)?,
        url:           row.get(9)?,
        volume:        row.get(10)?,
        issue:         row.get(11)?,
        pages:         row.get(12)?,
        publisher:     row.get(13)?,
        booktitle:     row.get(14)?,
        edition:       row.get(15)?,
        month:         row.get(16)?,
        keywords:      row.get(17)?,
        note:          row.get(18)?,
        isbn:          row.get(19)?,
        issn:          row.get(20)?,
        number:        row.get(21)?,
        institution:   row.get(22)?,
        added_at:      row.get(23)?,
        updated_at:    row.get(24)?,
        tags,
    })
}

const ITEM_SELECT: &str =
    "SELECT i.id, i.key, i.entry_type, i.title, i.authors, i.year, i.journal, i.doi,
            i.abstract_text, i.url, i.volume, i.issue, i.pages, i.publisher, i.booktitle,
            i.edition, i.month, i.keywords, i.note, i.isbn, i.issn, i.number, i.institution,
            i.added_at, i.updated_at,
            COALESCE(GROUP_CONCAT(DISTINCT t.name), '') as tag_names
     FROM items i
     LEFT JOIN item_tags it ON it.item_id = i.id
     LEFT JOIN tags t ON t.id = it.tag_id";

fn fetch_item(conn: &Connection, id: i64) -> Result<Option<LibraryItem>, String> {
    let sql = format!("{} WHERE i.id=?1 GROUP BY i.id", ITEM_SELECT);
    conn.query_row(&sql, [id], |row| row_to_item(row))
        .optional()
        .map_err(|e| e.to_string())
}

// ── Migration helpers ─────────────────────────────────────────────────────────

fn migrate_entries(conn: &Connection, entries: &[BibEntry]) -> Result<(), String> {
    for e in entries {
        conn.execute(
            "INSERT OR IGNORE INTO items
             (key, entry_type, title, authors, year, journal, doi, abstract_text, url,
              volume, issue, pages, publisher, booktitle, edition, month, keywords,
              note, isbn, issn, number, institution)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            params![
                e.key, e.entry_type, e.title, e.authors, e.year, e.journal,
                e.doi, e.abstract_text, e.url, e.volume, e.issue, e.pages,
                e.publisher, e.booktitle, e.edition, e.month, e.keywords,
                e.note, e.isbn, e.issn, e.number, e.institution
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── BibTeX serialiser (for backward-compat .bib regeneration) ─────────────────

fn items_to_bib(items: &[LibraryItem]) -> String {
    let mut out = String::from(
        "% Quire global bibliography — ~/.quire/references.bib\n\
         % Managed by Quire. Edit via the Library view.\n\n",
    );
    for item in items {
        out.push_str(&format!("@{}{{{}", item.entry_type, item.key));

        macro_rules! field {
            ($name:literal, $opt:expr) => {
                if let Some(ref v) = $opt {
                    out.push_str(&format!(",\n  {:13} = {{{}}}", $name, v));
                }
            };
        }

        field!("title",       item.title);
        field!("author",      item.authors);
        field!("year",        item.year);
        field!("journal",     item.journal);
        field!("booktitle",   item.booktitle);
        field!("volume",      item.volume);
        field!("number",      item.number);
        field!("issue",       item.issue);
        field!("pages",       item.pages);
        field!("publisher",   item.publisher);
        field!("institution", item.institution);
        field!("edition",     item.edition);
        field!("month",       item.month);
        field!("isbn",        item.isbn);
        field!("issn",        item.issn);
        field!("doi",         item.doi);
        field!("url",         item.url);
        field!("keywords",    item.keywords);
        field!("abstract",    item.abstract_text);
        field!("note",        item.note);

        out.push_str("\n}\n\n");
    }
    out
}

fn regenerate_bib(conn: &Connection) -> Result<(), String> {
    let sql = format!(
        "{} WHERE i.id NOT IN (SELECT item_id FROM trash) GROUP BY i.id ORDER BY i.added_at ASC",
        ITEM_SELECT
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items: Vec<LibraryItem> = stmt
        .query_map([], |row| row_to_item(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    fs::write(bib_path(), items_to_bib(&items)).map_err(|e| e.to_string())
}

// ── Public init (called from lib.rs) ─────────────────────────────────────────

pub fn init_library(bib_entries: &[BibEntry]) {
    let Ok(conn) = open_conn() else { return };
    if init_schema(&conn).is_err() {
        return;
    }
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))
        .unwrap_or(0);
    if count == 0 && !bib_entries.is_empty() {
        let _ = migrate_entries(&conn, bib_entries);
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_library_items() -> Result<Vec<LibraryItem>, String> {
    let conn = open_conn()?;
    let sql = format!(
        "{} WHERE i.id NOT IN (SELECT item_id FROM trash) GROUP BY i.id ORDER BY i.added_at DESC",
        ITEM_SELECT
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| row_to_item(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}

#[tauri::command]
pub fn get_library_item(id: i64) -> Result<Option<LibraryItem>, String> {
    let conn = open_conn()?;
    fetch_item(&conn, id)
}

#[tauri::command]
pub fn create_library_item(item: ItemInput) -> Result<LibraryItem, String> {
    let conn = open_conn()?;
    conn.execute(
        "INSERT INTO items
         (key, entry_type, title, authors, year, journal, doi, abstract_text, url,
          volume, issue, pages, publisher, booktitle, edition, month, keywords,
          note, isbn, issn, number, institution)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
        params![
            item.key, item.entry_type, item.title, item.authors, item.year,
            item.journal, item.doi, item.abstract_text, item.url, item.volume,
            item.issue, item.pages, item.publisher, item.booktitle, item.edition,
            item.month, item.keywords, item.note, item.isbn, item.issn,
            item.number, item.institution
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let created = fetch_item(&conn, id)?.ok_or("Item not found after insert")?;
    regenerate_bib(&conn)?;
    Ok(created)
}

#[tauri::command]
pub fn update_library_item(id: i64, item: ItemInput) -> Result<LibraryItem, String> {
    let conn = open_conn()?;
    let rows = conn
        .execute(
            "UPDATE items SET
             entry_type=?1, title=?2, authors=?3, year=?4, journal=?5, doi=?6,
             abstract_text=?7, url=?8, volume=?9, issue=?10, pages=?11,
             publisher=?12, booktitle=?13, edition=?14, month=?15, keywords=?16,
             note=?17, isbn=?18, issn=?19, number=?20, institution=?21,
             updated_at=strftime('%s','now')
             WHERE id=?22",
            params![
                item.entry_type, item.title, item.authors, item.year, item.journal,
                item.doi, item.abstract_text, item.url, item.volume, item.issue,
                item.pages, item.publisher, item.booktitle, item.edition, item.month,
                item.keywords, item.note, item.isbn, item.issn, item.number,
                item.institution, id
            ],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("Item {} not found", id));
    }
    let updated = fetch_item(&conn, id)?.ok_or("Item not found after update")?;
    regenerate_bib(&conn)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_library_item(id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute("DELETE FROM items WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    regenerate_bib(&conn)?;
    Ok(())
}

#[tauri::command]
pub fn search_library_items(query: SearchQuery) -> Result<Vec<LibraryItem>, String> {
    let conn = open_conn()?;

    let mut conditions = vec!["i.id NOT IN (SELECT item_id FROM trash)".to_string()];
    let mut binds: Vec<Value> = vec![];

    if let Some(ref text) = query.text {
        if !text.is_empty() {
            let n = binds.len() + 1;
            conditions.push(format!(
                "(i.title LIKE ?{n} OR i.authors LIKE ?{n} OR i.key LIKE ?{n} OR i.abstract_text LIKE ?{n})"
            ));
            binds.push(Value::Text(format!("%{}%", text)));
        }
    }

    if let Some(ref min) = query.year_min {
        let n = binds.len() + 1;
        conditions.push(format!("i.year >= ?{n}"));
        binds.push(Value::Text(min.clone()));
    }

    if let Some(ref max) = query.year_max {
        let n = binds.len() + 1;
        conditions.push(format!("i.year <= ?{n}"));
        binds.push(Value::Text(max.clone()));
    }

    if let Some(true) = query.has_doi {
        conditions.push("i.doi IS NOT NULL AND i.doi != ''".to_string());
    }

    if let Some(ref types) = query.entry_types {
        if !types.is_empty() {
            let start = binds.len() + 1;
            let placeholders: Vec<String> =
                (start..start + types.len()).map(|i| format!("?{i}")).collect();
            conditions.push(format!("i.entry_type IN ({})", placeholders.join(",")));
            for t in types {
                binds.push(Value::Text(t.clone()));
            }
        }
    }

    if let Some(coll_id) = query.collection_id {
        let n = binds.len() + 1;
        conditions.push(format!(
            "i.id IN (SELECT item_id FROM collection_items WHERE collection_id = ?{n})"
        ));
        binds.push(Value::Integer(coll_id));
    }

    if let Some(ref tag) = query.tag_name {
        if !tag.is_empty() {
            let n = binds.len() + 1;
            conditions.push(format!(
                "i.id IN (SELECT it2.item_id FROM item_tags it2 JOIN tags t2 ON t2.id=it2.tag_id WHERE t2.name=?{n})"
            ));
            binds.push(Value::Text(tag.clone()));
        }
    }

    let sql = format!(
        "SELECT i.id, i.key, i.entry_type, i.title, i.authors, i.year, i.journal, i.doi,
                i.abstract_text, i.url, i.volume, i.issue, i.pages, i.publisher, i.booktitle,
                i.edition, i.month, i.keywords, i.note, i.isbn, i.issn, i.number, i.institution,
                i.added_at, i.updated_at,
                COALESCE(GROUP_CONCAT(DISTINCT t.name), '') as tag_names
         FROM items i
         LEFT JOIN item_tags it ON it.item_id = i.id
         LEFT JOIN tags t ON t.id = it.tag_id
         WHERE {}
         GROUP BY i.id ORDER BY i.added_at DESC",
        conditions.join(" AND ")
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let refs: Vec<&dyn rusqlite::types::ToSql> =
        binds.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
    let items = stmt
        .query_map(refs.as_slice(), |row| row_to_item(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}

// ── Collection commands ───────────────────────────────────────────────────────

fn row_to_collection(row: &rusqlite::Row) -> rusqlite::Result<Collection> {
    Ok(Collection {
        id:         row.get(0)?,
        name:       row.get(1)?,
        parent_id:  row.get(2)?,
        created_at: row.get(3)?,
        item_count: row.get(4)?,
    })
}

#[tauri::command]
pub fn get_collections() -> Result<Vec<Collection>, String> {
    let conn = open_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.parent_id, c.created_at,
                    COUNT(DISTINCT ci.item_id) as item_count
             FROM collections c
             LEFT JOIN collection_items ci ON ci.collection_id = c.id
             LEFT JOIN items i ON i.id = ci.item_id
                 AND i.id NOT IN (SELECT item_id FROM trash)
             GROUP BY c.id ORDER BY c.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let cols = stmt
        .query_map([], |row| row_to_collection(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(cols)
}

#[tauri::command]
pub fn create_collection(name: String, parent_id: Option<i64>) -> Result<Collection, String> {
    let conn = open_conn()?;
    conn.execute(
        "INSERT INTO collections (name, parent_id) VALUES (?1, ?2)",
        params![name, parent_id],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, name, parent_id, created_at, 0 FROM collections WHERE id=?1",
        [id],
        |row| row_to_collection(row),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_collection(id: i64, name: String) -> Result<Collection, String> {
    let conn = open_conn()?;
    conn.execute("UPDATE collections SET name=?1 WHERE id=?2", params![name, id])
        .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT c.id, c.name, c.parent_id, c.created_at,
                COUNT(DISTINCT ci.item_id) as item_count
         FROM collections c
         LEFT JOIN collection_items ci ON ci.collection_id = c.id
         WHERE c.id=?1 GROUP BY c.id",
        [id],
        |row| row_to_collection(row),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_collection(id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute("DELETE FROM collections WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_item_to_collection(collection_id: i64, item_id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute(
        "INSERT OR IGNORE INTO collection_items (collection_id, item_id) VALUES (?1, ?2)",
        params![collection_id, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_item_from_collection(collection_id: i64, item_id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute(
        "DELETE FROM collection_items WHERE collection_id=?1 AND item_id=?2",
        params![collection_id, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_item_collection_ids(item_id: i64) -> Result<Vec<i64>, String> {
    let conn = open_conn()?;
    let mut stmt = conn
        .prepare("SELECT collection_id FROM collection_items WHERE item_id=?1")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([item_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<i64>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

// ── Tag commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_tags() -> Result<Vec<Tag>, String> {
    let conn = open_conn()?;
    let mut stmt = conn
        .prepare("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE")
        .map_err(|e| e.to_string())?;
    let tags = stmt
        .query_map([], |row| {
            Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)? })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
pub fn create_tag(name: String, color: String) -> Result<Tag, String> {
    let conn = open_conn()?;
    conn.execute(
        "INSERT OR IGNORE INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )
    .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name, color FROM tags WHERE name=?1",
        [&name],
        |row| Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)? }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag_color(id: i64, color: String) -> Result<Tag, String> {
    let conn = open_conn()?;
    conn.execute("UPDATE tags SET color=?1 WHERE id=?2", params![color, id])
        .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name, color FROM tags WHERE id=?1",
        [id],
        |row| Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)? }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute("DELETE FROM tags WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_item_tags(item_id: i64, tag_ids: Vec<i64>) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute("DELETE FROM item_tags WHERE item_id=?1", [item_id])
        .map_err(|e| e.to_string())?;
    for tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)",
            params![item_id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Import / Export types ─────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub added:   usize,
    pub skipped: usize,
    pub errors:  Vec<String>,
}

// ── Metadata fetch types ──────────────────────────────────────────────────────

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FetchedMetadata {
    pub title:         Option<String>,
    pub authors:       Option<String>,
    pub year:          Option<String>,
    pub journal:       Option<String>,
    pub volume:        Option<String>,
    pub issue:         Option<String>,
    pub pages:         Option<String>,
    pub doi:           Option<String>,
    pub issn:          Option<String>,
    pub publisher:     Option<String>,
    pub url:           Option<String>,
    pub abstract_text: Option<String>,
    pub booktitle:     Option<String>,
    pub isbn:          Option<String>,
    pub entry_type:    Option<String>,
}

// ── Import / Export commands ──────────────────────────────────────────────────

#[tauri::command]
pub fn import_bib_file(path: String, mode: String) -> Result<ImportResult, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let entries = parse_bib(&content);
    let conn = open_conn()?;

    let replace = mode == "replace";
    let mut added   = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = vec![];

    let sql_merge   = "INSERT OR IGNORE INTO items
         (key,entry_type,title,authors,year,journal,doi,abstract_text,url,
          volume,issue,pages,publisher,booktitle,edition,month,keywords,
          note,isbn,issn,number,institution)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)";
    let sql_replace = "INSERT OR REPLACE INTO items
         (key,entry_type,title,authors,year,journal,doi,abstract_text,url,
          volume,issue,pages,publisher,booktitle,edition,month,keywords,
          note,isbn,issn,number,institution)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)";
    let sql = if replace { sql_replace } else { sql_merge };

    for entry in &entries {
        match conn.execute(sql, params![
            entry.key, entry.entry_type, entry.title, entry.authors, entry.year,
            entry.journal, entry.doi, entry.abstract_text, entry.url, entry.volume,
            entry.issue, entry.pages, entry.publisher, entry.booktitle, entry.edition,
            entry.month, entry.keywords, entry.note, entry.isbn, entry.issn,
            entry.number, entry.institution
        ]) {
            Ok(rows) => { if rows > 0 { added += 1; } else { skipped += 1; } }
            Err(e)   => { errors.push(format!("{}: {}", entry.key, e)); }
        }
    }

    let _ = regenerate_bib(&conn);
    Ok(ImportResult { added, skipped, errors })
}

#[tauri::command]
pub fn export_bib_file(item_ids: Vec<i64>, path: String) -> Result<usize, String> {
    let conn = open_conn()?;
    let items: Vec<LibraryItem> = if item_ids.is_empty() {
        let sql = format!(
            "{} WHERE i.id NOT IN (SELECT item_id FROM trash) GROUP BY i.id ORDER BY i.added_at ASC",
            ITEM_SELECT
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| row_to_item(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    } else {
        let placeholders: Vec<String> =
            (1..=item_ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "{} WHERE i.id IN ({}) GROUP BY i.id ORDER BY i.added_at ASC",
            ITEM_SELECT,
            placeholders.join(",")
        );
        let binds: Vec<Value> = item_ids.iter().map(|&id| Value::Integer(id)).collect();
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            binds.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(refs.as_slice(), |row| row_to_item(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    let count = items.len();
    fs::write(&path, items_to_bib(&items)).map_err(|e| e.to_string())?;
    Ok(count)
}

// ── Metadata fetch helpers ────────────────────────────────────────────────────

fn strip_xml_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn xml_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open  = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let after = &xml[start..];
    let cs = after.find('>')? + 1;
    let ce = after.find(&close)?;
    if cs >= ce { return None; }
    let text = strip_xml_tags(after[cs..ce].trim());
    if text.is_empty() { None } else { Some(text) }
}

// ── DOI fetch — CrossRef + DataCite fallback ─────────────────────────────────

fn normalize_doi(raw: &str) -> &str {
    let s = raw.trim();
    for prefix in &[
        "https://doi.org/", "http://doi.org/",
        "https://dx.doi.org/", "http://dx.doi.org/",
        "doi:", "DOI:",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest.trim();
        }
    }
    s
}

fn parse_crossref_message(msg: &serde_json::Value, fallback_doi: &str) -> FetchedMetadata {
    let title = msg["title"].as_array()
        .and_then(|a| a.first()).and_then(|v| v.as_str()).map(String::from);

    let authors = msg["author"].as_array().map(|arr| {
        arr.iter().filter_map(|a| {
            let fam = a["family"].as_str()?;
            let giv = a["given"].as_str().unwrap_or("");
            Some(if giv.is_empty() { fam.to_string() } else { format!("{fam}, {giv}") })
        }).collect::<Vec<_>>().join(" and ")
    }).filter(|s| !s.is_empty());

    let year = msg["published"]["date-parts"]
        .as_array().and_then(|a| a.first())
        .and_then(|dp| dp.as_array()).and_then(|dp| dp.first())
        .and_then(|y| y.as_u64()).map(|y| y.to_string());

    let journal = msg["container-title"].as_array()
        .and_then(|a| a.first()).and_then(|v| v.as_str())
        .filter(|s| !s.is_empty()).map(String::from);

    let volume    = msg["volume"].as_str().map(String::from);
    let issue     = msg["issue"].as_str().map(String::from);
    let pages     = msg["page"].as_str().map(String::from);
    let doi_val   = msg["DOI"].as_str().map(String::from)
        .unwrap_or_else(|| fallback_doi.to_string());
    let issn      = msg["ISSN"].as_array()
        .and_then(|a| a.first()).and_then(|v| v.as_str()).map(String::from);
    let publisher = msg["publisher"].as_str().map(String::from);
    let url_val   = msg["URL"].as_str().map(String::from);
    let abstract_text = msg["abstract"].as_str()
        .map(|s| strip_xml_tags(s)).filter(|s| !s.is_empty());
    let entry_type = match msg["type"].as_str() {
        Some("journal-article") => Some("article".to_string()),
        Some("book") | Some("monograph") => Some("book".to_string()),
        Some("proceedings-article") => Some("inproceedings".to_string()),
        Some("report") => Some("techreport".to_string()),
        _ => Some("misc".to_string()),
    };

    FetchedMetadata {
        title, authors, year, journal, volume, issue, pages,
        doi: Some(doi_val), issn, publisher, url: url_val, abstract_text,
        entry_type, ..Default::default()
    }
}

fn parse_datacite_attrs(attrs: &serde_json::Value, fallback_doi: &str) -> FetchedMetadata {
    let title = attrs["titles"].as_array()
        .and_then(|a| a.first()).and_then(|t| t["title"].as_str()).map(String::from);

    let authors = attrs["creators"].as_array().map(|arr| {
        arr.iter().filter_map(|c| {
            let fam = c["familyName"].as_str();
            let giv = c["givenName"].as_str();
            match (fam, giv) {
                (Some(f), Some(g)) => Some(format!("{f}, {g}")),
                (Some(f), None)    => Some(f.to_string()),
                _                  => c["name"].as_str().map(String::from),
            }
        }).collect::<Vec<_>>().join(" and ")
    }).filter(|s| !s.is_empty());

    let year = attrs["publicationYear"].as_u64().map(|y| y.to_string());
    let publisher = attrs["publisher"].as_str().map(String::from);
    let doi_val   = attrs["doi"].as_str().map(String::from)
        .unwrap_or_else(|| fallback_doi.to_string());

    let abstract_text = attrs["descriptions"].as_array().and_then(|arr| {
        arr.iter().find(|d| d["descriptionType"].as_str() == Some("Abstract"))
            .and_then(|d| d["description"].as_str())
            .map(|s| strip_xml_tags(s))
    }).filter(|s| !s.is_empty());

    let entry_type = match attrs["types"]["resourceTypeGeneral"].as_str() {
        Some("JournalArticle") => Some("article".to_string()),
        Some("Book")           => Some("book".to_string()),
        Some("ConferencePaper") => Some("inproceedings".to_string()),
        Some("Report")         => Some("techreport".to_string()),
        _                      => Some("misc".to_string()),
    };

    FetchedMetadata {
        title, authors, year, publisher,
        doi: Some(doi_val),
        abstract_text, entry_type,
        ..Default::default()
    }
}

#[tauri::command]
pub async fn fetch_doi_metadata(doi: String) -> Result<FetchedMetadata, String> {
    let clean = normalize_doi(&doi).to_string();

    let client = reqwest::Client::builder()
        .user_agent("Quire/1.0 (https://github.com/quire)")
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    // Try CrossRef first (covers most journal articles and arXiv DOIs)
    let cr_url = format!("https://api.crossref.org/works/{}", clean);
    let cr_result = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        client.get(&cr_url).send(),
    ).await;
    if let Ok(Ok(resp)) = cr_result {
        if resp.status().is_success() {
            let json_result = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                resp.json::<serde_json::Value>(),
            ).await;
            if let Ok(Ok(json)) = json_result {
                return Ok(parse_crossref_message(&json["message"], &clean));
            }
        }
    }

    // Fallback: DataCite API (covers Zenodo, figshare, etc.)
    let dc_url = format!("https://api.datacite.org/dois/{}", clean);
    let dc_result = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        client.get(&dc_url).send(),
    ).await;
    if let Ok(Ok(resp)) = dc_result {
        if resp.status().is_success() {
            let json_result = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                resp.json::<serde_json::Value>(),
            ).await;
            if let Ok(Ok(json)) = json_result {
                return Ok(parse_datacite_attrs(&json["data"]["attributes"], &clean));
            }
        }
    }

    Err(format!("DOI not found in CrossRef or DataCite: {clean}"))
}

// ── arXiv fetch ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_arxiv_metadata(arxiv_id: String) -> Result<FetchedMetadata, String> {
    let raw = arxiv_id.trim()
        .trim_start_matches("arXiv:")
        .trim_start_matches("arxiv:");
    let clean = raw.split('v').next().unwrap_or(raw).trim();

    let url = format!("https://export.arxiv.org/api/query?id_list={clean}");
    let client = reqwest::Client::builder()
        .user_agent("Quire/1.0")
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;
    let send_result = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        client.get(&url).send(),
    ).await
    .map_err(|_| "arXiv request timed out — check your internet connection.".to_string())?
    .map_err(|e| format!("Network error: {e}"))?;
    let xml = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        send_result.text(),
    ).await
    .map_err(|_| "arXiv response timed out.".to_string())?
    .map_err(|e| e.to_string())?;

    if xml.contains("<opensearch:totalResults>0</opensearch:totalResults>") {
        return Err("arXiv ID not found".to_string());
    }

    let title   = xml_tag_text(&xml, "title").filter(|s| !s.contains("ArXiv Query"));
    let summary = xml_tag_text(&xml, "summary");
    let year    = xml_tag_text(&xml, "published").and_then(|d| d.get(..4).map(String::from));
    let doi     = xml_tag_text(&xml, "arxiv:doi");
    let journal = xml_tag_text(&xml, "arxiv:journal_ref");
    let url_val = xml_tag_text(&xml, "id").filter(|s| s.contains("arxiv.org"));

    // Collect all <author> blocks
    let authors = {
        let mut names = Vec::new();
        let mut pos = 0usize;
        while let Some(rel) = xml[pos..].find("<author>") {
            let s = pos + rel;
            let e = xml[s..].find("</author>").map(|x| s + x + 9).unwrap_or(s + 8);
            if let Some(name) = xml_tag_text(&xml[s..e], "name") {
                names.push(name);
            }
            pos = e;
        }
        if names.is_empty() { None } else { Some(names.join(" and ")) }
    };

    Ok(FetchedMetadata {
        title, authors, year, journal, doi, url: url_val,
        abstract_text: summary,
        entry_type: Some("misc".to_string()),
        ..Default::default()
    })
}

// ── ISBN fetch via OpenLibrary ────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_isbn_metadata(isbn: String) -> Result<FetchedMetadata, String> {
    let clean: String = isbn.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=ISBN:{clean}&format=json&jscmd=data"
    );
    let client = reqwest::Client::builder()
        .user_agent("Quire/1.0")
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;
    let send_result = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        client.get(&url).send(),
    ).await
    .map_err(|_| "OpenLibrary request timed out — check your internet connection.".to_string())?
    .map_err(|e| format!("Network error: {e}"))?;
    let json: serde_json::Value = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        send_result.json(),
    ).await
    .map_err(|_| "OpenLibrary response timed out.".to_string())?
    .map_err(|e| e.to_string())?;

    let key  = format!("ISBN:{clean}");
    let book = json.get(&key).ok_or("ISBN not found in OpenLibrary")?;

    let title = book["title"].as_str().map(String::from);

    let authors = book["authors"].as_array().map(|arr| {
        arr.iter().filter_map(|a| a["name"].as_str()).collect::<Vec<_>>().join(" and ")
    }).filter(|s| !s.is_empty());

    // publish_date can be "April 2004", "2004", "2004-03-01"
    let year = book["publish_date"].as_str().and_then(|d| {
        d.split(|c: char| !c.is_ascii_digit()).find(|p| p.len() == 4).map(String::from)
    });

    let publisher = book["publishers"].as_array()
        .and_then(|a| a.first())
        .and_then(|p| p["name"].as_str())
        .map(String::from);

    Ok(FetchedMetadata {
        title, authors, year, publisher, isbn: Some(clean),
        entry_type: Some("book".to_string()),
        ..Default::default()
    })
}

// ── Attachments ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pick_and_attach_file(
    app: tauri::AppHandle,
    item_id: i64,
) -> Result<Option<Attachment>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter("PDF files", &["pdf", "PDF"])
        .blocking_pick_file();
    let Some(p) = picked else { return Ok(None) };
    let src = p.into_path().map_err(|e| e.to_string())?;

    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment.pdf")
        .to_string();

    let dest_dir = attachments_dir(item_id);
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(&file_name);
    fs::copy(&src, &dest).map_err(|e| e.to_string())?;

    let dest_str = dest.to_string_lossy().to_string();
    let conn = open_conn()?;
    conn.execute(
        "INSERT INTO attachments (item_id, file_name, file_path) VALUES (?1, ?2, ?3)",
        params![item_id, file_name, dest_str],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let added_at: i64 = conn
        .query_row("SELECT added_at FROM attachments WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    Ok(Some(Attachment { id, item_id, file_name, file_path: dest_str, added_at }))
}

#[tauri::command]
pub fn get_item_attachments(item_id: i64) -> Result<Vec<Attachment>, String> {
    let conn = open_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, file_name, file_path, added_at
             FROM attachments WHERE item_id=?1 ORDER BY added_at",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(Attachment {
                id:        row.get(0)?,
                item_id:   row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                added_at:  row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn read_attachment_bytes(id: i64) -> Result<Vec<u8>, String> {
    let conn = open_conn()?;
    let path: String = conn
        .query_row("SELECT file_path FROM attachments WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    fs::read(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_attachment_external(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    let path: String = conn
        .query_row("SELECT file_path FROM attachments WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    use tauri_plugin_opener::OpenerExt;
    let url = format!("file:///{}", path.replace('\\', "/"));
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_attachment(id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    let path: String = conn
        .query_row("SELECT file_path FROM attachments WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM attachments WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&path);
    Ok(())
}

// ── Annotations ───────────────────────────────────────────────────────────────

fn row_to_annotation(row: &rusqlite::Row) -> rusqlite::Result<Annotation> {
    Ok(Annotation {
        id:            row.get(0)?,
        item_id:       row.get(1)?,
        attachment_id: row.get(2)?,
        page:          row.get(3)?,
        ann_type:      row.get(4)?,
        color:         row.get(5)?,
        selected_text: row.get(6)?,
        note_text:     row.get(7)?,
        position_json: row.get(8)?,
        created_at:    row.get(9)?,
    })
}

const ANN_SELECT: &str =
    "SELECT id, item_id, attachment_id, page, ann_type, color,
            selected_text, note_text, position_json, created_at
     FROM annotations";

#[tauri::command]
pub fn create_annotation(input: AnnotationInput) -> Result<Annotation, String> {
    let conn = open_conn()?;
    conn.execute(
        "INSERT INTO annotations
             (item_id, attachment_id, page, ann_type, color,
              selected_text, note_text, position_json)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            input.item_id, input.attachment_id, input.page,
            input.ann_type, input.color,
            input.selected_text, input.note_text, input.position_json
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let sql = format!("{} WHERE id=?1", ANN_SELECT);
    conn.query_row(&sql, [id], row_to_annotation)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_annotations_for_attachment(attachment_id: i64) -> Result<Vec<Annotation>, String> {
    let conn = open_conn()?;
    let sql = format!("{} WHERE attachment_id=?1 ORDER BY page, created_at", ANN_SELECT);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([attachment_id], row_to_annotation)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn delete_annotation(id: i64) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute("DELETE FROM annotations WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_annotation_note(id: i64, note_text: String) -> Result<(), String> {
    let conn = open_conn()?;
    conn.execute(
        "UPDATE annotations SET note_text=?1 WHERE id=?2",
        params![note_text, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_all_annotations() -> Result<Vec<AnnotationWithSource>, String> {
    let conn = open_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.item_id, a.attachment_id, a.page, a.ann_type, a.color,
                    a.selected_text, a.note_text, a.created_at,
                    i.title, i.authors, i.year, i.key,
                    att.file_name
             FROM annotations a
             JOIN attachments att ON att.id = a.attachment_id
             JOIN items i ON i.id = a.item_id
             ORDER BY i.title COLLATE NOCASE, a.page, a.created_at",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AnnotationWithSource {
                id:                   row.get(0)?,
                item_id:              row.get(1)?,
                attachment_id:        row.get(2)?,
                page:                 row.get(3)?,
                ann_type:             row.get(4)?,
                color:                row.get(5)?,
                selected_text:        row.get(6)?,
                note_text:            row.get(7)?,
                created_at:           row.get(8)?,
                item_title:           row.get(9)?,
                item_authors:         row.get(10)?,
                item_year:            row.get(11)?,
                item_key:             row.get(12)?,
                attachment_file_name: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
