extern crate regex;

use regex::Regex;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufWriter};

// Generate character mapping tables directly from the specification.
fn main() {
    // Input from the RFC.
    let reader = include_bytes!("rfc3454.txt");

    // Output to a Rust source file.
    let out_file = File::create("../src/rfc3454.rs").unwrap();
    let mut writer = BufWriter::new(out_file);

    // Generate tables.
    include_table(&mut writer, &mut &reader[..], "A.1");
    include_table(&mut writer, &mut &reader[..], "B.2");
}

// Generate code for the named mapping table.
fn include_table<R: BufRead, W: Write>(writer: &mut W, reader: &mut R, tablename: &str) {
    // Scan to start of table.
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line.contains("Start Table") && line.contains(tablename) {
            break;
        }
    }

    // Output table declaration.
    write!(writer, "pub const {}: &[(char, char, &str)] = &[\n", tablename.replace(".", "_")).unwrap();

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
            // start char
            let start = captures.get(1).unwrap().as_str();

            // end char (inclusive)
            let end = captures.get(3).map_or(start, |m| m.as_str());

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

            write!(writer, "    ('\\u{{{}}}', '\\u{{{}}}', \"{}\"),\n", start, end, replace).unwrap()
        }
    }

    // End table definition.
    write!(writer, "];\n\n").unwrap();
}
