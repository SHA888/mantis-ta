"""Generate a TA-Lib-parity reference for Donchian Channels.

Unlike `generate_ad_reference.py` / `generate_mfi_reference.py` /
`generate_sar_reference.py`, this does NOT require the TA-Lib C library.
Donchian Channels are a rolling highest-high / lowest-low over a window
(`upper = MAX(high, period)`, `lower = MIN(low, period)`,
`middle = (upper + lower) / 2`). Unlike recursive formulas (EMA, RSI, ATR,
AD, SAR) where accumulation order can introduce tiny floating-point drift
between implementations, MAX/MIN over a fixed window is an exact,
order-independent comparison — a straight Python reimplementation is
bit-identical to what TA-Lib's `MAX`/`MIN` functions would produce, so it
satisfies the crate's "verified TA-Lib parity" guarantee without needing
the native library installed.

Run:
    python3 fixtures/generate_donchian_reference.py
"""

from pathlib import Path
import csv
import json
from typing import List, Optional

ROOT = Path(__file__).resolve().parent
REFERENCE_DIR = ROOT / "reference"
MARKET_DATA_CSV = ROOT / "market_data" / "spy_daily_5y.csv"
PERIOD = 20


def load_hl(csv_path: Path) -> tuple[List[float], List[float]]:
    highs, lows = [], []
    with csv_path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            highs.append(float(row["high"]))
            lows.append(float(row["low"]))
    return highs, lows


def donchian(
    highs: List[float], lows: List[float], period: int
) -> tuple[List[Optional[float]], List[Optional[float]], List[Optional[float]]]:
    uppers: List[Optional[float]] = []
    middles: List[Optional[float]] = []
    lowers: List[Optional[float]] = []
    for i in range(len(highs)):
        if i + 1 < period:
            uppers.append(None)
            middles.append(None)
            lowers.append(None)
            continue
        window_high = max(highs[i + 1 - period : i + 1])
        window_low = min(lows[i + 1 - period : i + 1])
        uppers.append(window_high)
        lowers.append(window_low)
        middles.append((window_high + window_low) / 2.0)
    return uppers, middles, lowers


def main() -> None:
    highs, lows = load_hl(MARKET_DATA_CSV)
    uppers, middles, lowers = donchian(highs, lows, PERIOD)

    REFERENCE_DIR.mkdir(parents=True, exist_ok=True)
    for name, series in (
        (f"donchian_upper_{PERIOD}.json", uppers),
        (f"donchian_middle_{PERIOD}.json", middles),
        (f"donchian_lower_{PERIOD}.json", lowers),
    ):
        out_path = REFERENCE_DIR / name
        with out_path.open("w", encoding="utf-8") as f:
            json.dump(series, f)
        print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
