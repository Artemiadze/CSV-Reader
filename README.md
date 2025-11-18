# 📄 **CSVReader — Fast CSV Viewer for the Terminal**

`CSVReader` is a lightweight and efficient command-line tool for inspecting large CSV files directly in the terminal.
It provides fast equivalents of common data-analysis operations such as:

* Viewing the first N rows (`head`)
* Viewing the last N rows (`tail`)
* Computing statistics for numeric columns (`stats`)
* Checking the shape of the dataset (`shape`)

The tool is optimized for speed and can process gigabyte-sized CSV files with minimal memory usage.

---

## 🚀 Features

* **Fast streaming CSV reader**
* **Zero memory load** for `head`, `shape`
* **O(n)** pass for `stats`
* **Optimized tail reading** using a ring buffer (`VecDeque`)
* Supports **UTF-8 CSV files** with or without headers

---

## 🔧 Installation

### Build from source

```bash
git clone https://github.com/Artemiadze/CSV-Reader
cd csv_reader
cargo build --release
```

The compiled binary will be located at:

```
target/release/csv_reader
```

(Windows: `csv_reader.exe`)

---

## 📘 Usage

```
csv_reader <COMMAND> [OPTIONS]
```

Available commands:

| Command | Description                                |
| ------- | ------------------------------------------ |
| `head`  | Show the first N rows of the CSV file      |
| `tail`  | Show the last N rows of the CSV file       |
| `stats` | Compute statistics for each numeric column |
| `shape` | Print the number of rows and columns       |

---

# 📝 Commands & Examples

---

## 🔹 **1. View first N rows — `head`**

### Syntax

```
cargo run head [OPTIONS] <FILE>
```

### Options

`-n, --n <NUM>` — number of rows to display (default: `5`)

### Example

```bash
cargo run head -n 10 data.csv
```

---

## 🔹 **2. View last N rows — `tail`**

### Syntax

```
csv_reader tail [OPTIONS] <FILE>
```

### Options

`-n, --n <NUM>` — number of rows to display (default: `5`)

### Example

```bash
cargo run tail -n 20 data.csv
```

---

## 🔹 **3. Compute statistics — `stats`**

Calculates:

* Min
* Max
* Mean
* Count of numeric values

### Syntax

```
cargo run stats <FILE>
```

### Example

```bash
cargo run stats big_dataset.csv
```

---

## 🔹 **4. Show the shape — `shape`**

Displays the number of rows and columns in the file.

### Syntax

```
cargo run shape <FILE>
```

### Example

```bash
cargo run shape huge.csv
```

---

# ⚙️ Running With Cargo

```bash
cargo run --release -- head -n 5 data.csv
cargo run --release -- stats data.csv
cargo run --release -- tail -n 10 data.csv
cargo run --release -- shape data.csv
```

The `--` is required so Cargo correctly forwards arguments to your program.

---

## 🏎 Performance Notes

* `head` reads only the first N lines → **extremely fast**
* `tail` uses a `VecDeque` ring buffer → **constant memory usage**
* `stats` streams through the file only once
* No loading entire file into RAM
