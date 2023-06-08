use crate::msg::AllowedWithdrawlSettings;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const GRANTS: Map<Addr, AllowedWithdrawlSettings> = Map::new("grants");
