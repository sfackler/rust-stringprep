extern crate regex;

use regex::Regex;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

// Generate character mapping tables directly from the specification.
fn main() {
    // Input from the RFC.
    let in_file = File::open("rfc3454.txt").unwrap();
    let mut reader = BufReader::new(in_file);

    // Output to a Rust source file.
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("rfc3454.rs");
    let out_file = File::create(&dest_path).unwrap();
    let mut writer = BufWriter::new(out_file);

    // Generate tables.
    include_table(&mut writer, &mut reader, "A.1");
    include_table(&mut writer, &mut reader, "B.2");
}

// Generate code for the named mapping table.
fn include_table<R: Read, W: Write>(writer: &mut BufWriter<W>, reader: &mut BufReader<R>, tablename: &str) {
    // Scan to start of table.
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line.contains("Start Table") && line.contains(tablename) {
            break;
        }
    }

    // Output table declaration.
    write!(writer, "pub const RFC3454_{}: &[(char, Option<char>, Option<&str>)] = &[\n", tablename.replace(".", "_")).unwrap();

    // For each line:
    let target_re = Regex::new(r"([0-9A-F]+)(-([0-9A-F]+))?(; ([0-9A-F]+)( ([0-9A-F]+))?( ([0-9A-F]+))?( ([0-9A-F]+))?;)?").unwrap();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        // Done when reach the end of the table.
        if line.contains("End Table") {
            break;
        }

        // Skip RFC metadata.
        if line.contains("Hoffman & Blanchet") || line.contains("RFC 3454") {
            continue;
        }

        // Generate an entry for each data line.
        if let Some(captures) = target_re.captures(&line) {
            let start = captures.get(1).unwrap();

            // '\u{start}',
            let mut entry = String::from("'\\u{");
            entry.push_str(start.as_str());
            entry.push_str("}', ");

            // '\u{start}', None,
            // '\u{start}', Some('\u{end}'),
            match captures.get(3) {
                None => entry.push_str("None, "),
                Some(end) => {
                    entry.push_str("Some('\\u{");
                    entry.push_str(end.as_str());
                    entry.push_str("}'), ");
                }
            }

            // 0-4 character replacement string
            let mut replace = String::new();
            for &i in [5, 7, 9, 11].iter() {
                match captures.get(i) {
                    None => break,
                    Some(c) => {
                        replace.push_str("\\u{");
                        replace.push_str(c.as_str());
                        replace.push_str("}");
                    }
                }
            }

            // '\u{start}', None, None
            // '\u{start}', None, Some("replace")
            if replace.is_empty() {
                entry.push_str("None");
            } else {
                entry.push_str("Some(\"");
                entry.push_str(&replace);
                entry.push_str("\")");
            }

            // Output entry for this line.
            write!(writer, "    ({}),\n", entry).unwrap();
        }
    }

    // End table definition.
    write!(writer, "];\n\n").unwrap();
}
