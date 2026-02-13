use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};

use csv::ReaderBuilder;
use serde_json::Value;

use mantis_ta::types::Candle;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures_path() -> PathBuf {
    manifest_path().join("fixtures")
}

/// Load candles from a CSV file under `fixtures/`.
///
/// Expected columns: timestamp,open,high,low,close,volume
pub fn load_candles<P: AsRef<Path>>(relative: P) -> Result<Vec<Candle>, Box<dyn Error>> {
    let path = fixtures_path().join(relative);
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        let timestamp: i64 = rec[0].parse()?;
        let open: f64 = rec[1].parse()?;
        let high: f64 = rec[2].parse()?;
        let low: f64 = rec[3].parse()?;
        let close: f64 = rec[4].parse()?;
        let volume: f64 = rec[5].parse()?;
        out.push(Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });
    }
    Ok(out)
}

/// Load a JSON value from `fixtures/reference/`.
pub fn load_reference_json<P: AsRef<Path>>(relative: P) -> Result<Value, Box<dyn Error>> {
    let path = fixtures_path().join("reference").join(relative);
    let file = File::open(path)?;
    let value = serde_json::from_reader(file)?;
    Ok(value)
}

/// Load a reference series stored as a JSON array of numbers or nulls.
pub fn load_reference_series<P: AsRef<Path>>(
    relative: P,
) -> Result<Vec<Option<f64>>, Box<dyn Error>> {
    let value = load_reference_json(relative)?;
    let arr = value
        .as_array()
        .ok_or_else(|| "expected JSON array".to_string())?;
    let series = arr
        .iter()
        .map(|v| match v {
            Value::Null => Ok(None),
            Value::Number(n) => n
                .as_f64()
                .map(Some)
                .ok_or_else(|| "expected f64".to_string()),
            _ => Err("expected number or null".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(series)
}
