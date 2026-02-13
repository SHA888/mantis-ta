"""Generate reference JSONs for Tier 1 indicators using a synthetic dataset.

This does **not** depend on TA-Lib; it uses the same formulas as the Rust
implementations to produce reference series for verification tests.

When you have real market data and TA-Lib available, replace the synthetic
source with your datasets and, optionally, TA-Lib parity checks.
"""

from pathlib import Path
import json
import math
from typing import List, Optional, Dict, Tuple

ROOT = Path(__file__).resolve().parent
REFERENCE_DIR = ROOT / "reference"
MARKET_DATA_DIR = ROOT / "market_data"


def ensure_dirs() -> None:
    REFERENCE_DIR.mkdir(parents=True, exist_ok=True)
    MARKET_DATA_DIR.mkdir(parents=True, exist_ok=True)


def write_json(name: str, data) -> None:
    path = REFERENCE_DIR / name
    with path.open("w", encoding="utf-8") as f:
        json.dump(data, f)
    print(f"wrote {path}")


def write_csv(name: str, rows: List[Dict[str, float]]) -> None:
    path = MARKET_DATA_DIR / name
    with path.open("w", encoding="utf-8") as f:
        f.write("timestamp,open,high,low,close,volume\n")
        for r in rows:
            f.write(
                f"{int(r['timestamp'])},{r['open']:.15g},{r['high']:.15g},{r['low']:.15g},{r['close']:.15g},{r['volume']:.15g}\n"
            )
    print(f"wrote {path}")


def synthetic_candles(n: int = 240) -> List[Dict[str, float]]:
    rows = []
    price = 100.0
    ts = 0
    for i in range(n):
        drift = 0.05
        shock = math.sin(i / 15) * 0.5
        price += drift + shock
        high = price + 0.3
        low = price - 0.3
        open_ = price - 0.1
        close = price
        volume = 1_000 + 50 * math.sin(i / 7)
        rows.append(
            {
                "timestamp": ts,
                "open": open_,
                "high": high,
                "low": low,
                "close": close,
                "volume": volume,
            }
        )
        ts += 86_400_000  # 1 day ms
    return rows


def sma(values: List[float], period: int) -> List[Optional[float]]:
    out = []
    window_sum = 0.0
    for i, v in enumerate(values):
        window_sum += v
        if i >= period:
            window_sum -= values[i - period]
        if i + 1 >= period:
            out.append(window_sum / period)
        else:
            out.append(None)
    return out


def ema(values: List[float], period: int) -> List[Optional[float]]:
    """EMA seeded with SMA of first `period` values (matches Rust impl)."""
    out: List[Optional[float]] = []
    k = 2.0 / (period + 1)
    ema_val: Optional[float] = None
    window: List[float] = []
    for v in values:
        if ema_val is None:
            window.append(v)
            if len(window) < period:
                out.append(None)
            else:
                ema_val = sum(window) / period
                out.append(ema_val)
        else:
            ema_val = (v - ema_val) * k + ema_val
            out.append(ema_val)
    return out


def _rsi_from_avgs(avg_gain: float, avg_loss: float) -> float:
    if avg_loss == 0.0:
        return 100.0
    if avg_gain == 0.0:
        return 0.0
    rs = avg_gain / avg_loss
    return 100.0 - 100.0 / (1.0 + rs)


def rsi(values: List[float], period: int) -> List[Optional[float]]:
    out: List[Optional[float]] = []
    prev = None
    gains = 0.0
    losses = 0.0
    avg_gain: Optional[float] = None
    avg_loss: Optional[float] = None
    count = 0
    for v in values:
        if prev is None:
            prev = v
            out.append(None)
            continue
        change = v - prev
        gain = max(change, 0.0)
        loss = max(-change, 0.0)
        if avg_gain is None:
            gains += gain
            losses += loss
            count += 1
            if count >= period:
                avg_gain = gains / period
                avg_loss = losses / period
                out.append(_rsi_from_avgs(avg_gain, avg_loss))
            else:
                out.append(None)
        else:
            avg_gain = (avg_gain * (period - 1) + gain) / period
            avg_loss = (avg_loss * (period - 1) + loss) / period
            out.append(_rsi_from_avgs(avg_gain, avg_loss))
        prev = v
    return out


def bollinger(values: List[float], period: int, k_std: float) -> Tuple[List[Optional[float]], List[Optional[float]], List[Optional[float]]]:
    mids = sma(values, period)
    uppers: List[Optional[float]] = []
    lowers: List[Optional[float]] = []
    for i, mid in enumerate(mids):
        if mid is None:
            uppers.append(None)
            lowers.append(None)
            continue
        start = i - period + 1
        window = values[start : i + 1]
        mean = sum(window) / period
        var = sum((x - mean) ** 2 for x in window) / period
        std = var ** 0.5
        uppers.append(mid + k_std * std)
        lowers.append(mid - k_std * std)
    return uppers, mids, lowers


def atr(highs: List[float], lows: List[float], closes: List[float], period: int) -> List[Optional[float]]:
    out: List[Optional[float]] = []
    trs: List[float] = []
    prev_close: Optional[float] = None
    atr_val: Optional[float] = None
    for i, (h, l, c) in enumerate(zip(highs, lows, closes)):
        if prev_close is None:
            tr = h - l
        else:
            tr = max(h - l, abs(h - prev_close), abs(l - prev_close))
        prev_close = c
        trs.append(tr)
        if atr_val is None:
            if len(trs) >= period:
                atr_val = sum(trs[-period:]) / period
                out.append(atr_val)
            else:
                out.append(None)
        else:
            atr_val = (atr_val * (period - 1) + tr) / period
            out.append(atr_val)
    return out


def obv(closes: List[float], volumes: List[float]) -> List[float]:
    out: List[float] = []
    prev_close: Optional[float] = None
    current = 0.0
    for c, v in zip(closes, volumes):
        if prev_close is not None:
            if c > prev_close:
                current += v
            elif c < prev_close:
                current -= v
        out.append(current)
        prev_close = c
    return out


def pivot_points(highs: List[float], lows: List[float], closes: List[float]) -> List[Dict[str, float]]:
    out = []
    for h, l, c in zip(highs, lows, closes):
        pp = (h + l + c) / 3.0
        r1 = 2 * pp - l
        s1 = 2 * pp - h
        r2 = pp + (h - l)
        s2 = pp - (h - l)
        r3 = h + 2 * (pp - l)
        s3 = l - 2 * (h - pp)
        out.append({"pp": pp, "r1": r1, "r2": r2, "r3": r3, "s1": s1, "s2": s2, "s3": s3})
    return out


def macd(values: List[float], fast: int = 12, slow: int = 26, signal: int = 9) -> Tuple[List[Optional[float]], List[Optional[float]], List[Optional[float]]]:
    fast_ema = ema(values, fast)
    slow_ema = ema(values, slow)
    macd_line: List[Optional[float]] = []
    for f, s in zip(fast_ema, slow_ema):
        if f is not None and s is not None:
            macd_line.append(f - s)
        else:
            macd_line.append(None)
    # Feed only valid MACD values into signal EMA, then re-align with indices.
    valid_macd = [m for m in macd_line if m is not None]
    valid_signal = ema(valid_macd, signal)
    signal_line: List[Optional[float]] = []
    vi = 0
    for m in macd_line:
        if m is None:
            signal_line.append(None)
        else:
            signal_line.append(valid_signal[vi])
            vi += 1
    histogram: List[Optional[float]] = []
    for m, s in zip(macd_line, signal_line):
        if m is None or s is None:
            histogram.append(None)
        else:
            histogram.append(m - s)
    return macd_line, signal_line, histogram


def stochastic(highs: List[float], lows: List[float], closes: List[float], k_period: int = 14, d_period: int = 3) -> Tuple[List[Optional[float]], List[Optional[float]]]:
    k_vals: List[Optional[float]] = []
    d_vals: List[Optional[float]] = []
    for i in range(len(closes)):
        if i + 1 < k_period:
            k_vals.append(None)
            d_vals.append(None)
            continue
        window_high = max(highs[i + 1 - k_period : i + 1])
        window_low = min(lows[i + 1 - k_period : i + 1])
        denom = window_high - window_low
        k = 50.0 if denom == 0 else ((closes[i] - window_low) / denom) * 100.0
        k_vals.append(k)
        if len([x for x in k_vals if x is not None]) < d_period:
            d_vals.append(None)
        else:
            recent = [x for x in k_vals[-d_period:] if x is not None]
            d_vals.append(sum(recent) / d_period)
    return k_vals, d_vals


def volume_sma(volumes: List[float], period: int) -> List[Optional[float]]:
    return sma(volumes, period)


def generate_references(rows: List[Dict[str, float]]) -> None:
    closes = [r["close"] for r in rows]
    highs = [r["high"] for r in rows]
    lows = [r["low"] for r in rows]
    volumes = [r["volume"] for r in rows]

    # SMA
    for p in [5, 10, 20, 50, 100, 200]:
        write_json(f"sma_{p}.json", sma(closes, p))

    # EMA
    for p in [5, 10, 20, 50, 100, 200]:
        write_json(f"ema_{p}.json", ema(closes, p))

    # MACD 12/26/9
    macd_line, signal_line, hist = macd(closes)
    write_json("macd_line.json", macd_line)
    write_json("macd_signal.json", signal_line)
    write_json("macd_hist.json", hist)

    # RSI
    for p in [7, 14, 21]:
        write_json(f"rsi_{p}.json", rsi(closes, p))

    # Stochastic 14,3
    k_vals, d_vals = stochastic(highs, lows, closes)
    write_json("stoch_k.json", k_vals)
    write_json("stoch_d.json", d_vals)

    # Bollinger 20, 2
    upper, middle, lower = bollinger(closes, 20, 2.0)
    write_json("bb_upper.json", upper)
    write_json("bb_middle.json", middle)
    write_json("bb_lower.json", lower)

    # ATR 14
    write_json("atr_14.json", atr(highs, lows, closes, 14))

    # Volume SMA 20
    write_json("volume_sma_20.json", volume_sma(volumes, 20))

    # OBV
    write_json("obv.json", obv(closes, volumes))

    # Pivot Points
    write_json("pivot_points.json", pivot_points(highs, lows, closes))


def generate_all() -> None:
    ensure_dirs()
    rows = synthetic_candles()
    # Write synthetic data as stand-ins for the three datasets
    for fname in ["aapl_daily_2y.csv", "eurusd_1h_1y.csv", "spy_daily_5y.csv"]:
        write_csv(fname, rows)
    generate_references(rows)


if __name__ == "__main__":
    generate_all()
