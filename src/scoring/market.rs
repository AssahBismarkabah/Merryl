use crate::domain::models::{
    DailyPrice, IndustryScore, MarketRegimeScore, SectorMap, SectorScore, StockScore, Symbol,
};

use super::indicators::histories_by_symbol;
use super::industries::score_industries;
use super::regime::score_market_regime;
use super::sectors::score_sectors;
use super::stocks::score_stocks;

#[derive(Debug, Clone)]
pub struct MarketScores {
    pub regime: MarketRegimeScore,
    pub sectors: Vec<SectorScore>,
    pub industries: Vec<IndustryScore>,
    pub stocks: Vec<StockScore>,
}

pub fn score_market(
    date: &str,
    symbols: &[Symbol],
    prices: &[DailyPrice],
    sector_maps: &[SectorMap],
) -> MarketScores {
    let histories = histories_by_symbol(prices);
    let regime = score_market_regime(date, &histories);
    let sector_scores = score_sectors(date, symbols, &histories, sector_maps);
    let industry_scores = score_industries(date, symbols, &histories);
    let stock_scores = score_stocks(date, symbols, &histories, sector_maps, &sector_scores);

    MarketScores {
        regime,
        sectors: sector_scores,
        industries: industry_scores,
        stocks: stock_scores,
    }
}
