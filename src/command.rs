mod computation;

use csv::{ReaderBuilder, StringRecord};
use std::path::PathBuf;
use std::error::Error;
use comfy_table::{Table, presets::UTF8_FULL};
use std::collections::VecDeque;

use computation::{ColumnStats};

// Opening CSV via ReaderBuilder:
// Take the first `n` records using records().take(n) and print them in a table format.
pub fn head(path: &PathBuf, n: usize) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)  // headers in first row
        .flexible(true)     // Varied number of fields
        .from_path(path)?;

    let headers = rdr.headers()?.clone();   // clone headers for later use

    // reading first n rows
    let records: Vec<StringRecord> = rdr
        .records()
        .take(n)
        .collect::<Result<Vec<_>, _>>()?;

    print_table(&records, &headers, 0);
    Ok(())
}

// Read the file line by line using read_record (to minimize allocations).
// Store the last N rows in a circular buffer.
pub fn tail(path: &PathBuf, n: usize) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let headers = rdr.headers()?.clone();

    let mut buffer: VecDeque<StringRecord> = Vec::with_capacity(n).into();
    let mut record = StringRecord::new();

    while rdr.read_record(&mut record)? {
        if buffer.len() == n {
            buffer.pop_front();
        }
        buffer.push_back(record.clone());
    }

    let vec: Vec<_> = buffer.into_iter().collect();

    print_table(&vec, &headers, 0);
    Ok(())
}

// Calculate the number of rows and columns.
// headers.len() gives the number of columns, and the read_record loop counts the records.
pub fn shape(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let headers = rdr.headers()?.clone();
    let column_count = headers.len();

    let mut row_count: u64 = 0;
    let mut record = csv::StringRecord::new();

    while rdr.read_record(&mut record)? {
        row_count += 1;
    }

    // Changing headers to &str for display
    let header_strings: Vec<&str> = headers.iter().collect();
    
    println!("Строк: {row_count}");
    println!("Колонок: {column_count}");
    println!("Заголовки: {}", header_strings.join(", "));

    Ok(())
}

// Collecting statistics on numeric columns:
// - Initialize the structure (name, vector of values, min, max, sum, count)
// - For each row, we try to parse the fields into f64 and update the statistics
// - After the pass, calculate the mean and std (sample, divided by n-1)
pub fn stats(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let headers = rdr.headers()?.clone();

    // index map
    let mut index_map = Vec::new();
    let mut stats_vec: Vec<ColumnStats> = Vec::new();

    for header in headers.iter() {
        let lower = header.to_lowercase();

        if lower.contains("time") || lower.contains("date") || lower.contains("timestamp") {
            index_map.push(None);
            continue;
        }

        index_map.push(Some(stats_vec.len()));
        stats_vec.push(ColumnStats::new(header));
    }

    // fill stats
    for result in rdr.records() {
        let record = result?;

        for (csv_i, field) in record.iter().enumerate() {
            let Some(stat_i) = index_map[csv_i] else { continue };

            if let Ok(value) = field.trim().parse::<f64>() {
                stats_vec[stat_i].update(value);
            }
        }
    }

    // build table
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["column", "count", "mean", "std", "min", "25%", "50%", "75%", "max"]);

    for mut s in stats_vec {
        let f = s.finalize();

        if f.count == 0 {
            continue;
        }

        table.add_row(vec![
            f.name,
            f.count.to_string(),
            format!("{:.6}", f.mean),
            format!("{:.6}", f.std),
            format!("{:.6}", f.min),
            format!("{:.6}", f.p25),
            format!("{:.6}", f.p50),
            format!("{:.6}", f.p75),
            format!("{:.6}", f.max),
        ]);
    }

    println!("{table}");
    Ok(())
}

// Creating a table using comfy_table:
// - Create a header: line number + CSV headers
// - truncate the fields to the number of columns
// - We use dynamic width for an accurate terminal output.
fn print_table(records: &[StringRecord], headers: &StringRecord, start_index: usize) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth);


    let mut header_row = vec!["#".to_string()];
    header_row.extend(headers.iter().map(|s| s.to_string()));
    table.set_header(header_row);

    for (i, record) in records.iter().enumerate() {
        let mut row = vec![(start_index + i + 1).to_string()];
        
        let fields: Vec<&str> = record.iter().collect();
        let expected = headers.len();
        let mut padded = fields;
        if padded.len() < expected {
            padded.extend(std::iter::repeat("").take(expected - padded.len()));
        } else if padded.len() > expected {
            padded.truncate(expected);
        }
        
        row.extend(padded.iter().map(|s| s.to_string()));
        table.add_row(row);
    }

    println!("{table}");
    println!("Показано {} строк", records.len());
}