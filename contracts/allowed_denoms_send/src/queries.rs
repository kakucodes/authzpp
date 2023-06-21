use crate::{
    msg::{ActiveGrantsResponse, AllowedDenomsSendSettings, VersionResponse},
    state::GRANTS,
};
use authzpp_utils::helpers::Expirable;
use cosmwasm_std::{Addr, BlockInfo, Order, Storage};

use crate::ContractError;

pub fn query_version() -> VersionResponse {
    VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// search for and return the grant settings for an abitrary granter
pub fn query_active_grant(
    storage: &dyn Storage,
    block: &BlockInfo,
    granter_addr: &Addr,
    grantee_addr: &Addr,
) -> Result<AllowedDenomsSendSettings, ContractError> {
    // get the grant for the delegator from state
    let grant_settings = GRANTS.load(storage, (granter_addr, grantee_addr))?;

    match grant_settings {
        // check that the grant is not expired and that the grantee is correct
        grant if grant.grantee.eq(grantee_addr) && block.time <= grant.expiration => Ok(grant),
        _ => Err(ContractError::NoActiveGrant {
            granter: granter_addr.to_string(),
            grantee: grantee_addr.to_string(),
        }),
    }
}

/// returns all of the grant settings that are for grants to the given grantee
pub fn query_active_grants_by_grantee(
    storage: &dyn Storage,
    block: &BlockInfo,
    grantee: &Addr,
) -> ActiveGrantsResponse {
    GRANTS
        // grab all of the grants
        .range(storage, None, None, Order::Ascending)
        // filter out the grants that are not for the requested grantee
        .filter_map(|item| {
            if let Ok((_, grant)) = item {
                // also ensure that the grant is active and unexpired
                if grant.is_not_expired(block) && grant.grantee.eq(grantee) {
                    return Some(grant);
                }
            }
            None
        })
        .collect::<Vec<_>>()
}
