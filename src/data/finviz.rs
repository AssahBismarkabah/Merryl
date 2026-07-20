use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use std::time::Duration;

const SCREENER_URL: &str = "https://finviz.com/screener.ashx";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";

/// A single stock result from the Finviz screener.
///
/// Combines Overview (v=111) and Financial (v=161) view data for a
/// complete Golden Ticket field set. v=111 provides the descriptive fields
/// (ticker, company, sector, industry, market_cap, pe_ratio, price, change,
/// volume). v=161 supplements with financial metrics (dividend, roa, roe,
/// debt_equity, net_profit_margin).
#[derive(Debug, Clone, Default)]
pub struct ScreenerResult {
    pub ticker: String,
    pub company: String,
    pub sector: String,
    pub industry: String,
    pub market_cap: String,
    pub pe_ratio: String,
    pub price: String,
    pub change: String,
    pub volume: String,
    // Financial view supplements (v=161)
    pub dividend: String,
    pub roa: String,
    pub roe: String,
    pub debt_equity: String,
    pub net_profit_margin: String,
}

/// Financial view row — only the fields that v=161 uniquely provides.
/// price, change, volume, and market_cap are already supplied by v=111
/// (Overview) so they are not Repeated here.
#[derive(Debug, Clone, Default)]
pub struct FinancialResult {
    pub ticker: String,
    pub dividend: String,
    pub roa: String,
    pub roe: String,
    pub debt_equity: String,
    pub net_profit_margin: String,
}

/// Filter parameters for a sector, derived from the master screening table.
struct SectorFilter {
    /// Finviz sector code (e.g. "sec_technology"), empty for no sector filter.
    sector_code: &'static str,
    /// Dividend yield threshold: "o1", "o2", "o3", or "" for any.
    dividend: &'static str,
    /// Debt/equity threshold: "u0.7", "u1", or "" for any.
    debt_equity: &'static str,
    /// P/E threshold: "u20" for under 20, "" for any.
    pe: &'static str,
    /// P/B threshold: "u3" for under 3, "" for any.
    pb: &'static str,
    /// If true, the sector_code is ignored by Finviz (returns unfiltered results).
    /// We must filter results by the parsed sector column locally.
    needs_post_filter: bool,
}

/// The master screening table from the doc, mapped to Finviz filter codes.
///
/// Each sector's row defines the sector-specific overrides for dividend yield,
/// debt/equity, P/E, and P/B. Common filters (cap, EPS, ROA, ROE, net margin)
/// apply to all sectors.
///
/// Real Estate is the only sector that waives P/E ("Any") and P/B ("Any") per the doc.
fn sector_filters() -> Vec<SectorFilter> {
    vec![
        SectorFilter { sector_code: "sec_basicmaterials", dividend: "o2", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: false },
        // Note: sec_communication is ignored by Finviz (returns baseline). Post-filter by sector name.
        SectorFilter { sector_code: "sec_communication", dividend: "o2", debt_equity: "u1", pe: "u20", pb: "u3", needs_post_filter: true },
        // Note: sec_consumer_cyclical is ignored by Finviz. Post-filter by sector name.
        SectorFilter { sector_code: "sec_consumer_cyclical", dividend: "o1", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: true },
        // Note: sec_consumer_defensive is ignored by Finviz. Post-filter by sector name.
        SectorFilter { sector_code: "sec_consumer_defensive", dividend: "o2", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: true },
        SectorFilter { sector_code: "sec_energy", dividend: "o2", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: false },
        SectorFilter { sector_code: "sec_financial", dividend: "o2", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: false },
        SectorFilter { sector_code: "sec_healthcare", dividend: "", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: false },
        SectorFilter { sector_code: "sec_industrials", dividend: "o1", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: false },
        SectorFilter { sector_code: "sec_realestate", dividend: "o3", debt_equity: "", pe: "", pb: "", needs_post_filter: false },
        SectorFilter { sector_code: "sec_technology", dividend: "o1", debt_equity: "u0.7", pe: "u20", pb: "u3", needs_post_filter: false },
        SectorFilter { sector_code: "sec_utilities", dividend: "o3", debt_equity: "", pe: "u20", pb: "u3", needs_post_filter: false },
    ]
}

/// Build the Finviz screener URL for a given sector filter and view.
///
/// Uses the master screening table filters from the doc:
/// - Common (all sectors): Market Cap > $2B, EPS 5yr positive,
///   ROA positive, ROE > 10%, Net Profit Margin positive
/// - Sector-specific: dividend yield, debt/equity, P/E, P/B, sector code
///
/// ROA uses `fa_roa_pos` (positive) not `fa_roa_o5` (over 5%) because
/// the doc says "Positive or Over 5%" -- positive is the minimum.
///
/// `view` is the Finviz view number (e.g. 111 for Overview, 161 for Financial).
fn build_url(filter: &SectorFilter, view: u16) -> String {
    // Start with filters that apply to every sector unconditionally.
    let mut filters = String::from(
        "cap_midover,fa_eps5years_pos,fa_roa_pos,fa_roe_o10,fa_netmargin_pos",
    );

    // P/E (waived for Real Estate per doc: "Any")
    if !filter.pe.is_empty() {
        filters.push_str(",fa_pe_");
        filters.push_str(filter.pe);
    }
    // P/B (waived for Real Estate per doc: "Any")
    if !filter.pb.is_empty() {
        filters.push_str(",fa_pb_");
        filters.push_str(filter.pb);
    }
    // Dividend yield
    if !filter.dividend.is_empty() {
        filters.push_str(",fa_div_");
        filters.push_str(filter.dividend);
    }
    // Debt/equity
    if !filter.debt_equity.is_empty() {
        filters.push_str(",fa_debteq_");
        filters.push_str(filter.debt_equity);
    }
    // Sector code (skip if Finviz ignores this filter code)
    if !filter.needs_post_filter && !filter.sector_code.is_empty() {
        filters.push(',');
        filters.push_str(filter.sector_code);
    }

    format!("{SCREENER_URL}?v={view}&f={filters}&ft=4")
}

/// Create a reqwest blocking client with proper User-Agent and timeout.
pub fn new_client() -> Result<Client> {
    ClientBuilder::new()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .context("failed to create HTTP client")
}

/// Run the Finviz screener for a specific sector.
///
/// Fetches both the Overview (v=111) and Financial (v=161) views and merges
/// them to produce a complete ScreenerResult with all Golden Ticket fields.
/// v=111 provides: ticker, company, sector, industry, market_cap, pe_ratio, price,
/// change, volume. v=161 supplements: dividend, roa, roe, debt_equity,
/// net_profit_margin.
///
/// `sector` is the sector name (e.g. "Technology", "Healthcare").
/// Returns an error if the sector name is not recognized.
pub fn run_screener(client: &Client, sector: &str) -> Result<Vec<ScreenerResult>> {
    let code = sector_name_to_code(sector);
    let filters = sector_filters();
    let Some(filter) = filters.iter().find(|f| f.sector_code == code) else {
        return Ok(Vec::new());
    };

    // Fetch both views concurrently.
    let overview_url = build_url(filter, 111);
    let financial_url = build_url(filter, 161);

    let (mut overview_results, financial_results) = {
        let overview = fetch_screener_page(client, &overview_url)
            .with_context(|| format!("failed to fetch Finviz Overview for {sector}"))?;
        let financial = fetch_and_parse_financial_page(client, &financial_url)
            .with_context(|| format!("failed to fetch Finviz Financial for {sector}"))?;
        (overview, financial)
    };

    // Post-filter by sector name for sectors whose sec_xxx code is ignored by Finviz.
    if filter.needs_post_filter && !overview_results.is_empty() {
        overview_results.retain(|r| r.sector.eq_ignore_ascii_case(sector));
    }

    // Merge financial metrics into overview results.
    let financial_by_ticker: std::collections::HashMap<_, _> = financial_results
        .into_iter()
        .map(|fr| (fr.ticker.clone(), fr))
        .collect();

    let mut results = Vec::with_capacity(overview_results.len());
    for overview_row in overview_results {
        let ticker = overview_row.ticker.clone();
        let fin = financial_by_ticker.get(&ticker);

        results.push(ScreenerResult {
            ticker: overview_row.ticker,
            company: overview_row.company,
            sector: overview_row.sector,
            industry: overview_row.industry,
            market_cap: overview_row.market_cap,
            pe_ratio: overview_row.pe_ratio,
            price: overview_row.price,
            change: overview_row.change,
            volume: overview_row.volume,
            dividend: fin.map(|f| f.dividend.clone()).unwrap_or_default(),
            roa: fin.map(|f| f.roa.clone()).unwrap_or_default(),
            roe: fin.map(|f| f.roe.clone()).unwrap_or_default(),
            debt_equity: fin.map(|f| f.debt_equity.clone()).unwrap_or_default(),
            net_profit_margin: fin.map(|f| f.net_profit_margin.clone()).unwrap_or_default(),
        });
    }

    Ok(results)
}

fn sector_name_to_code(name: &str) -> &'static str {
    match name {
        "Basic Materials" => "sec_basicmaterials",
        "Communication Services" | "Communication" => "sec_communication",
        "Consumer Cyclical" => "sec_consumer_cyclical",
        "Consumer Defensive" => "sec_consumer_defensive",
        "Energy" => "sec_energy",
        "Financial" | "Financials" => "sec_financial",
        "Healthcare" | "Health Care" => "sec_healthcare",
        "Industrials" => "sec_industrials",
        "Real Estate" => "sec_realestate",
        "Technology" => "sec_technology",
        "Utilities" => "sec_utilities",
        _ => "",
    }
}

/// Fetch all pages of screener results from a starting URL.
///
/// Finviz paginates at 20 results per page. We iterate through all pages
/// by checking the pagination links in the HTML footer for the last page number,
/// then fetching each page with `&r=<offset>`.
fn fetch_screener_page(client: &Client, url: &str) -> Result<Vec<ScreenerResult>> {
    let mut all_results = Vec::new();

    // Fetch first page to get results and pagination info
    let html = fetch_url(client, url)?;
    let results = parse_screener_table(&html)?;
    let page_count = parse_page_count(&html);

    all_results.extend(results);

    // Fetch subsequent pages if any
    if page_count > 1 {
        for page in 2..=page_count {
            let offset = (page - 1) * 20 + 1;
            let page_url = if url.contains('?') {
                format!("{url}&r={offset}")
            } else {
                format!("{url}?r={offset}")
            };
            let html = fetch_url(client, &page_url)?;
            let results = parse_screener_table(&html)?;
            all_results.extend(results);
        }
    }

    Ok(all_results)
}

/// Fetch all pages of Financial (v=161) screener results and parse them.
fn fetch_and_parse_financial_page(client: &Client, url: &str) -> Result<Vec<FinancialResult>> {
    let mut all_results = Vec::new();

    let html = fetch_url(client, url)?;
    let results = parse_financial_screener_table(&html)?;
    let page_count = parse_page_count(&html);

    all_results.extend(results);

    if page_count > 1 {
        for page in 2..=page_count {
            let offset = (page - 1) * 20 + 1;
            let page_url = if url.contains('?') {
                format!("{url}&r={offset}")
            } else {
                format!("{url}?r={offset}")
            };
            let html = fetch_url(client, &page_url)?;
            let results = parse_financial_screener_table(&html)?;
            all_results.extend(results);
        }
    }

    Ok(all_results)
}

/// Parse the financial view screener table (v=161).
///
/// v=161 columns: No., Ticker, Market Cap, Dividend, ROA, ROE, ROIC,
/// Curr R, Quick R, LTDebt/Eq, Debt/Eq, Gross M, Oper M, Profit M,
/// Earnings, Price, Change, Volume
fn parse_financial_screener_table(html: &str) -> Result<Vec<FinancialResult>> {
    let document = Html::parse_document(html);

    let table_selector =
        Selector::parse("table.screener_table").map_err(|e| anyhow::anyhow!(
            "failed to create screener table selector: {e}"
        ))?;

    let Some(table) = document.select(&table_selector).next() else {
        return Ok(Vec::new());
    };

    let row_selector =
        Selector::parse("tr.styled-row").map_err(|e| anyhow::anyhow!(
            "failed to create row selector: {e}"
        ))?;

    let cell_selector = Selector::parse("td")
        .map_err(|e| anyhow::anyhow!("failed to create cell selector: {e}"))?;

    let tab_link_selector = Selector::parse("a.tab-link")
        .map_err(|e| anyhow::anyhow!("failed to create tab-link selector: {e}"))?;

    let mut results = Vec::new();

    for row in table.select(&row_selector) {
        let cells: Vec<String> = row
            .select(&cell_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // v=161 has 18 columns: No, Ticker, Market Cap, Dividend, ROA, ROE, ROIC,
        // Curr R, Quick R, LTDebt/Eq, Debt/Eq, Gross M, Oper M, Profit M,
        // Earnings, Price, Change, Volume
        // We only need columns 0-13; reduce threshold to 14 to accept partial rows.
        if cells.len() < 14 {
            continue;
        }

        let ticker = row
            .select(&tab_link_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| cells[1].clone());

        results.push(FinancialResult {
            ticker,
            dividend: cells[3].clone(),
            roa: cells[4].clone(),
            roe: cells[5].clone(),
            debt_equity: cells[10].clone(), // Debt/Eq (total debt/equity)
            net_profit_margin: cells[13].clone(), // Profit M
        });
    }

    Ok(results)
}

fn fetch_url(client: &Client, url: &str) -> Result<String> {
    client
        .get(url)
        .send()
        .context("failed to fetch Finviz screener page")?
        .error_for_status()
        .context("Finviz screener request failed")?
        .text()
        .context("failed to read Finviz screener response")
}

/// Parse the total page count from the screener pagination footer.
/// Returns 1 if no pagination found.
fn parse_page_count(html: &str) -> usize {
    // Find `screener_pagination` section and extract all `r=` offsets
    let Some(idx) = html.find("screener_pagination") else {
        return 1;
    };
    let section = &html[idx..(idx + 1500).min(html.len())];

    let mut max_offset = 0usize;
    let mut search_start = 0;
    while let Some(rpos) = section[search_start..].find("r=") {
        let num_start = search_start + rpos + 2;
        // Read digits after "r="
        let num_end = num_start
            + section[num_start..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count();
        if num_end > num_start {
            if let Ok(offset) = section[num_start..num_end].parse::<usize>() {
                if offset > max_offset {
                    max_offset = offset;
                }
            }
        }
        search_start = num_end.max(search_start + 1);
    }

    if max_offset == 0 {
        return 1;
    }
    // Convert max offset to page count (20 per page)
    (max_offset + 19) / 20
}

pub(crate) fn parse_screener_table(html: &str) -> Result<Vec<ScreenerResult>> {
    let document = Html::parse_document(html);

    let table_selector = Selector::parse("table.screener_table")
        .map_err(|e| anyhow::anyhow!("failed to create screener table selector: {e}"))?;

    let Some(table) = document.select(&table_selector).next() else {
        return Ok(Vec::new());
    };

    let row_selector = Selector::parse("tr.styled-row")
        .map_err(|e| anyhow::anyhow!("failed to create row selector: {e}"))?;

    let cell_selector = Selector::parse("td")
        .map_err(|e| anyhow::anyhow!("failed to create cell selector: {e}"))?;

    // Selector for the ticker link specifically, avoiding the grade-letter span
    // inside the company-ticker link. The ticker is in a separate <a class="tab-link">.
    let tab_link_selector = Selector::parse("a.tab-link")
        .map_err(|e| anyhow::anyhow!("failed to create tab-link selector: {e}"))?;

    let mut results = Vec::new();

    for row in table.select(&row_selector) {
        let cells: Vec<String> = row
            .select(&cell_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // Expect 11 cells: No., Ticker (maybe with grade prefix), Company,
        // Sector, Industry, Country, Market Cap, P/E, Price, Change, Volume
        if cells.len() < 11 {
            continue;
        }

        // The ticker in cell[1] is in <a class="tab-link"> (separate from the grade letter
        // in <a class="company-ticker"><span>A</span></a>). Extract it specifically.
        let ticker = row
            .select(&tab_link_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| cells[1].clone());

        results.push(ScreenerResult {
            ticker,
            company: cells[2].clone(),
            sector: cells[3].clone(),
            industry: cells[4].clone(),
            market_cap: cells[6].clone(),
            pe_ratio: cells[7].clone(),
            price: cells[8].clone(),
            change: cells[9].clone(),
            volume: cells[10].clone(),
            ..Default::default()
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_real_finviz_html() {
        // Pre-fetch with: curl -sL '...' > /tmp/finviz_debug.html
        let html = fs::read_to_string("/tmp/finviz_debug.html")
            .expect("fetch a Finviz screener page first");
        let results = parse_screener_table(&html).unwrap();
        assert!(!results.is_empty(), "expected at least one result");
    }

    #[test]
    fn test_sector_name_to_code() {
        assert_eq!(sector_name_to_code("Technology"), "sec_technology");
        assert_eq!(sector_name_to_code("Healthcare"), "sec_healthcare");
        assert_eq!(sector_name_to_code("Health Care"), "sec_healthcare");
        assert_eq!(sector_name_to_code("Financial"), "sec_financial");
        assert_eq!(sector_name_to_code("Unknown"), "");
    }

    #[test]
    fn test_build_url_technology_has_all_filters() {
        let filter = SectorFilter {
            sector_code: "sec_technology",
            dividend: "o1",
            debt_equity: "u0.7",
            pe: "u20",
            pb: "u3",
            needs_post_filter: false,
        };
        let url = build_url(&filter, 111);
        assert!(url.contains("cap_midover"));
        assert!(url.contains("fa_pe_u20"));
        assert!(url.contains("fa_pb_u3"));
        assert!(url.contains("fa_roe_o10"));
        assert!(url.contains("fa_netmargin_pos"));
        assert!(url.contains("fa_eps5years_pos"));
        assert!(url.contains("fa_roa_pos"));
        assert!(!url.contains("fa_roa_o5"));
        assert!(url.contains("fa_div_o1"));
        assert!(url.contains("fa_debteq_u0.7"));
        assert!(url.contains("sec_technology"));
        assert!(url.contains("v=111"));
    }

    #[test]
    fn test_build_url_financial_view() {
        let filter = SectorFilter {
            sector_code: "sec_technology",
            dividend: "o1",
            debt_equity: "u0.7",
            pe: "u20",
            pb: "u3",
            needs_post_filter: false,
        };
        let url = build_url(&filter, 161);
        assert!(url.contains("v=161"));
        assert!(url.contains("cap_midover"));
        assert!(url.contains("sec_technology"));
    }

    #[test]
    fn test_build_url_healthcare_no_dividend() {
        let filter = SectorFilter {
            sector_code: "sec_healthcare",
            dividend: "",
            debt_equity: "u0.7",
            pe: "u20",
            pb: "u3",
            needs_post_filter: false,
        };
        let url = build_url(&filter, 111);
        assert!(!url.contains("fa_div_"));
        assert!(url.contains("fa_debteq_u0.7"));
        assert!(url.contains("fa_pe_u20"));
        assert!(url.contains("fa_pb_u3"));
    }

    #[test]
    fn test_build_url_realestate_any_pe_pb_and_no_debt_equity() {
        let filter = SectorFilter {
            sector_code: "sec_realestate",
            dividend: "o3",
            debt_equity: "",
            pe: "",
            pb: "",
            needs_post_filter: false,
        };
        let url = build_url(&filter, 111);
        assert!(url.contains("fa_div_o3"));
        assert!(url.contains("cap_midover"));
        assert!(!url.contains("fa_debteq_"));
        assert!(!url.contains("fa_pe_"), "Real Estate should have no P/E filter");
        assert!(!url.contains("fa_pb_"), "Real Estate should have no P/B filter");
        assert!(url.contains("sec_realestate"));
    }

    #[test]
    fn test_build_url_post_filter_sector_omits_sector_code() {
        let filter = SectorFilter {
            sector_code: "sec_communication",
            dividend: "o2",
            debt_equity: "u1",
            pe: "u20",
            pb: "u3",
            needs_post_filter: true,
        };
        let url = build_url(&filter, 111);
        // Should NOT include the sector_code since Finviz ignores it
        assert!(!url.contains("sec_communication"), "post-filter sector should omit sector_code in URL");
        // But should include fundamental filters
        assert!(url.contains("fa_div_o2"));
        assert!(url.contains("fa_debteq_u1"));
    }

    #[test]
    fn test_parse_page_count_no_pagination() {
        assert_eq!(parse_page_count("<html><body>no pagination</body></html>"), 1);
    }

    #[test]
    fn test_parse_page_count_with_pagination() {
        let html = r#"<div class="screener_pagination">
            <a href="screener?v=111&f=abc&ft=4" class="screener-pages is-selected"><b>1</b></a>
            <a href="screener?v=111&f=abc&ft=4&r=21" class="screener-pages">2</a>
            <a href="screener?v=111&f=abc&ft=4&r=41" class="screener-pages">3</a>
        </div>"#;
        assert_eq!(parse_page_count(html), 3);

        // Single page
        let html2 = r#"<div class="screener_pagination">
            <a href="screener?v=111&f=abc&ft=4" class="screener-pages is-selected"><b>1</b></a>
        </div>"#;
        assert_eq!(parse_page_count(html2), 1);
    }

    #[test]
    fn test_parse_screener_table_empty() {
        let results = parse_screener_table("<html><body></body></html>").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_screener_table_no_table() {
        let results = parse_screener_table("<html><body><p>no table</p></body></html>").unwrap();
        assert!(results.is_empty());
    }
}
