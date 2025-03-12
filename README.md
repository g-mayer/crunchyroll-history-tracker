# Crunchyroll Watch History Extractor  

This Rust script extracts and processes your **Crunchyroll watch history**, organizing data about watched shows, episode counts, and additional series details. It keeps track of the last processed date to prevent duplicate entries in subsequent runs.  

## Features  

✅ Fetches watch history from Crunchyroll  
✅ Extracts **series metadata** (title, slug, description, episodes, seasons, publisher, etc.)  
✅ Tracks **episodes watched per series**  
✅ Uses **cutoff_date.txt** to prevent duplicate processing  
✅ Saves extracted data as **JSON**  

---

## 📌 Prerequisites  

Before running the script, ensure you have:  

- **Rust & Cargo** installed ([Install Rust](https://www.rust-lang.org/tools/install))  
- A **Crunchyroll account**  
- Your **Crunchyroll login credentials**  

---

## 🚀 Installation & Setup  

### 1️⃣ Clone the Repository  

```bash
git clone https://github.com/yourusername/crunchyroll-history.git
cd crunchyroll-history
```

### 2️⃣ Install Dependencies  

```bash
cargo build
```

This will download and compile all required dependencies, including `crunchyroll_rs` and `serde_json`.  

### 3️⃣ Configure Your Credentials  

1. Create a **`.env` file** in your project directory  
2. Add your **Crunchyroll credentials**:  

   ```plaintext
   CR_USERNAME=your-email@example.com
   CR_PASSWORD=your-secure-password
   ```

3. The script will **automatically load credentials** when executed.  

> **⚠️ Do NOT commit `.env` to Git.** Add it to `.gitignore` to keep credentials secure.

---

## ▶️ Running the Script  

```bash
cargo run
```

### Expected Output  

The script will:  

1. **Check for `cutoff_date.txt`**  
   - If found, it will process entries **after** that date.  
   - If missing, it will warn you and process all available history.  

2. **Fetch & Process Watch History**  
   - Counts episodes watched for each series  
   - Extracts metadata for series (if not already extracted)  

3. **Save Data to JSON File**  
   - Creates a new `show_data.json` file with extracted information.  

4. **Update `cutoff_date.txt`**  
   - Saves the script's **run time** for the next execution.  

---

## 📁 Understanding the Output  

### `show_data.json` (Extracted Data)  

Each object in the JSON file represents a **series** with episode counts:  

```json
[
  {
    "series": {
      "title": "Attack on Titan",
      "slug": "attack-on-titan",
      "description": "A story about humanity's fight for survival...",
      "extendedDescription": "Longer description...",
      "episodes": 75,
      "seasons": 4,
      "publisher": "Kodansha",
      "keywords": ["Action", "Drama", "Fantasy"],
      "posterTall": "https://image-link.com/poster.jpg"
    },
    "episodesWatched": 12
  }
]
```

### `cutoff_date.txt` (Last Run Date)  

This file stores the last run's **UTC timestamp**:  

```
2025-03-11T12:00:00Z
```

On the next run, the script **skips entries before this date**.

---

## ⚙️ Customization  

### ✅ Change the History Limit  

Modify this line in `main.rs` to **limit the number of processed shows**:  

```rust
let limit: Option<usize> = Some(10); // Only process 10 shows
```

Set `None` to process **all available history**:  

```rust
let limit: Option<usize> = None;
```

---

### ✅ Extend Extracted Metadata  

If you want to **extract more fields** from series, modify this section in `main.rs`:  

```rust
let series_data = json!({
    "title": series.title,
    "slug": series.slug_title,
    "description": series.description,
    "extendedDescription": series.extended_description,
    "episodes": series.episode_count,
    "seasons": series.season_count,
    "publisher": series.content_provider.clone().unwrap_or("Unknown".to_string()),
    "keywords": series.keywords,
    "posterTall": series.images.poster_tall
        .get(2)
        .map(|img| img.source.clone())
        .unwrap_or("No image available".to_string())
});
```

For example, to include **release year**:

```rust
"releaseYear": series.series_launch_year.unwrap_or(0)
```

---

### ✅ Handle Different Cutoff Date Formats  

If `cutoff_date.txt` contains an invalid date, the script **warns and continues**.  

To enforce strict date parsing, modify `read_cutoff_date()`:  

```rust
Err(io::Error::new(io::ErrorKind::InvalidData, "Cutoff date is required!"))
```

---

### ✅ Save JSON to a Custom Directory  

Modify `get_unique_filename()` in `main.rs`:  

```rust
let output_path = format!("./data/{}", new_name);
File::create(&output_path)?;
```

---

## 🛠 Troubleshooting  

**❌ Error: Invalid date format in `cutoff_date.txt`**  
- Ensure `cutoff_date.txt` contains a **valid UTC timestamp**, e.g.:  
  ```
  2025-02-20T00:00:00Z
  ```

**❌ Error: Login failed**  
- Verify your **Crunchyroll email & password**  
- If using 2FA, generate an **app password** ([Crunchyroll Help](https://help.crunchyroll.com/))  

**❌ Error: API rate limit exceeded**  
- Try **waiting a few minutes** before running again.  