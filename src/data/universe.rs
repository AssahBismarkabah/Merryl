use std::collections::HashSet;

use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::config::universe::{
    ASSET_BROAD_ETF, ASSET_MACRO_ETF, ASSET_SECTOR_ETF, ASSET_STOCK, BROAD_ETFS, EXCHANGE_US,
    MACRO_ETFS, SP500_WIKIPEDIA_URL,
};
use crate::domain::models::{IndustryMap, Symbol};

use super::sector_map::sector_maps;

pub fn fetch_sp500_symbols(client: &Client) -> Result<Vec<Symbol>> {
    let html = client
        .get(SP500_WIKIPEDIA_URL)
        .send()
        .context("failed to fetch S&P 500 constituents from Wikipedia")?
        .error_for_status()
        .context("Wikipedia S&P 500 constituent request failed")?
        .text()
        .context("failed to read S&P 500 constituent response")?;

    parse_sp500_symbols(&html)
}

pub fn etf_symbols() -> Vec<Symbol> {
    let mut symbols: Vec<Symbol> = BROAD_ETFS
        .iter()
        .map(|(ticker, name)| symbol(ticker, name, ASSET_BROAD_ETF, None, None))
        .collect();

    symbols.extend(
        MACRO_ETFS
            .iter()
            .map(|(ticker, name)| symbol(ticker, name, ASSET_MACRO_ETF, None, None)),
    );

    for sector_map in sector_maps() {
        symbols.push(symbol(
            &sector_map.sector_etf,
            &sector_map.description,
            ASSET_SECTOR_ETF,
            Some(&sector_map.sector),
            None,
        ));
    }

    symbols
}

pub fn industry_maps(symbols: &[Symbol]) -> Vec<IndustryMap> {
    let mut seen = HashSet::new();
    let mut maps = Vec::new();

    for symbol in symbols
        .iter()
        .filter(|symbol| symbol.asset_type == ASSET_STOCK)
    {
        let (Some(sector), Some(industry)) = (&symbol.sector, &symbol.industry) else {
            continue;
        };
        if seen.insert((sector.clone(), industry.clone())) {
            maps.push(IndustryMap {
                industry: industry.clone(),
                sector: sector.clone(),
                description: format!("{industry} group within {sector}"),
            });
        }
    }

    maps.sort_by(|a, b| a.sector.cmp(&b.sector).then(a.industry.cmp(&b.industry)));
    maps
}

fn parse_sp500_symbols(html: &str) -> Result<Vec<Symbol>> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table#constituents tbody tr")
        .map_err(|err| anyhow::anyhow!("failed to create S&P 500 row selector: {err}"))?;
    let cell_selector = Selector::parse("td")
        .map_err(|err| anyhow::anyhow!("failed to create S&P 500 cell selector: {err}"))?;
    let mut symbols = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<String> = row
            .select(&cell_selector)
            .map(|cell| normalized_cell_text(&cell.text().collect::<Vec<_>>().join(" ")))
            .collect();
        if cells.len() < 4 {
            continue;
        }

        symbols.push(Symbol {
            symbol: cells[0].clone(),
            name: cells[1].clone(),
            asset_type: ASSET_STOCK.to_string(),
            sector: Some(normalize_sector(&cells[2])),
            industry: Some(cells[3].clone()),
            exchange: EXCHANGE_US.to_string(),
            market_cap: None,
            is_active: true,
        });
    }

    if symbols.is_empty() {
        bail!("could not parse S&P 500 constituents from Wikipedia");
    }

    Ok(symbols)
}

fn symbol(
    symbol: &str,
    name: &str,
    asset_type: &str,
    sector: Option<&str>,
    industry: Option<&str>,
) -> Symbol {
    Symbol {
        symbol: symbol.to_string(),
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        sector: sector.map(str::to_string),
        industry: industry.map(str::to_string),
        exchange: EXCHANGE_US.to_string(),
        market_cap: None,
        is_active: true,
    }
}

fn normalized_cell_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn normalize_sector(sector: &str) -> String {
    match sector {
        "Information Technology" => "Technology".to_string(),
        value => value.to_string(),
    }
}
