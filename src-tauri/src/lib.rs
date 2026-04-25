use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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

// ── Commands ──────────────────────────────────────────────────────────────────

#[tauri::command]
async fn open_document(
    app: tauri::AppHandle,
) -> Result<OpenResult, String> {
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
        content: content.clone(),
        frontmatter,
        body,
    })
}

#[tauri::command]
async fn save_document(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_document_as(
    app: tauri::AppHandle,
    content: String,
) -> Result<String, String> {
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

    // Common .bib locations relative to the document
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

    // Walk up one directory level
    if let Some(parent) = dir.parent() {
        let up_candidates = [
            parent.join("references.bib"),
            parent.join("bibliography.bib"),
        ];
        for candidate in &up_candidates {
            if candidate.exists() {
                return Ok(Some(candidate.to_string_lossy().to_string()));
            }
        }
    }

    Ok(None)
}

#[tauri::command]
async fn run_quarto(doc_path: String, format: String) -> Result<String, String> {
    let output = Command::new("quarto")
        .args(["render", &doc_path, "--to", &format])
        .output()
        .map_err(|e| format!("Failed to run quarto: {e}. Is Quarto installed and on PATH?"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // Return the expected output path
        let doc = Path::new(&doc_path);
        let ext = match format.as_str() {
            "pdf" => "pdf",
            "docx" => "docx",
            "html" => "html",
            _ => "pdf",
        };
        let out_path = doc
            .with_extension(ext)
            .to_string_lossy()
            .to_string();
        Ok(format!("{}\nOutput: {}", stdout.trim(), out_path))
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

    // Find closing ---
    let after_open = &content[3..];
    let close_pos = after_open.find("\n---")
        .or_else(|| after_open.find("\r\n---"));

    let Some(pos) = close_pos else {
        return (FrontmatterFields::default(), content.to_string());
    };

    let yaml = &after_open[..pos];
    let body_start = pos + 4; // skip "\n---"
    let body = after_open[body_start..].trim_start().to_string();

    let fields = parse_frontmatter_fields(yaml);
    (fields, body)
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
            // Handle "- name: ..." vs "- Author Name"
            if let Some(rest) = author.strip_prefix("name:") {
                authors.push(clean_yaml_string(rest.trim()));
            } else if !author.is_empty() {
                authors.push(clean_yaml_string(author));
            }
        } else if !trimmed.starts_with(' ') && !trimmed.starts_with('-') {
            in_authors = false;
        }
    }

    FrontmatterFields { title, authors, date }
}

fn clean_yaml_string(s: &str) -> String {
    s.trim_matches('"').trim_matches('\'').to_string()
}

// ── .bib parser ───────────────────────────────────────────────────────────────

fn parse_bib(content: &str) -> Vec<BibEntry> {
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
    // Read entry type
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
    *i += 1; // skip opening { or (

    // Read citation key
    let key_start = *i;
    while *i < chars.len() && chars[*i] != ',' {
        *i += 1;
    }
    let key: String = chars[key_start..*i].iter().collect::<String>().trim().to_string();
    if *i < chars.len() {
        *i += 1; // skip comma
    }

    // Read fields until closing }
    let mut fields: HashMap<String, String> = HashMap::new();
    parse_bib_fields(chars, i, &mut fields);

    Some(BibEntry {
        key,
        entry_type,
        title: fields.get("title").map(|s| clean_bib_braces(s)),
        authors: fields
            .get("author")
            .or_else(|| fields.get("authors"))
            .map(|s| clean_bib_braces(s)),
        year: fields.get("year").map(|s| clean_bib_braces(s)),
        journal: fields
            .get("journal")
            .or_else(|| fields.get("journaltitle"))
            .map(|s| clean_bib_braces(s)),
        doi: fields.get("doi").map(|s| clean_bib_braces(s)),
        abstract_text: fields.get("abstract").map(|s| clean_bib_braces(s)),
        url: fields.get("url").map(|s| clean_bib_braces(s)),
    })
}

fn parse_bib_fields(chars: &[char], i: &mut usize, fields: &mut HashMap<String, String>) {
    let n = chars.len();

    loop {
        // Skip whitespace and commas
        while *i < n && (chars[*i].is_whitespace() || chars[*i] == ',') {
            *i += 1;
        }

        if *i >= n || chars[*i] == '}' || chars[*i] == ')' {
            if *i < n { *i += 1; }
            break;
        }

        // Read field name
        let name_start = *i;
        while *i < n && chars[*i] != '=' && chars[*i] != '}' && !chars[*i].is_whitespace() {
            *i += 1;
        }
        if *i == name_start { break; }
        let name: String = chars[name_start..*i]
            .iter()
            .collect::<String>()
            .trim()
            .to_lowercase();

        // Skip whitespace and =
        while *i < n && (chars[*i].is_whitespace() || chars[*i] == '=') {
            *i += 1;
        }

        if *i >= n { break; }

        // Read value
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
                // Bare value (e.g. year = 2022)
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
            '{' => { depth += 1; result.push('{'); }
            '}' => {
                depth -= 1;
                if depth == 0 { *i += 1; break; }
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
    if *i < chars.len() { *i += 1; }
    result
}

fn skip_bib_block(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i] != '{' { *i += 1; }
    if *i < chars.len() { *i += 1; }
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
    let stripped = s.replace('{', "").replace('}', "");
    stripped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            open_document,
            save_document,
            save_document_as,
            load_bib,
            find_bib_for_document,
            run_quarto,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
