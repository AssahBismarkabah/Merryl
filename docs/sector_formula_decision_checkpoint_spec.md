# Sector Formula Decision Checkpoint

Version: 0.2
Date: 2026-05-27
Status: Complete; Option B2 accepted

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/sector_score_review_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/phase_3_backtest_validation_spec.md`

## 1. Purpose

This checkpoint exists because PDB-2 found a real sector formula issue, and PDB-3 is now complete.

The specific issue:

```text
SECTOR_RANK_CHANGE_WEIGHT existed in the sector score formula,
but rank_change was only known after sector ranks are calculated.
The score therefore used a neutral placeholder for rank_change.
```

This meant the formula looked like rank change was part of sector scoring, but it was not truly contributing to sector ranking.

Before continuing to PDB-4, we need to decide whether to leave this as-is, remove it, or build a real second-pass rank-change design.

## 2. Why This Must Be Controlled

Sector ranking is core to the original market map:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

Changing sector scoring can affect:

- Sector score values.
- Sector ranking.
- Stock scores, because stock scoring uses `sector_score`.
- Watchlist membership.
- Backtest results.
- Report interpretation.

Therefore the fix must be validated, not guessed.

## 3. Current Behavior

Current formula:

```text
sector_score =
  relative_return_weight * relative_return_component
+ trend_weight * trend_component
+ relative_volume_weight * relative_volume_component
+ breadth_weight * breadth_component
+ rank_change_weight * neutral_score
```

Current result:

```text
rank_change_weight adds a constant neutral contribution to every sector.
That constant does not improve sector ranking.
It does affect absolute sector score values.
Stock scoring consumes those absolute sector score values.
```

## 4. Candidate Decisions

### Option A: Keep Current Formula

Keep the current formula and keep sector ranking labeled as map-only.

Use this only if:

- Removing or redesigning rank change does not improve validation.
- The current absolute sector score scale is preferable for stock scoring.
- We explicitly document the neutral placeholder as intentional V1 behavior.

### Option B: Remove Neutral Rank-Change Component

Remove the rank-change placeholder from the score formula.

Two sub-options must be tested:

```text
B1: Remove rank_change weight and do not renormalize remaining weights.
B2: Remove rank_change weight and renormalize remaining sector weights to sum to 1.0.
```

Why this is the cleanest first candidate:

- It removes misleading formula semantics.
- It does not pretend rank change is predictive.
- It avoids a second-pass ranking design.

Risk:

- It can change absolute sector scores.
- It can change stock scores and watchlists.

### Option C: Real Second-Pass Rank-Change Score

Create a real rank-change component after initial sector ranks are known.

Possible design:

```text
1. Score sectors without rank change.
2. Rank sectors.
3. Compare current rank to previous scored date.
4. Convert rank improvement into a bounded component.
5. Recalculate final sector score.
6. Re-rank sectors.
```

Use this only if validation justifies the complexity.

Risks:

- More moving parts.
- More difficult to explain.
- Can over-reward short-term rank jumps.
- Requires careful future-leak prevention.

## 5. Validation Method

Validation should compare current behavior against candidate behavior using stored daily prices.

Do not add a public CLI command for this checkpoint.

Use existing workflow surfaces:

```text
merryl run daily --date latest
merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD
merryl status
```

Before changing formula code:

1. Save the current baseline backtest report/CSV references.
2. Record current counts from `merryl status`.
3. Record current sector, stock, and industry validation summaries.

After a candidate formula change:

1. Run `merryl run daily --date latest` to regenerate historical scores from stored/fetched daily prices.
2. Run the same backtest window as the baseline.
3. Compare sector decile behavior.
4. Compare stock decile behavior.
5. Compare industry validation behavior.
6. Compare watchlist stability.

## 6. Required Acceptance Checks

The candidate can be accepted only if all are true:

- Sector formula semantics are honest and explainable.
- Sector ranking remains a market-map layer unless validation clearly improves.
- No future prices are used.
- Stock score validation does not materially regress.
- Watchlist behavior remains understandable.
- Daily report wording remains honest.
- Backtest output remains framed as score behavior, not trade profitability.

## 7. Metrics To Compare

Compare baseline versus candidate:

```text
Sector decile 10 minus decile 1 average relative return:
1D, 5D, 10D, 20D, 60D

Sector component behavior:
relative volume
5D return
20D return
60D return
breadth
rank_change

Stock decile 10 minus decile 1 average relative return versus sector:
1D, 5D, 10D, 20D, 60D

Stock decile 10 hit rate:
1D, 5D, 10D, 20D, 60D

Watchlist churn:
top 25 overlap between baseline and candidate
```

## 8. Execution Result

Decision:

```text
Accept Option B2.
Remove the neutral rank-change contribution from sector scoring.
Renormalize the remaining sector score weights.
Keep rank_change stored and reported, but do not score it yet.
```

Code behavior after this decision:

```text
sector_score =
  (
    relative_return_weight * relative_return_component
  + trend_weight * trend_component
  + relative_volume_weight * relative_volume_component
  + breadth_weight * breadth_component
  ) / active_sector_weight_total
```

`rank_change` remains useful as a report field:

```text
It shows how the sector rank moved versus the previous scored date.
It is not treated as a predictive scoring component.
```

## 9. Validation Run

Baseline:

```text
cargo run -- run backtest --from 2025-07-01 --to 2026-05-27
backtest result id: 6
```

Candidate after Option B2:

```text
cargo run -- run daily --date latest
cargo run -- run backtest --from 2025-07-01 --to 2026-05-27
backtest result id: 7
```

Observation counts were unchanged:

```text
sector observations: 11484
sector component observations: 80388
stock observations: 52200
industry validation observations: 52200
```

Latest top-25 watchlist overlap:

```text
24 of 25 names stayed the same.
SWKS replaced CIEN.
```

## 10. Metric Comparison

Sector decile 10 minus decile 1 average relative return:

| Horizon | Baseline | Candidate | Change |
|---:|---:|---:|---:|
| 1D | -0.01% | -0.01% | 0.00% |
| 5D | -0.18% | -0.18% | 0.00% |
| 10D | 0.13% | 0.13% | 0.00% |
| 20D | 0.46% | 0.47% | 0.00% |
| 60D | -1.81% | -1.82% | 0.00% |

Stock decile 10 average relative return versus sector:

| Horizon | Baseline | Candidate | Change |
|---:|---:|---:|---:|
| 1D | 0.32% | 0.34% | 0.02% |
| 5D | 1.02% | 1.07% | 0.05% |
| 10D | 1.31% | 1.37% | 0.06% |
| 20D | 2.62% | 2.69% | 0.07% |
| 60D | 10.95% | 10.73% | -0.22% |

Stock decile 10 hit rate:

| Horizon | Baseline | Candidate | Change |
|---:|---:|---:|---:|
| 1D | 51.54% | 52.07% | 0.53% |
| 5D | 53.63% | 54.17% | 0.54% |
| 10D | 53.58% | 53.58% | 0.00% |
| 20D | 53.56% | 54.04% | 0.48% |
| 60D | 59.29% | 59.40% | 0.12% |

## 11. Interpretation

Option B2 should be kept.

Reasons:

- It fixes misleading formula semantics.
- Sector ranking behavior is effectively unchanged.
- The strongest stock decile does not materially regress.
- Latest watchlist churn is small.
- It avoids a second-pass rank-change design before evidence justifies that complexity.

Sector ranking still remains map-only:

```text
This cleanup makes the formula honest.
It does not prove sector score is a forward-return signal.
```

## 12. Rejected For Now

Option C remains deferred.

Reason:

```text
Rank change was weak in PDB-2 validation.
A second-pass scoring design is more complex and is not justified yet.
```

## 13. Historical Plan

Original recommended execution order:

1. Document this checkpoint and make it the next pre-dashboard item.
2. Capture baseline summary from the latest current backtest.
3. Implement Option B2 first in a small branch or small patch:
   - remove neutral rank-change contribution from score calculation
   - renormalize remaining weights
   - keep `rank_change` stored and reported
4. Run focused tests.
5. Run full tests.
6. Run daily workflow.
7. Run backtest over the same range.
8. Compare metrics and write a decision note.
9. Either keep the change or revert it before moving to PDB-4.

## 14. Initial Recommendation From Plan

Test Option B2 first.

Reason:

```text
It removes a misleading neutral component while preserving a normalized 0-100 score design.
```

Do not implement Option C yet.

Reason:

```text
PDB-2 showed rank_change was weak as a predictive component.
A second-pass design adds complexity before evidence justifies it.
```

## 15. Completion Check

This checkpoint is done when:

- A candidate sector formula decision is implemented or deliberately rejected. Complete.
- The same backtest window is rerun. Complete.
- The decision is documented. Complete.
- The backlog says whether sector formula cleanup is complete or deferred. Complete.
- PDB-4 is not started until this is closed. Complete.
