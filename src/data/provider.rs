use anyhow::Result;
use chrono::NaiveDate;

use crate::domain::models::{
    DailyPrice, IndustryMap, IntradayPrice, MacroObservation, MarketEvent, SectorMap, Symbol,
};

pub trait DailyOhlcvProvider {
    fn symbols(&self) -> Result<Vec<Symbol>>;
    fn sector_maps(&self) -> Vec<SectorMap>;
    fn industry_maps(&self, symbols: &[Symbol]) -> Vec<IndustryMap>;
    fn daily_prices(&self, symbols: &[Symbol], end_date: NaiveDate) -> Result<Vec<DailyPrice>>;
}

pub trait IntradayOhlcvProvider {
    fn intraday_prices(
        &self,
        symbols: &[String],
        date: NaiveDate,
        timeframe: &str,
    ) -> Result<Vec<IntradayPrice>>;
}

pub trait CatalystEventProvider {
    fn recent_news_events(
        &self,
        symbols: &[String],
        end_date: NaiveDate,
    ) -> Result<Vec<MarketEvent>>;
}

pub trait EarningsCalendarProvider {
    fn upcoming_earnings_events(&self, symbols: &[String]) -> Result<Vec<MarketEvent>>;
}

pub trait FilingEventProvider {
    fn recent_filing_events(
        &self,
        symbols: &[String],
        end_date: NaiveDate,
    ) -> Result<Vec<MarketEvent>>;
}

pub trait MacroSeriesProvider {
    fn macro_observations(&self, end_date: NaiveDate) -> Result<Vec<MacroObservation>>;
}
