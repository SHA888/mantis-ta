# Backtest Rigor & Honest Reporting

> **Provenance.** This spec distills the `xbt` backtesting design (a proposed
> standalone research engine, since dropped) into concrete rigor upgrades for
> **mantis-ta's existing backtest engine** (shipped v0.4.0). This document is the
> single source of record: the original three xbt design docs were fully
> distilled into it (principles, scorecards, skew thesis, and deferred items
> below) and then removed.
>
> **Decision (2026-07-11).** mantis-ta is the ecosystem's *asset-agnostic*
> backtesting engine and already ships costs, next-bar execution, walk-forward,
> and scalar metrics. Rather than fork a second engine, its stronger
> lie-prevention *principles* are harvested here as roadmap items. The separate
> `xbt-*` crate family is **not** built. Downstream consumers (`signal-engine`,
> MANTIS platform) depend inward on mantis-ta and do not reimplement it.

## The problem a backtest actually solves

A backtest is not "run a strategy over data." It is a **claim about
counterfactual fills**: what you would have received, *net of costs*, *without
using information unavailable at decision time*. Every serious backtest defect
is one of three things:

1. **Lookahead leakage** — using bar *t*'s outcome to make bar *t*'s decision.
2. **Cost fantasy** — filling at mid with zero slippage/fees.
3. **Overfit** — tuning to one path, or gating regimes by hindsight.

The engineering goal is to make (1) and (2) *hard to commit by accident* and to
make (3) *visible in the output*, rather than catching them in review.

## What mantis-ta already does (v0.4.0)

- **Costs**: `BacktestConfig` carries `commission_per_trade`, `commission_pct`,
  `slippage_pct`; `BrokerSim` applies them on every fill.
- **Lookahead avoidance**: `ExecutionModel::NextBarOpen` (default) fills at the
  *next* bar's open; indicators see only data up to the current bar.
- **Walk-forward**: `WalkForwardResult` + `ParameterSensitivity`.
- **Overfit guards**: minimum-trade-count and excessive-condition warnings.
- **Metrics**: return, Sharpe, max drawdown, win rate, trade log.

## What this spec adds (the four harvested principles)

### A. Cost-fantasy tagging — zero cost is opt-in and labeled

Zero-cost (or near-zero) results must never be silently presentable as
realistic. Concretely:

- Tag every `BacktestResult` with the cost basis it was produced under (e.g. a
  `cost_model` label / enum; `"zero"` when all cost params are zero).
- Surface a warning or explicit marker when a run is effectively cost-free.
- **Property tests** (build-failing invariants): a fill is never more favorable
  than mid; slippage is never negative; commission is never a rebate unless a
  maker model is explicitly selected.

*Why:* the reference-bot failure this whole discipline exists to prevent is
reporting a mid-fill edge that a real spread would have eaten.

### B. Cost-reconciliation scorecard — realistic vs zero-cost gap

Run the **same** strategy twice — under the configured cost model and under
`ZeroCost` — and report the gap (return, expectancy, Sharpe, win rate).

> The gap is where "edge" was actually just unmodeled cost.

Emit it as a first-class section of the result, not a manual two-run
comparison. This is the survivorship-/cost-reporting defense in engine form.

### C. Return distributions — not a single scalar

Scalar Sharpe/return hides the shape that decides whether a strategy survives.
Add to `BacktestMetrics`:

- **skew** and **kurtosis** of the return series (mean/median already present);
- optional export of the full per-trade return distribution.

*Why:* mean-reversion (many small wins, rare large losses → negative skew) and
trend (many small losses, rare large wins → positive skew) look similar on
average return and opposite on shape. Exits live in the tails.

### D. Regime-conditional edge — as a lens

Let the backtest accept an **optional per-bar regime label** (supplied, or
derived from ADX — shipped v0.5.0 — plus a realized-vol percentile) and report
metrics **conditioned on regime**. Tests whether an edge is regime-dependent or
curve-fit to one regime.

- The *classifier* stays in `mantis-regime` (the ecosystem's ADX-based regime
  component); mantis-ta's backtest only **conditions metrics** on a supplied
  regime series. No new gating trait is required inside mantis-ta for this lens.

## Deliberately deferred (Chesterton's Fence — documented, not built)

- **Typestate lookahead** (`Bar<Decision>` exposing only `open`/`prior_close`,
  `Bar<Resolved>` exposing `close`/`high`/`low`). Elegant — makes the bar-close
  leak a *compile* error — but it is a breaking rewrite of a published v0.5+
  API that already prevents the leak via `NextBarOpen`. **Apply only at a v1.0
  API break**, not before.
- **Integer/newtype money** (no `f64` in money paths). mantis-ta is
  deliberately float-based as a general TA library (`*_per_share` is legitimately
  `f64`). Integer-money discipline belongs in downstream products
  (`signal-engine` already enforces it), not in this library.
- **Multi-sleeve portfolio / covariance sizing / skew-diversification.** Genuine
  new scope with no consumer yet — you need a *book of strategies* to diversify
  before covariance-aware sizing earns its complexity. Build only when that
  book exists, and as a thin layer that **consumes** mantis-ta, never inside it.

## Additional design ideas preserved

*(Folded in from the source docs so nothing is lost after `docs/xbt-source/`
is removed. Not immediate roadmap items — recorded so the rationale survives.)*

### The four scorecards

The engine was to grade output on four axes, not a single equity curve:

1. **Directional accuracy** — signal vs realized forward return. *(cheap add on
   the metrics layer; not yet a harvested principle above.)*
2. **Exit/pullback skew capture** — did exits capture the intended skew, or
   convert positive-skew trades to negative? *(the "why" behind principle C —
   distributions.)*
3. **Regime-conditional edge** — forward returns conditioned on the regime call.
   *(= principle D.)*
4. **Claimed-vs-actual reconciliation** — realistic-cost vs zero-cost P&L.
   *(= principle B.)*

Principles B–D operationalize scorecards 2–4; scorecard 1 is a small, separate
metrics addition.

### Skew-diversification thesis (why a multi-sleeve book, once one exists)

- Equity mean-reversion: many small wins + rare large losses → **negative skew**.
- BTC/commodity trend: many small losses + rare large wins → **positive skew**.
- Combined, they smooth the equity curve **only if** they are not stopped out in
  the same regime shift. Measure it: rolling pairwise correlation **and**
  stress-conditional correlation (worst-decile book-level days). The protective
  mechanism is a **gross-exposure cap that tightens as average pairwise
  correlation rises** — and the engine measures whether it holds out-of-sample
  rather than assuming it. Deferred (see above); recorded here for when a real
  strategy book exists.

### Position typestate

`Position<Flat> → Position<Open> → Position<Closed>` makes "close a flat
position" or "open an already-open one" unrepresentable. mantis-ta's backtest
tracks position state internally; `signal-engine`'s `se-tracker` already encodes
an equivalent state machine. Recorded as a modeling option, not a mantis-ta
change.

## Placement

These land as a **backtest-rigor minor** on the mantis-ta roadmap (see
`TODO.md`). Public-API additions (new metric fields, cost-model label) are
minor-version, gated by the standard DoD: `fmt --check`, `clippy -D warnings`,
tests + property tests, Rustdoc, and a `CHANGELOG.md` entry.
