use std::path::Path;

use csv::ReaderBuilder;

use mantis_ta::types::Candle;

pub fn load_candles<P: AsRef<Path>>(relative: P) -> Vec<Candle> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .unwrap();
    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec.unwrap();
        out.push(Candle {
            timestamp: rec[0].parse().unwrap(),
            open: rec[1].parse().unwrap(),
            high: rec[2].parse().unwrap(),
            low: rec[3].parse().unwrap(),
            close: rec[4].parse().unwrap(),
            volume: rec[5].parse().unwrap(),
        });
    }
    out
}
