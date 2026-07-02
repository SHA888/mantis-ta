"""Generate a real TA-Lib reference for Parabolic SAR.

Unlike `generate_references.py` (which reimplements formulas in Python to
avoid a TA-Lib dependency), this script requires the actual TA-Lib C library
and its Python binding, since SAR's reversal/bootstrap clamping is easy to
get subtly wrong and self-consistency tests alone won't catch a divergence.

Install (no sudo, user-local build):
    curl -fsSL -o /tmp/ta-lib.tar.gz \
        https://github.com/ta-lib/ta-lib/releases/download/v0.6.4/ta-lib-0.6.4-src.tar.gz
    tar xzf /tmp/ta-lib.tar.gz -C /tmp
    (cd /tmp/ta-lib-0.6.4 && ./configure --prefix="$HOME/.local/ta-lib" && make -j"$(nproc)" && make install)
    TA_LIBRARY_PATH="$HOME/.local/ta-lib/lib" TA_INCLUDE_PATH="$HOME/.local/ta-lib/include" \
        pip install TA-Lib numpy

Run:
    LD_LIBRARY_PATH="$HOME/.local/ta-lib/lib" python3 fixtures/generate_sar_reference.py
"""

from pathlib import Path
import csv
import json

import numpy as np
import talib

ROOT = Path(__file__).resolve().parent
REFERENCE_DIR = ROOT / "reference"
MARKET_DATA_CSV = ROOT / "market_data" / "spy_daily_5y.csv"


def load_high_low(csv_path: Path) -> tuple[np.ndarray, np.ndarray]:
    highs, lows = [], []
    with csv_path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            highs.append(float(row["high"]))
            lows.append(float(row["low"]))
    return np.array(highs, dtype=float), np.array(lows, dtype=float)


def main() -> None:
    high, low = load_high_low(MARKET_DATA_CSV)
    sar = talib.SAR(high, low, acceleration=0.02, maximum=0.2)
    series = [None if np.isnan(v) else float(v) for v in sar]

    REFERENCE_DIR.mkdir(parents=True, exist_ok=True)
    out_path = REFERENCE_DIR / "sar.json"
    with out_path.open("w", encoding="utf-8") as f:
        json.dump(series, f)
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
