"""Generate a real TA-Lib reference for the Accumulation/Distribution Line (AD).

Unlike `generate_references.py` (which reimplements formulas in Python to
avoid a TA-Lib dependency), this script requires the actual TA-Lib C library
and its Python binding. TA-Lib ships a native `AD` function, so the crate's
"verified against TA-Lib to < 1e-10 relative error" guarantee should be backed
by a real reference series rather than hand-computed synthetic values alone.

Install (no sudo, user-local build):
    curl -fsSL -o /tmp/ta-lib.tar.gz \
        https://github.com/ta-lib/ta-lib/releases/download/v0.6.4/ta-lib-0.6.4-src.tar.gz
    tar xzf /tmp/ta-lib.tar.gz -C /tmp
    (cd /tmp/ta-lib-0.6.4 && ./configure --prefix="$HOME/.local/ta-lib" && make -j"$(nproc)" && make install)
    TA_LIBRARY_PATH="$HOME/.local/ta-lib/lib" TA_INCLUDE_PATH="$HOME/.local/ta-lib/include" \
        pip install TA-Lib numpy

Run:
    LD_LIBRARY_PATH="$HOME/.local/ta-lib/lib" python3 fixtures/generate_ad_reference.py
"""

from pathlib import Path
import csv
import json

import numpy as np
import talib

ROOT = Path(__file__).resolve().parent
REFERENCE_DIR = ROOT / "reference"
MARKET_DATA_CSV = ROOT / "market_data" / "spy_daily_5y.csv"


def load_hlcv(csv_path: Path) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    highs, lows, closes, volumes = [], [], [], []
    with csv_path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            highs.append(float(row["high"]))
            lows.append(float(row["low"]))
            closes.append(float(row["close"]))
            volumes.append(float(row["volume"]))
    return (
        np.array(highs, dtype=float),
        np.array(lows, dtype=float),
        np.array(closes, dtype=float),
        np.array(volumes, dtype=float),
    )


def main() -> None:
    high, low, close, volume = load_hlcv(MARKET_DATA_CSV)
    ad = talib.AD(high, low, close, volume)
    series = [None if np.isnan(v) else float(v) for v in ad]

    REFERENCE_DIR.mkdir(parents=True, exist_ok=True)
    out_path = REFERENCE_DIR / "ad.json"
    with out_path.open("w", encoding="utf-8") as f:
        json.dump(series, f)
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
