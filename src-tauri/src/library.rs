use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::BibEntry;

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
        );",
    )
    .map_err(|e| e.to_string())
}

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<LibraryItem> {
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
        tags:          vec![],
    })
}

fn fetch_item(conn: &Connection, id: i64) -> Result<Option<LibraryItem>, String> {
    conn.query_row(
        "SELECT id, key, entry_type, title, authors, year, journal, doi, abstract_text, url,
                volume, issue, pages, publisher, booktitle, edition, month, keywords, note,
                isbn, issn, number, institution, added_at, updated_at
         FROM items WHERE id=?1",
        [id],
        |row| row_to_item(row),
    )
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
    let mut stmt = conn.prepare(
        "SELECT id, key, entry_type, title, authors, year, journal, doi, abstract_text, url,
                volume, issue, pages, publisher, booktitle, edition, month, keywords, note,
                isbn, issn, number, institution, added_at, updated_at
         FROM items WHERE id NOT IN (SELECT item_id FROM trash) ORDER BY added_at ASC",
    )
    .map_err(|e| e.to_string())?;
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
    let mut stmt = conn
        .prepare(
            "SELECT id, key, entry_type, title, authors, year, journal, doi, abstract_text, url,
                    volume, issue, pages, publisher, booktitle, edition, month, keywords, note,
                    isbn, issn, number, institution, added_at, updated_at
             FROM items WHERE id NOT IN (SELECT item_id FROM trash) ORDER BY added_at DESC",
        )
        .map_err(|e| e.to_string())?;
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

    let mut conditions = vec!["id NOT IN (SELECT item_id FROM trash)".to_string()];
    let mut bind_strs: Vec<String> = vec![];

    if let Some(ref text) = query.text {
        if !text.is_empty() {
            let n = bind_strs.len() + 1;
            conditions.push(format!(
                "(title LIKE ?{n} OR authors LIKE ?{n} OR key LIKE ?{n} OR abstract_text LIKE ?{n})"
            ));
            bind_strs.push(format!("%{}%", text));
        }
    }

    if let Some(ref min) = query.year_min {
        let n = bind_strs.len() + 1;
        conditions.push(format!("year >= ?{n}"));
        bind_strs.push(min.clone());
    }

    if let Some(ref max) = query.year_max {
        let n = bind_strs.len() + 1;
        conditions.push(format!("year <= ?{n}"));
        bind_strs.push(max.clone());
    }

    if let Some(true) = query.has_doi {
        conditions.push("doi IS NOT NULL AND doi != ''".to_string());
    }

    if let Some(ref types) = query.entry_types {
        if !types.is_empty() {
            let start = bind_strs.len() + 1;
            let placeholders: Vec<String> = (start..start + types.len())
                .map(|i| format!("?{i}"))
                .collect();
            conditions.push(format!("entry_type IN ({})", placeholders.join(",")));
            bind_strs.extend(types.iter().cloned());
        }
    }

    let sql = format!(
        "SELECT id, key, entry_type, title, authors, year, journal, doi, abstract_text, url,
                volume, issue, pages, publisher, booktitle, edition, month, keywords, note,
                isbn, issn, number, institution, added_at, updated_at
         FROM items WHERE {} ORDER BY added_at DESC",
        conditions.join(" AND ")
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = bind_strs
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();
    let items = stmt
        .query_map(refs.as_slice(), |row| row_to_item(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}
