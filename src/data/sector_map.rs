use crate::config::universe::SECTOR_ETFS;
use crate::domain::models::SectorMap;

pub fn sector_maps() -> Vec<SectorMap> {
    SECTOR_ETFS
        .iter()
        .map(|(sector, etf)| SectorMap {
            sector: (*sector).to_string(),
            sector_etf: (*etf).to_string(),
            description: format!("{sector} sector ETF proxy"),
        })
        .collect()
}
