# Plans

Last updated: 2026-07-11

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
| 6.4 | Keltner Channels (`KeltnerOutput`) | Satisfies SPEC §4.2 checklist | - | cc:done [fbba1b5] |
| 6.5 | VWAP — Volume Weighted Average Price (`f64`) | Satisfies SPEC §4.2 checklist | - | cc:done [74e3fa9] |
| 6.6 | Accumulation/Distribution Line (`f64`) | Satisfies SPEC §4.2 checklist | - | cc:done [4c1ccfd] |
| 6.7 | Donchian Channels (`DonchianOutput`) | Satisfies SPEC §4.2 checklist | - | cc:done [b691cad] |
| 6.6.1 | Fix A/D Line TA-Lib parity fixture — regenerate `ad.json` against a non-degenerate dataset | Money-flow-multiplier is non-trivial (`\|MFM\|` not ~0) across the reference series so the fixture can actually discriminate a correct AD implementation from a broken one (sign flip, swapped high/low, missing division); regenerated via real `talib.AD` per `generate_ad_reference.py`'s existing design; `cargo test --all-features` passes | 6.6 | cc:done [6f2d270] |
| 6.7.1 | Extract shared rolling highest-high/lowest-low window helper | The min/max-over-window scan hand-duplicated across `Ichimoku`, `Stochastic`, and `Donchian` is consolidated into one helper (e.g. in `src/utils/`); all three call sites use it; `cargo test --all-features` passes with no behavior change | 6.7 | cc:TODO |
| 6.8 | `IndicatorRef` convenience constructors for all Batch B indicators | Each of 6.1-6.7 has a matching `IndicatorRef` constructor; `cargo test` passes | 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7 | cc:TODO |
| 6.9 | Verify Batch B indicators work in strategy builder → eval → signal flow | Integration test exercises each new indicator through builder → eval → signal for at least one condition; `cargo test --all-features` passes | 6.8 | cc:TODO |

## Phase BR: Backtest Rigor & Honest Reporting (xbt principles)

> **Proposed minor — version placement at maintainer's discretion** (slots as a
> minor after v0.6.0; may precede or interleave with the indicator batches).
> Honest-reporting upgrades to the **existing v0.4.0 backtest engine** — not a new
> engine. Full rationale, the four scorecards, the skew-diversification thesis, and
> the deferred items live in
> [`docs/research-backtest-rigor.md`](docs/research-backtest-rigor.md) (the
> distilled source of record).
>
> **Scorecard coverage:** doc scorecard 1 → BR.E (directional accuracy), 2 → BR.C
> (exit/skew via distributions), 3 → BR.D (regime-conditional edge), 4 → BR.B (cost
> reconciliation); BR.A is the enabling mechanism (cost-fantasy tagging) behind
> scorecard 4.
>
> **Per-item DoD:** `cargo fmt --check` clean, `cargo clippy -- -D warnings` clean,
> unit + property tests green, Rustdoc on new public items, `CHANGELOG.md` entry
> (public-API additions are minor-version).
>
> **Deferred (documented, not built now — see the doc):** `Bar<Decision>`/`Bar<Resolved>`
> typestate lookahead (apply only at a v1.0 API break), integer/newtype money (stays
> float-based — a general TA lib), multi-sleeve portfolio / covariance sizing (YAGNI
> until a real strategy book exists; would layer *above* mantis-ta).

| Task | Description | DoD | Depends | Status |
|------|--------------|-----|---------|--------|
| BR.A | Cost-fantasy tagging — zero cost is opt-in and labeled `[tdd:required]` | Every `BacktestResult` tagged with the cost basis (`cost_model` label; `"zero"` when all cost params are zero); explicit warn/mark on effectively cost-free runs; `proptest`: fill never more favorable than mid, slippage never negative, commission never a rebate unless a maker model is explicitly selected | - | cc:TODO |
| BR.B | Cost-reconciliation scorecard — realistic vs zero-cost gap `[tdd:required]` | Same strategy run under the configured cost model AND `ZeroCost`; gap (return, expectancy, Sharpe, win rate) reported as a first-class result section; golden test: gap non-zero under non-trivial costs and exactly zero under `ZeroCost` | BR.A | cc:TODO |
| BR.C | Return distributions — not a single scalar `[tdd:required]` | skew + kurtosis of the return series added to `BacktestMetrics`; optional export of the full per-trade return distribution; test: known synthetic return series reproduces textbook skew/kurtosis | - | cc:TODO |
| BR.D | Regime-conditional edge — as a lens `[tdd:required]` | `BacktestConfig` accepts an optional per-bar regime label (supplied, or derived from ADX + realized-vol percentile); metrics conditioned on regime (edge per regime); classifier stays in `mantis-regime`, backtest only *conditions on* the supplied series | - | cc:TODO |
| BR.E | Directional accuracy — metrics-layer add `[tdd:required]` | directional-accuracy metric on `BacktestMetrics` (signal vs realized forward-return sign agreement / hit rate), reported independently of P&L; test: synthetic series with known forward returns yields the expected accuracy | - | cc:TODO |

## Recently Completed

### v0.5.3 — Rust Upgrade
- edition 2024, MSRV 1.85 → 1.88 (cargo-tarpaulin compat), CI toolchain pin update, clippy/rustfmt fixes

### v0.5.0 — Tier 2 Indicators: Batch A (8 indicators)
- ADX, WMA, DEMA, TEMA, CCI, Williams %R, ROC, Standard Deviation — all TA-Lib verified, benched, and wired into strategy builder
