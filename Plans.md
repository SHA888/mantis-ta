# Plans

Last updated: 2026-07-02

Source of truth for scope/checklist detail: [TODO.md](./TODO.md).
Product contract for indicator correctness: [SPEC.md](./SPEC.md) §4.2 Indicator Implementation Checklist.

## Phase v0.6.0: Tier 2 Indicators — Batch B (7 indicators)

> Completes Tier 2. ADX from v0.5.0 now enables `mantis-regime` to
> implement rule-based regime detection (ADX > 25 = trending, etc.).
> Every indicator task's DoD is SPEC §4.2's 10-point checklist: TA-Lib
> parity < 1e-10, streaming `next()`, batch `calculate()`, `warmup_period()`,
> `reset()`, no panics, zero allocation in `next()`, Rustdoc example,
> unit + TA-Lib verification tests, streaming + batch Criterion benchmarks.

| Task | Description | DoD | Depends | Status |
|------|--------------|-----|---------|--------|
| 6.1 | Ichimoku Cloud (`IchimokuOutput`) | Satisfies SPEC §4.2 checklist (see phase note) | - | cc:done [49ed9b1] |
| 6.2 | Parabolic SAR (`f64`) | Satisfies SPEC §4.2 checklist | - | cc:done [58fd6ca] |
| 6.3 | MFI — Money Flow Index (`f64`) | Satisfies SPEC §4.2 checklist | - | cc:done [519c647] |
| 6.4 | Keltner Channels (`KeltnerOutput`) | Satisfies SPEC §4.2 checklist | - | cc:TODO |
| 6.5 | VWAP — Volume Weighted Average Price (`f64`) | Satisfies SPEC §4.2 checklist | - | cc:TODO |
| 6.6 | Accumulation/Distribution Line (`f64`) | Satisfies SPEC §4.2 checklist | - | cc:TODO |
| 6.7 | Donchian Channels (`DonchianOutput`) | Satisfies SPEC §4.2 checklist | - | cc:TODO |
| 6.8 | `IndicatorRef` convenience constructors for all Batch B indicators | Each of 6.1-6.7 has a matching `IndicatorRef` constructor; `cargo test` passes | 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7 | cc:TODO |
| 6.9 | Verify Batch B indicators work in strategy builder → eval → signal flow | Integration test exercises each new indicator through builder → eval → signal for at least one condition; `cargo test --all-features` passes | 6.8 | cc:TODO |

## Recently Completed

### v0.5.3 — Rust Upgrade
- edition 2024, MSRV 1.85 → 1.88 (cargo-tarpaulin compat), CI toolchain pin update, clippy/rustfmt fixes

### v0.5.0 — Tier 2 Indicators: Batch A (8 indicators)
- ADX, WMA, DEMA, TEMA, CCI, Williams %R, ROC, Standard Deviation — all TA-Lib verified, benched, and wired into strategy builder
