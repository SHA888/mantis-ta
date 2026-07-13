"""Generate a real TA-Lib reference for the Accumulation/Distribution Line (AD).

Unlike `generate_references.py` (which reimplements formulas in Python to
avoid a TA-Lib dependency), this script requires the actual TA-Lib C library
and its Python binding. TA-Lib ships a native `AD` function, so the crate's
"verified against TA-Lib to < 1e-10 relative error" guarantee should be backed
by a real reference series rather than hand-computed synthetic values alone.

This script generates its own dedicated synthetic dataset
(`market_data/ad_parity_synthetic.csv`) rather than reusing
`market_data/spy_daily_5y.csv`. That shared fixture has `close` pinned to the
exact midpoint of `high`/`low` on every bar (by construction: `high =
close+0.3`, `low = close-0.3`), which makes the money-flow-multiplier
`((close-low)-(high-close))/(high-low)` ~0 on every bar — a test built on it
cannot distinguish a correct AD implementation from a broken one (sign flip,
swapped high/low, wrong divisor would all still pass). This dataset instead
varies the close's position within the high/low range bar-to-bar (roughly
[-0.8, 0.8] money-flow-multiplier), so the reference series actually
exercises AD's accumulation behavior in both directions.

Install (no sudo, user-local build):
    curl -fsSL -o /tmp/ta-lib.tar.gz \
        https://github.com/ta-lib/ta-lib/releases/download/v0.6.4/ta-lib-0.6.4-src.tar.gz
    tar xzf /tmp/ta-lib.tar.gz -C /tmp
    (cd /tmp/ta-lib-0.6.4 && ./configure --prefix="$HOME/.local/ta-lib" && make -j"$(nproc)" && make install)
    TA_LIBRARY_PATH="$HOME/.local/ta-lib/lib" TA_INCLUDE_PATH="$HOME/.local/ta-lib/include" \
        python3 -m pip install --break-system-packages TA-Lib numpy

Run:
    LD_LIBRARY_PATH="$HOME/.local/ta-lib/lib" python3 fixtures/generate_ad_reference.py
"""

from pathlib import Path
import csv
import json
import math

import numpy as np
import talib

ROOT = Path(__file__).resolve().parent
REFERENCE_DIR = ROOT / "reference"
MARKET_DATA_DIR = ROOT / "market_data"
DATASET_CSV = MARKET_DATA_DIR / "ad_parity_synthetic.csv"
N_BARS = 2000


def generate_non_degenerate_candles(n: int) -> list[dict[str, float]]:
    """Deterministic synthetic HLCV where close is NOT pinned to the high/low midpoint."""
    rows = []
    price = 100.0
    for i in range(n):
        price += 0.05 + math.sin(i / 15) * 0.5
        half_range = 0.3
        low = price - half_range
        high = price + half_range
        # Oscillate close's position within [low, high] between ~10% and ~90%
        # of the range (money-flow-multiplier ~ [-0.8, 0.8]), instead of
        # always sitting at the exact midpoint (multiplier == 0).
        clv_frac = 0.5 + 0.4 * math.sin(i / 7)
        close = low + clv_frac * (high - low)
        open_ = low + (1.0 - clv_frac) * (high - low)
        volume = 1_000.0 + 400.0 * math.sin(i / 9) + i * 0.05
        rows.append(
            {
                "timestamp": float(i),
                "open": open_,
                "high": high,
                "low": low,
                "close": close,
                "volume": volume,
            }
        )
    return rows


def write_dataset_csv(rows: list[dict[str, float]], path: Path) -> None:
    MARKET_DATA_DIR.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        f.write("timestamp,open,high,low,close,volume\n")
        for r in rows:
            f.write(
                f"{int(r['timestamp'])},{r['open']:.15g},{r['high']:.15g},"
                f"{r['low']:.15g},{r['close']:.15g},{r['volume']:.15g}\n"
            )
    print(f"wrote {path}")


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
    rows = generate_non_degenerate_candles(N_BARS)
    write_dataset_csv(rows, DATASET_CSV)

    high, low, close, volume = load_hlcv(DATASET_CSV)
    ad = talib.AD(high, low, close, volume)
    series = [None if np.isnan(v) else float(v) for v in ad]

    REFERENCE_DIR.mkdir(parents=True, exist_ok=True)
    out_path = REFERENCE_DIR / "ad.json"
    with out_path.open("w", encoding="utf-8") as f:
        json.dump(series, f)
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
