//! tb - precompiled helpers for jobs that would otherwise need a shell pipeline or a new
//! `rs` script: inspect JSON, convert data formats, summarize CSV, dump Excel sheets,
//! count lines and sum up directory sizes. Built once by `rs --warmup`, so it starts
//! instantly. Searching text stays with `rg`.
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "tb", about = "Precompiled data helpers of the Rust toolbox")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Pretty-print JSON, JSON5, YAML or TOML, optionally only the value at a JSON pointer.
    Json {
        /// Input file; `-` or nothing reads stdin
        file: Option<PathBuf>,
        /// JSON pointer such as /items/0/name
        #[arg(short, long)]
        pointer: Option<String>,
        /// Print the keys (objects) or the length (arrays) instead of the value
        #[arg(short, long)]
        keys: bool,
        /// Compact single-line output
        #[arg(short, long)]
        compact: bool,
    },
    /// Convert between JSON, JSON5, YAML, TOML and CSV (records as array of objects).
    Conv {
        /// Input file; the format comes from its extension
        file: PathBuf,
        /// Output format
        #[arg(short, long)]
        to: Format,
    },
    /// Summarize a CSV file: columns with type, empty cells and min/max/mean of numbers.
    Csv {
        file: PathBuf,
        /// Field delimiter
        #[arg(short, long, default_value_t = ',')]
        delimiter: char,
        /// Also print the first N rows
        #[arg(short = 'n', long, default_value_t = 0)]
        head: usize,
    },
    /// Print an Excel/ODS sheet as CSV, or list the sheets.
    Xlsx {
        file: PathBuf,
        /// Sheet name; default is the first sheet
        #[arg(short, long)]
        sheet: Option<String>,
        /// List the sheet names with their sizes
        #[arg(short, long)]
        list: bool,
    },
    /// Count identical lines (or the first regex capture group), most frequent first.
    Count {
        /// Input file; `-` or nothing reads stdin
        file: Option<PathBuf>,
        /// Count the first capture group (or the whole match) of this regex instead of lines
        #[arg(short, long)]
        regex: Option<String>,
        /// Show only the N most frequent values
        #[arg(short = 'n', long)]
        top: Option<usize>,
    },
    /// Size and file count of each entry in a directory, largest first.
    Size {
        #[arg(default_value = ".")]
        dir: PathBuf,
        /// Show only the N largest entries
        #[arg(short = 'n', long)]
        top: Option<usize>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Yaml,
    Toml,
    Csv,
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Json { file, pointer, keys, compact } => json(file, pointer, keys, compact),
        Cmd::Conv { file, to } => conv(&file, to),
        Cmd::Csv { file, delimiter, head } => csv_summary(&file, delimiter, head),
        Cmd::Xlsx { file, sheet, list } => xlsx(&file, sheet, list),
        Cmd::Count { file, regex, top } => count(file, regex, top),
        Cmd::Size { dir, top } => size(&dir, top),
    }
}

fn read_input(file: Option<&Path>) -> Result<String> {
    match file {
        Some(p) if p != Path::new("-") => {
            std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))
        }
        _ => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
    }
}

/// Parse text by file extension; stdin and unknown extensions try JSON, then YAML.
fn parse(text: &str, path: Option<&Path>) -> Result<serde_json::Value> {
    let ext = path.and_then(|p| p.extension()).and_then(|e| e.to_str()).unwrap_or("");
    Ok(match ext.to_ascii_lowercase().as_str() {
        "json" => serde_json::from_str(text)?,
        "json5" | "jsonc" => json5::from_str(text)?,
        "yaml" | "yml" => serde_yaml_ng::from_str(text)?,
        "toml" => toml::from_str(text)?,
        "csv" | "tsv" => {
            let delim = if ext.eq_ignore_ascii_case("tsv") { b'\t' } else { b',' };
            let mut r = csv::ReaderBuilder::new().delimiter(delim).from_reader(text.as_bytes());
            let head = r.headers()?.clone();
            let mut rows = Vec::new();
            for rec in r.records() {
                let rec = rec?;
                let obj = head.iter().zip(rec.iter())
                    .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
                    .collect();
                rows.push(serde_json::Value::Object(obj));
            }
            serde_json::Value::Array(rows)
        }
        _ => serde_json::from_str(text).or_else(|_| serde_yaml_ng::from_str(text))
            .context("input is neither JSON nor YAML")?,
    })
}

fn json(file: Option<PathBuf>, pointer: Option<String>, keys: bool, compact: bool) -> Result<()> {
    let value = parse(&read_input(file.as_deref())?, file.as_deref())?;
    let v = match &pointer {
        Some(p) => value.pointer(p).with_context(|| format!("nothing at {p}"))?,
        None => &value,
    };
    if keys {
        match v {
            serde_json::Value::Object(m) => m.keys().for_each(|k| println!("{k}")),
            serde_json::Value::Array(a) => println!("{}", a.len()),
            _ => bail!("value is neither object nor array"),
        }
    } else if let (serde_json::Value::String(s), true) = (v, pointer.is_some()) {
        println!("{s}"); // raw string like `jq -r`
    } else if compact {
        println!("{}", serde_json::to_string(v)?);
    } else {
        println!("{}", serde_json::to_string_pretty(v)?);
    }
    Ok(())
}

fn conv(file: &Path, to: Format) -> Result<()> {
    let value = parse(&read_input(Some(file))?, Some(file))?;
    let out = match to {
        Format::Json => serde_json::to_string_pretty(&value)? + "\n",
        Format::Yaml => serde_yaml_ng::to_string(&value)?,
        Format::Toml => toml::to_string_pretty(&value)?,
        Format::Csv => {
            let rows = value.as_array().context("CSV output needs an array of objects")?;
            let mut cols: Vec<String> = Vec::new();
            for r in rows {
                for k in r.as_object().context("CSV output needs an array of objects")?.keys() {
                    if !cols.contains(k) { cols.push(k.clone()); }
                }
            }
            let mut w = csv::Writer::from_writer(Vec::new());
            w.write_record(&cols)?;
            for r in rows {
                w.write_record(cols.iter().map(|c| match &r[c] {
                    serde_json::Value::Null => String::new(),
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                }))?;
            }
            String::from_utf8(w.into_inner()?)?
        }
    };
    std::io::stdout().lock().write_all(out.as_bytes())?;
    Ok(())
}

fn csv_summary(file: &Path, delimiter: char, head: usize) -> Result<()> {
    let mut r = csv::ReaderBuilder::new().delimiter(delimiter as u8).flexible(true)
        .from_path(file).with_context(|| format!("reading {}", file.display()))?;
    let names: Vec<String> = r.headers()?.iter().map(String::from).collect();
    // Per column: empty cells, numeric count, min, max, sum, first non-numeric value.
    let mut stats = vec![(0usize, 0usize, f64::INFINITY, f64::NEG_INFINITY, 0f64, None::<String>); names.len()];
    let mut rows = 0;
    let mut out = csv::Writer::from_writer(std::io::stdout());
    if head > 0 { out.write_record(&names)?; }
    for rec in r.records() {
        let rec = rec?;
        if rows < head { out.write_record(&rec)?; }
        rows += 1;
        for (i, cell) in rec.iter().enumerate().take(names.len()) {
            let s = &mut stats[i];
            let cell = cell.trim();
            if cell.is_empty() { s.0 += 1; continue; }
            match cell.parse::<f64>() {
                Ok(x) => { s.1 += 1; s.2 = s.2.min(x); s.3 = s.3.max(x); s.4 += x; }
                Err(_) => if s.5.is_none() { s.5 = Some(cell.to_string()) },
            }
        }
    }
    out.flush()?;
    if head > 0 { println!(); }
    println!("{rows} rows, {} columns", names.len());
    let mut t = comfy_table::Table::new();
    t.load_style(comfy_table::presets::UTF8_FULL_CONDENSED)
        .set_header(["column", "type", "empty", "min", "max", "mean / example"]);
    for (name, (empty, nums, min, max, sum, text)) in names.iter().zip(&stats) {
        let kind = match (*nums > 0, text.is_some()) { (true, false) => "number", (true, true) => "mixed", _ => "text" };
        let (lo, hi) = if *nums > 0 { (fmt(*min), fmt(*max)) } else { Default::default() };
        let last = match text {
            None if *nums > 0 => fmt(sum / *nums as f64),
            _ => text.clone().unwrap_or_default().chars().take(40).collect(),
        };
        t.add_row([name.clone(), kind.into(), empty.to_string(), lo, hi, last]);
    }
    println!("{t}");
    Ok(())
}

fn fmt(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 { format!("{x:.0}") } else { format!("{x:.4}") }
}

fn xlsx(file: &Path, sheet: Option<String>, list: bool) -> Result<()> {
    use calamine::Reader;
    let mut wb = calamine::open_workbook_auto(file).with_context(|| format!("opening {}", file.display()))?;
    let names = wb.sheet_names();
    if list {
        for n in &names {
            let (h, w) = wb.worksheet_range(n)?.get_size();
            println!("{n}\t{h} rows x {w} columns");
        }
        return Ok(());
    }
    let name = match sheet {
        Some(s) => s,
        None => names.first().context("workbook has no sheets")?.clone(),
    };
    let range = wb.worksheet_range(&name).with_context(|| format!("sheet {name}"))?;
    let mut out = csv::Writer::from_writer(std::io::stdout());
    for row in range.rows() {
        out.write_record(row.iter().map(|c| c.to_string()))?;
    }
    out.flush()?;
    Ok(())
}

fn count(file: Option<PathBuf>, regex: Option<String>, top: Option<usize>) -> Result<()> {
    let text = read_input(file.as_deref())?;
    let re = regex.as_deref().map(regex::Regex::new).transpose()?;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for line in text.lines() {
        match &re {
            Some(re) => for c in re.captures_iter(line) {
                let m = c.get(1).or_else(|| c.get(0)).unwrap();
                *counts.entry(m.as_str()).or_default() += 1;
            },
            None => *counts.entry(line).or_default() += 1,
        }
    }
    let mut v: Vec<_> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut out = std::io::stdout().lock();
    for (k, n) in v.iter().take(top.unwrap_or(usize::MAX)) {
        writeln!(out, "{n:>8}  {k}")?;
    }
    Ok(())
}

fn size(dir: &Path, top: Option<usize>) -> Result<()> {
    let mut entries = Vec::new();
    for e in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let e = e?;
        let (mut bytes, mut files) = (0u64, 0u64);
        for f in walkdir::WalkDir::new(e.path()).into_iter().filter_map(|f| f.ok()) {
            if f.file_type().is_file() {
                bytes += f.metadata().map(|m| m.len()).unwrap_or(0);
                files += 1;
            }
        }
        entries.push((bytes, files, e.file_name().to_string_lossy().into_owned()));
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    let (total, total_files) = entries.iter().fold((0, 0), |a, e| (a.0 + e.0, a.1 + e.1));
    for (bytes, files, name) in entries.iter().take(top.unwrap_or(usize::MAX)) {
        println!("{:>10}  {files:>8} files  {name}", bytesize::ByteSize(*bytes).to_string());
    }
    println!("{:>10}  {total_files:>8} files  total", bytesize::ByteSize(total).to_string());
    Ok(())
}
