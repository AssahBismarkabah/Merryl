use crate::domain::models::{
    DailyPrice, IndustryScore, SectorMap, SectorScore, StockScore, Symbol,
};

use super::indicators::histories_by_symbol;
use super::industries::score_industries;
use super::sectors::score_sectors;
use super::stocks::score_stocks;

pub fn score_market(
    date: &str,
    symbols: &[Symbol],
    prices: &[DailyPrice],
    sector_maps: &[SectorMap],
) -> (Vec<SectorScore>, Vec<IndustryScore>, Vec<StockScore>) {
    let histories = histories_by_symbol(prices);
    let sector_scores = score_sectors(date, symbols, &histories, sector_maps);
    let industry_scores = score_industries(date, symbols, &histories);
    let stock_scores = score_stocks(date, symbols, &histories, sector_maps, &sector_scores);

    (sector_scores, industry_scores, stock_scores)
}
