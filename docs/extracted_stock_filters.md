# Stock Classification Filters

This document contains the complete set of stock classification filters and parameters extracted from the book "Investing with VP with Golden Ticket".

## 1. Master Screening Table (The "Golden Ticket" Filter)

The author provides a master table on page 102 of the book, which serves as the definitive guide for screening stocks by sector using the FINVIZ screener.

| Sector | Market Cap | Dividend Yield | P/E | P/B | EPS past 5 yrs | ROA | ROE | Debt/Equity | Net Profit Margin |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Basic Materials** | Over $2B | Over 2% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 0.7 | Positive |
| **Communication Services** | Over $2B | Over 2% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 1 | Positive |
| **Consumer Cyclical** | Over $2B | Over 1% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Consumer Defensive** | Over $2B | Over 2% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Energy** | Over $2B | Over 2% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Financial** | Over $2B | Over 2% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Healthcare** | Over $2B | Any | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Industrials** | Over $2B | Over 1% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Real Estate** | Over $2B | Over 3% | Any | Any | Positive | Positive or Over 5% | Over 10% | Any | Positive |
| **Technology** | Over $2B | Over 1% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Under 70% | Positive |
| **Utilities** | Over $2B | Over 3% | Under 20 | Under 3 | Positive | Positive or Over 5% | Over 10% | Any | Positive |

---

## 2. Sector Selection Strategy (Author's Recommendation)

While 65 stocks might pass the initial filters, the author recommends narrowing down the selection by excluding certain sectors based on risk and economic sensitivity.

### Step 1: Initial Reduction
*   **Exclude Financials:** Reduces basket by ~8 stocks.
*   **Exclude Technology:** Reduces basket by ~5 stocks.
*   *Resulting basket size: 52 stocks.*

### Step 2: Risk-Averse Reduction
*   **Exclude Real Estate:** (Often high debt).
*   **Exclude Consumer Cyclical:** (Very sensitive to economic cycles).
*   *Resulting basket size: 41 stocks.*

### Final Recommended Sectors to Focus On:
1.  Communication
2.  Consumer Defensive
3.  Energy
4.  Healthcare
5.  Industrials
6.  Utilities
7.  Basic Materials

---

## 3. Parameter Definitions & Threshold Logic

### Descriptive Tab Filters
*   **Market Cap:** Baseline is > $300M, but the author strongly prefers **Over $2 Billion** (Mid-cap and above) to ensure stability.
*   **Dividend Yield:** Generally **Over 2%** to combat inflation, with higher requirements (3%+) for cash-rich sectors like Utilities and Real Estate.

### Fundamental Tab Filters
*   **P/E Ratio:** Maximum **20** (can relax to 25 for specific high-growth sectors, but 20 is the rule).
*   **P/B Ratio:** Maximum **3** (or below the sector average).
*   **EPS Growth (Past 5 Years):** Must be **Positive** (> 0%).
*   **ROA (Return on Assets):** Must be **Positive**, preferably **Over 5%**.
*   **ROE (Return on Equity):** Minimum **10%**.
*   **Debt/Equity:** Generally **Under 0.7 (70%)**. The author avoids companies with negative debt/equity and is cautious of those with too little debt (< 10-20%) as they may not be growing efficiently.
*   **Net Profit Margin:** Must be **Positive** (> 0%).

---

## 4. Implementation Notes
*   **Source:** *Investing with VP with Golden Ticket*, pages 38-102 (Filter logic) and 190-193 (Recap and sector reduction).
*   **Tool:** All filters are designed for the **FINVIZ Stock Screener**.
*   **Timing:** The fundamental filters are the first step; the second step is using **Volume Profile** to pick the entry point.
