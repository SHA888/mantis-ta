# Plans

最終更新: 2026-07-02

Source of truth for scope/checklist detail: [TODO.md](./TODO.md).
This file tracks only the active phase + last completed phase to avoid drift; see `docs/plans-maintenance.md` convention.

## Active Phase

### v0.6.0 — Tier 2 Indicators: Batch B (7 indicators)

> Completes Tier 2. ADX from v0.5.0 now enables `mantis-regime` to
> implement rule-based regime detection (ADX > 25 = trending, etc.).

- [ ] Ichimoku Cloud — `IchimokuOutput`
- [ ] Parabolic SAR — `f64`
- [ ] MFI (Money Flow Index) — `f64`
- [ ] Keltner Channels — `KeltnerOutput`
- [ ] VWAP (Volume Weighted Average Price) — `f64`
- [ ] Accumulation/Distribution Line — `f64`
- [ ] Donchian Channels — `DonchianOutput`
- [ ] TA-Lib reference JSONs + verification tests + unit tests + benchmarks for all Batch B indicators
- [ ] `IndicatorRef` convenience constructors for new indicators
- [ ] Verify new indicators work in strategy builder → eval → signal flow

## Recently Completed

### v0.5.3 — Rust Upgrade
- edition 2024, MSRV 1.85 → 1.88 (cargo-tarpaulin compat), CI toolchain pin update, clippy/rustfmt fixes

### v0.5.0 — Tier 2 Indicators: Batch A (8 indicators)
- ADX, WMA, DEMA, TEMA, CCI, Williams %R, ROC, Standard Deviation — all TA-Lib verified, benched, and wired into strategy builder
