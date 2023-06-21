use crate::msg::AllowedDenomsSendSettings;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

// map from Granter & Grantee addresses to GrantSettings
pub const GRANTS: Map<(&Addr, &Addr), AllowedDenomsSendSettings> = Map::new("grants");
