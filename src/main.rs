use clap::{Parser, Subcommand};
use csv::{ReaderBuilder, StringRecord};
use std::error::Error;
use std::path::PathBuf;
use comfy_table::{Table, presets::UTF8_FULL};

// Defining the command-line interface (clap).
// The subcommands:
//  - Head: show the first N lines
//  - Tail: show the last N lines
//  - Stats: calculate statistics by numeric columns
//  - Shape: show the number of rows/columns and headers
#[derive(Parser)]
#[command(name = "csvview")]
#[command(about = "Быстрый просмотр больших CSV-файлов в терминале")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}  

#[derive(Subcommand)]
enum Commands {
    Head { 
        file: PathBuf, 
        #[arg(short, long, default_value_t = 5)] 
        n: usize 
    },
    
    Tail { 
        file: PathBuf, 
        #[arg(short, long, default_value_t = 5)] 
        n: usize 
    },
    
    Stats { file: PathBuf },

    Shape { file: PathBuf },
}

// The application's entry point.
// Parse the CLI using clap and dispatch to the appropriate function for the selected subcommand.
// Return the Result to handle possible errors (IO, CSV parsing, etc.) correctly.
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Head { file, n } => head(&file, n)?,
        Commands::Tail { file, n } => tail(&file, n)?,
        Commands::Stats { file } => stats(&file)?,
        Commands::Shape { file } => shape(&file)?,
    }

    Ok(())
}

// Opening CSV via ReaderBuilder:
// Take the first `n` records using records().take(n) and print them in a table format.
fn head(path: &PathBuf, n: usize) -> Result<(), Box<dyn Error>> {
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
fn tail(path: &PathBuf, n: usize) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let headers = rdr.headers()?.clone();

    let mut buffer: Vec<StringRecord> = Vec::with_capacity(n);
    let mut record = StringRecord::new();

    while rdr.read_record(&mut record)? {
        if buffer.len() == n {
            buffer.remove(0);
        }
        buffer.push(record.clone());
    }

    let start_idx = if buffer.len() > n { buffer.len() - n } else { 0 };
    print_table(&buffer[start_idx..], &headers, start_idx);
    Ok(())
}

// Calculate the number of rows and columns.
// headers.len() gives the number of columns, and the read_record loop counts the records.
fn shape(path: &PathBuf) -> Result<(), Box<dyn Error>> {
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
fn stats(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;

    let headers = rdr.headers()?.clone();
    println!("Файл содержит колонок: {}", headers.len());

    // All stats
    let mut column_stats: Vec<(
        String,                // column name
        Vec<f64>,              // values
        f64,                   // min
        f64,                   // max
        f64,                   // sum
        u64                    // count
    )> = Vec::new();

    for header in headers.iter() {
        if header == "timestamp" || header.contains("time") || header.contains("date") {
            continue; // miss timestamp columns
        }
        column_stats.push((header.to_string(), Vec::new(), f64::INFINITY, f64::NEG_INFINITY, 0.0, 0));
    }

    for result in rdr.records() {
        let record = result?;
        for (i, field) in record.iter().enumerate() {
            if i >= column_stats.len() { continue; }
            if let Ok(value) = field.trim().parse::<f64>() {
                if let Some(stat) = column_stats.get_mut(i) {
                    stat.1.push(value);                    // values
                    stat.2 = stat.2.min(value);           // min
                    stat.3 = stat.3.max(value);           // max
                    stat.4 += value;                      // sum
                    stat.5 += 1;                          // count
                }
            }
        }
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
         .set_header(vec!["column", "count", "mean", "std", "min", "25%", "50%", "75%", "max"]);

    for (name, mut values, min, max, sum, count) in column_stats {
        if count == 0 { continue; }

        let mean = sum / count as f64;
        
        // Compute standard deviation
        let std = if values.len() > 1 {
            let variance = values.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / (values.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        // Compute percentiles
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p25 = if !values.is_empty() { 
            let idx = (values.len() as f64 * 0.25) as usize;
            values[idx.min(values.len() - 1)]
        } else { 0.0 };
        
        let p50 = if !values.is_empty() { 
            let idx = (values.len() as f64 * 0.5) as usize;
            values[idx.min(values.len() - 1)]
        } else { 0.0 };
        
        let p75 = if !values.is_empty() { 
            let idx = (values.len() as f64 * 0.75) as usize;
            values[idx.min(values.len() - 1)]
        } else { 0.0 };

        table.add_row(vec![
            name,
            count.to_string(),
            format!("{:.6}", mean),
            format!("{:.6}", std),
            format!("{:.6}", min),
            format!("{:.6}", p25),
            format!("{:.6}", p50),
            format!("{:.6}", p75),
            format!("{:.6}", max),
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