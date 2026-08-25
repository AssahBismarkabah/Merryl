# Daily Volume-Profile Classification Tab

Status: implementation slice; candidate classifier only  
Scope: active US stocks already stored in Merryl

## Purpose

Add a separate dashboard tab that reduces the active stock universe to daily
volume-profile structure candidates for manual chart review. It is not an
automatic trading signal and does not claim to reproduce an intraday TradingView
profile exactly.

## Frozen input assumptions

- Instrument: stocks.
- Chart timeframe: 1D.
- Intended holding period: approximately 2–4 weeks.
- Volume: total traded volume, not buy/sell delta.
- Universe: active symbols with `asset_type = 'stock'` and enough stored daily bars.
- Profile source: stored daily OHLCV. Each daily bar's volume is allocated across
  its high-low price range as a documented approximation.
- Output status: every result is `candidate_review`, never `buy` or `sell`.

## Classifier outputs

Each stock may receive one or more labels:

- `box_candidate`: recent bounded structure with repeated internal activity;
- `trend_candidate`: directional leg with a dominant volume node;
- `rejection_candidate`: range-expansion reversal with a return toward the
  event's high-volume area;
- `level_failure_candidate`: price crossed a previously identified node and is
  testing it from the opposite side;
- `no_candidate`: no current structure passed the coarse screen.

Every non-empty candidate carries the profile window, POC, node band, current
price, direction, and a human-review note. The tab is therefore a charting queue,
not an execution queue.

## Honest limitation

The classifier can propose structural anchors from daily bars, but it cannot
know with certainty which box/leg/rejection a discretionary trader would draw.
The UI must keep the distinction visible. A future faithful version requires
lower-timeframe data for the actual stock and a review workflow that records
accepted/rejected anchors.

## Acceptance criteria

1. The tab loads without requiring a new workflow run.
2. It evaluates all eligible active stocks, not only the existing watchlist.
3. It shows the latest stored daily date and the total-volume approximation.
4. It exposes the candidate label, direction, anchor window, POC/node band, and
   review status.
5. Empty or insufficient histories are reported as skipped counts, not errors.
6. No result is described as a validated edge or automatic trade.
