mod library;
pub use library::{
    // items
    create_library_item, delete_library_item, get_library_item, get_library_items,
    search_library_items, update_library_item,
    // collections
    add_item_to_collection, create_collection, delete_collection, get_collections,
    get_item_collection_ids, remove_item_from_collection, rename_collection,
    // tags
    create_tag, delete_tag, get_tags, set_item_tags, update_tag_color,
    // import / export
    import_bib_file, export_bib_file,
    // metadata fetch
    fetch_doi_metadata, fetch_arxiv_metadata, fetch_isbn_metadata,
    // attachments
    pick_and_attach_file, get_item_attachments, read_attachment_bytes,
    open_attachment_external, remove_attachment,
    // annotations
    create_annotation, get_annotations_for_attachment, get_all_annotations,
    delete_annotation, update_annotation_note,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BibEntry {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenResult {
    pub path: String,
    pub content: String,
    pub frontmatter: FrontmatterFields,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FrontmatterFields {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecentFile {
    pub path: String,
    pub title: String,
    pub last_opened: String,
}

// ── Quire home directory ──────────────────────────────────────────────────────

fn quire_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".quire")
}

fn global_bib_path() -> PathBuf {
    quire_home().join("references.bib")
}

fn recent_json_path() -> PathBuf {
    quire_home().join("recent.json")
}

const DEFAULT_BIB: &str = r#"% Quire global bibliography — ~/.quire/references.bib
% Add your sources here. All Quire documents share this file.

@article{popova2022,
  title     = {Allergen Labelling Compliance in Food Manufacturing: A Systematic Review},
  author    = {Popova, M. and Chen, L. and Okafor, R.},
  year      = {2022},
  journal   = {Food Control},
  volume    = {134},
  doi       = {10.1016/j.foodcont.2022.108940},
  abstract  = {A systematic review examining allergen labelling compliance across 14 countries. Of 847 products sampled, 34% showed discrepancies between declared allergens and actual ingredient lists, with peanut and tree nut labelling showing the highest non-compliance rates. The study identifies inconsistent definition of precautionary labelling as a primary driver of consumer confusion.}
}

@techreport{fda2021,
  title       = {Food Allergen Labeling and Consumer Protection Act: Compliance Report 2021},
  author      = {{U.S. Food and Drug Administration}},
  year        = {2021},
  institution = {FDA},
  number      = {FDA-TR-2021-0042},
  abstract    = {Annual compliance report on FALCPA implementation, documenting a 1-in-8 consumer experience with allergic reactions attributable to labelling failures in packaged foods over a 24-month surveillance period. Survey conducted across n=12,400 households with at least one member reporting a food allergy.}
}

@article{hadley2019,
  title    = {Hidden Allergens and Precautionary Labelling: Consumer Understanding and Risk},
  author   = {Hadley, C. and King, J.},
  year     = {2019},
  journal  = {Journal of Allergy and Clinical Immunology},
  volume   = {143},
  number   = {3},
  doi      = {10.1016/j.jaci.2018.11.025},
  abstract = {Cross-sectional study of consumer comprehension of precautionary allergen labels (PAL) across a 2019 baseline cohort. Finds significant variation in PAL interpretation by education level and prior allergy diagnosis. 67% of respondents misinterpreted "may contain" as indicating lower risk than a direct ingredient declaration.}
}

@techreport{fssai2023,
  title       = {Food Safety and Standards (Labelling and Display) Regulations 2020: Implementation Guidelines},
  author      = {{Food Safety and Standards Authority of India}},
  year        = {2023},
  institution = {FSSAI},
  abstract    = {Implementation guidelines for the 2020 labelling regulations, including threshold specifications for allergen declarations under Section 4.2.1 (10 mg/kg per individual allergen).}
}
"#;

fn setup_quire_dir() {
    let dir = quire_home();
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }

    let bib = global_bib_path();
    if !bib.exists() {
        fs::write(&bib, DEFAULT_BIB).ok();
    }

    let recent = recent_json_path();
    if !recent.exists() {
        fs::write(&recent, "[]").ok();
    }

    // Initialise SQLite library; migrate existing .bib on first run
    let seed_entries = fs::read_to_string(&global_bib_path())
        .map(|c| parse_bib(&c))
        .unwrap_or_default();
    library::init_library(&seed_entries);
}

// ── Quire dir commands ────────────────────────────────────────────────────────

#[tauri::command]
fn get_quire_dir() -> String {
    quire_home().to_string_lossy().to_string()
}

#[tauri::command]
fn get_global_bib() -> Result<Vec<BibEntry>, String> {
    let content = fs::read_to_string(global_bib_path()).map_err(|e| e.to_string())?;
    Ok(parse_bib(&content))
}

#[tauri::command]
fn get_global_bib_raw() -> Result<String, String> {
    fs::read_to_string(global_bib_path()).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_global_bib(content: String) -> Result<(), String> {
    fs::write(global_bib_path(), content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent_files() -> Result<Vec<RecentFile>, String> {
    let content = fs::read_to_string(recent_json_path()).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_recent_file(path: String, title: String) -> Result<(), String> {
    let content = fs::read_to_string(recent_json_path()).unwrap_or_else(|_| "[]".to_string());
    let mut recent: Vec<RecentFile> = serde_json::from_str(&content).unwrap_or_default();

    // Remove existing entry for this path, then prepend fresh entry
    recent.retain(|r| r.path != path);
    recent.insert(
        0,
        RecentFile {
            path,
            title,
            last_opened: chrono_now(),
        },
    );

    // Keep only the 15 most recent
    recent.truncate(15);

    let serialized = serde_json::to_string_pretty(&recent).map_err(|e| e.to_string())?;
    fs::write(recent_json_path(), serialized).map_err(|e| e.to_string())
}

fn chrono_now() -> String {
    // Simple ISO-8601 without pulling in chrono crate
    // Uses SystemTime which is available in std
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as seconds since epoch — frontend converts to human label
    secs.to_string()
}

// ── Document commands ─────────────────────────────────────────────────────────

#[tauri::command]
async fn open_document(app: tauri::AppHandle) -> Result<OpenResult, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("Quarto / Markdown", &["qmd", "md"])
        .blocking_pick_file()
        .ok_or("No file selected")?;

    let path_str = path
        .into_path()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string();

    let content = fs::read_to_string(&path_str).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&content);

    Ok(OpenResult {
        path: path_str,
        content,
        frontmatter,
        body,
    })
}

#[tauri::command]
async fn open_document_path(path: String) -> Result<OpenResult, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&content);
    Ok(OpenResult { path, content, frontmatter, body })
}

#[tauri::command]
async fn save_document(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_document_as(app: tauri::AppHandle, content: String) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("Quarto Document", &["qmd"])
        .set_file_name("untitled.qmd")
        .blocking_save_file()
        .ok_or("No file selected")?;

    let path_str = path
        .into_path()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string();

    fs::write(&path_str, content).map_err(|e| e.to_string())?;
    Ok(path_str)
}

#[tauri::command]
async fn load_bib(path: String) -> Result<Vec<BibEntry>, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Ok(parse_bib(&content))
}

#[tauri::command]
async fn find_bib_for_document(doc_path: String) -> Result<Option<String>, String> {
    let doc = Path::new(&doc_path);
    let dir = doc.parent().ok_or("Invalid path")?;

    let candidates = [
        doc.with_extension("bib"),
        dir.join("references.bib"),
        dir.join("bibliography.bib"),
        dir.join("refs.bib"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(Some(candidate.to_string_lossy().to_string()));
        }
    }

    if let Some(parent) = dir.parent() {
        for name in &["references.bib", "bibliography.bib"] {
            let p = parent.join(name);
            if p.exists() {
                return Ok(Some(p.to_string_lossy().to_string()));
            }
        }
    }

    Ok(None)
}

#[tauri::command]
async fn pick_import_bib(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("BibTeX", &["bib"])
        .blocking_pick_file();
    match path {
        Some(p) => Ok(Some(p.into_path().map_err(|e| e.to_string())?.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn pick_export_bib(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("BibTeX", &["bib"])
        .set_file_name("references.bib")
        .blocking_save_file();
    match path {
        Some(p) => Ok(Some(p.into_path().map_err(|e| e.to_string())?.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn run_quarto(doc_path: String, format: String) -> Result<String, String> {
    let output = Command::new("quarto")
        .args(["render", &doc_path, "--to", &format])
        .output()
        .map_err(|e| format!("Failed to run quarto: {e}. Is Quarto installed and on PATH?"))?;

    if output.status.success() {
        let ext = match format.as_str() {
            "pdf" => "pdf",
            "docx" => "docx",
            "html" => "html",
            _ => "pdf",
        };
        let out_path = Path::new(&doc_path)
            .with_extension(ext)
            .to_string_lossy()
            .to_string();
        Ok(format!("Render complete\nOutput: {out_path}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Quarto error:\n{}", stderr.trim()))
    }
}

// ── Frontmatter parsing ───────────────────────────────────────────────────────

fn split_frontmatter(content: &str) -> (FrontmatterFields, String) {
    if !content.starts_with("---") {
        return (FrontmatterFields::default(), content.to_string());
    }
    let after_open = &content[3..];
    let close_pos = after_open.find("\n---").or_else(|| after_open.find("\r\n---"));
    let Some(pos) = close_pos else {
        return (FrontmatterFields::default(), content.to_string());
    };
    let yaml = &after_open[..pos];
    let body = after_open[pos + 4..].trim_start().to_string();
    (parse_frontmatter_fields(yaml), body)
}

fn parse_frontmatter_fields(yaml: &str) -> FrontmatterFields {
    let mut title: Option<String> = None;
    let mut authors: Vec<String> = Vec::new();
    let mut date: Option<String> = None;
    let mut in_authors = false;

    for line in yaml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("title:") {
            title = Some(clean_yaml_string(rest.trim()));
            in_authors = false;
        } else if trimmed == "author:" || trimmed == "authors:" {
            in_authors = true;
        } else if let Some(rest) = trimmed.strip_prefix("date:") {
            date = Some(clean_yaml_string(rest.trim()));
            in_authors = false;
        } else if in_authors && trimmed.starts_with('-') {
            let author = trimmed.trim_start_matches('-').trim();
            let name = if let Some(rest) = author.strip_prefix("name:") {
                clean_yaml_string(rest.trim())
            } else {
                clean_yaml_string(author)
            };
            if !name.is_empty() {
                authors.push(name);
            }
        } else if !trimmed.starts_with(' ') && !trimmed.starts_with('-') && !trimmed.is_empty() {
            in_authors = false;
        }
    }

    FrontmatterFields { title, authors, date }
}

fn clean_yaml_string(s: &str) -> String {
    s.trim_matches('"').trim_matches('\'').to_string()
}

// ── .bib parser ───────────────────────────────────────────────────────────────

pub(crate) fn parse_bib(content: &str) -> Vec<BibEntry> {
    let mut entries = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '@' {
            i += 1;
            if let Some(entry) = parse_bib_entry(&chars, &mut i) {
                entries.push(entry);
            }
        } else {
            i += 1;
        }
    }
    entries
}

fn parse_bib_entry(chars: &[char], i: &mut usize) -> Option<BibEntry> {
    let type_start = *i;
    while *i < chars.len() && chars[*i] != '{' && chars[*i] != '(' {
        *i += 1;
    }
    let entry_type: String = chars[type_start..*i]
        .iter()
        .collect::<String>()
        .trim()
        .to_lowercase();

    if matches!(entry_type.as_str(), "comment" | "preamble" | "string") {
        skip_bib_block(chars, i);
        return None;
    }
    if *i >= chars.len() {
        return None;
    }
    *i += 1;

    let key_start = *i;
    while *i < chars.len() && chars[*i] != ',' {
        *i += 1;
    }
    let key: String = chars[key_start..*i].iter().collect::<String>().trim().to_string();
    if *i < chars.len() {
        *i += 1;
    }

    let mut fields: HashMap<String, String> = HashMap::new();
    parse_bib_fields(chars, i, &mut fields);

    let get = |k: &str| fields.get(k).map(|s| clean_bib_braces(s));
    Some(BibEntry {
        key,
        entry_type,
        title:         get("title"),
        authors:       fields.get("author").or_else(|| fields.get("authors")).map(|s| clean_bib_braces(s)),
        year:          get("year"),
        journal:       fields.get("journal").or_else(|| fields.get("journaltitle")).map(|s| clean_bib_braces(s)),
        doi:           get("doi"),
        abstract_text: get("abstract"),
        url:           get("url"),
        volume:        get("volume"),
        issue:         get("issue"),
        pages:         get("pages"),
        publisher:     get("publisher"),
        booktitle:     get("booktitle"),
        edition:       get("edition"),
        month:         get("month"),
        keywords:      get("keywords"),
        note:          get("note"),
        isbn:          get("isbn"),
        issn:          get("issn"),
        number:        get("number"),
        institution:   get("institution"),
    })
}

fn parse_bib_fields(chars: &[char], i: &mut usize, fields: &mut HashMap<String, String>) {
    let n = chars.len();
    loop {
        while *i < n && (chars[*i].is_whitespace() || chars[*i] == ',') {
            *i += 1;
        }
        if *i >= n || chars[*i] == '}' || chars[*i] == ')' {
            if *i < n {
                *i += 1;
            }
            break;
        }
        let name_start = *i;
        while *i < n && chars[*i] != '=' && chars[*i] != '}' && !chars[*i].is_whitespace() {
            *i += 1;
        }
        if *i == name_start {
            break;
        }
        let name: String = chars[name_start..*i]
            .iter()
            .collect::<String>()
            .trim()
            .to_lowercase();

        while *i < n && (chars[*i].is_whitespace() || chars[*i] == '=') {
            *i += 1;
        }
        if *i >= n {
            break;
        }
        let value = match chars[*i] {
            '{' => {
                *i += 1;
                read_bib_braced(chars, i)
            }
            '"' => {
                *i += 1;
                read_bib_quoted(chars, i)
            }
            _ => {
                let start = *i;
                while *i < n && chars[*i] != ',' && chars[*i] != '\n' && chars[*i] != '}' {
                    *i += 1;
                }
                chars[start..*i].iter().collect::<String>().trim().to_string()
            }
        };
        if !name.is_empty() {
            fields.insert(name, value);
        }
    }
}

fn read_bib_braced(chars: &[char], i: &mut usize) -> String {
    let mut depth = 1usize;
    let mut result = String::new();
    while *i < chars.len() {
        match chars[*i] {
            '{' => {
                depth += 1;
                result.push('{');
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    *i += 1;
                    break;
                }
                result.push('}');
            }
            c => result.push(c),
        }
        *i += 1;
    }
    result
}

fn read_bib_quoted(chars: &[char], i: &mut usize) -> String {
    let mut result = String::new();
    while *i < chars.len() && chars[*i] != '"' {
        result.push(chars[*i]);
        *i += 1;
    }
    if *i < chars.len() {
        *i += 1;
    }
    result
}

fn skip_bib_block(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i] != '{' {
        *i += 1;
    }
    if *i < chars.len() {
        *i += 1;
    }
    let mut depth = 1usize;
    while *i < chars.len() && depth > 0 {
        match chars[*i] {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        *i += 1;
    }
}

fn clean_bib_braces(s: &str) -> String {
    s.replace('{', "")
        .replace('}', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_quire_dir();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Quire home
            get_quire_dir,
            get_global_bib,
            get_global_bib_raw,
            save_global_bib,
            get_recent_files,
            add_recent_file,
            // Documents
            pick_import_bib,
            pick_export_bib,
            open_document,
            open_document_path,
            save_document,
            save_document_as,
            load_bib,
            find_bib_for_document,
            run_quarto,
            // Library — items
            get_library_items,
            get_library_item,
            create_library_item,
            update_library_item,
            delete_library_item,
            search_library_items,
            // Library — collections
            get_collections,
            create_collection,
            rename_collection,
            delete_collection,
            add_item_to_collection,
            remove_item_from_collection,
            get_item_collection_ids,
            // Library — tags
            get_tags,
            create_tag,
            update_tag_color,
            delete_tag,
            set_item_tags,
            // Library — import / export
            import_bib_file,
            export_bib_file,
            // Library — metadata fetch
            fetch_doi_metadata,
            fetch_arxiv_metadata,
            fetch_isbn_metadata,
            // Library — attachments
            pick_and_attach_file,
            get_item_attachments,
            read_attachment_bytes,
            open_attachment_external,
            remove_attachment,
            // Library — annotations
            create_annotation,
            get_annotations_for_attachment,
            get_all_annotations,
            delete_annotation,
            update_annotation_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
