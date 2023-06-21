use crate::msg::AllowedDenomsSendSettings;
use crate::ContractError;
use authzpp_utils::helpers::Expirable;
use cosmwasm_std::{Addr, Api, BlockInfo, Coin};

pub fn validate_granter_address(api: &dyn Api, granter: &str) -> Result<Addr, ContractError> {
    api.addr_validate(granter)
        .map_err(|_| ContractError::InvalidGranterAddress(granter.to_string()))
}

pub fn validate_grantee_address(api: &dyn Api, grantee: &str) -> Result<Addr, ContractError> {
    api.addr_validate(grantee)
        .map_err(|_| ContractError::InvalidGranteeAddress(grantee.to_string()))
}

pub fn validate_receiver_address(api: &dyn Api, grantee: &str) -> Result<Addr, ContractError> {
    api.addr_validate(grantee)
        .map_err(|_| ContractError::InvalidReceiverAddress(grantee.to_string()))
}

impl Expirable for AllowedDenomsSendSettings {
    fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expiration > block.time
    }
    fn is_not_expired(&self, block: &BlockInfo) -> bool {
        self.expiration <= block.time
    }
}

/// check that the coins attempting to be sent are in the allowlist
pub fn denoms_allowed(allowed_denoms: &[String], to_send: &[Coin]) -> Result<(), ContractError> {
    // verify that each of the denoms that are being sent are in the allow list
    if to_send
        .iter()
        .all(|coin| allowed_denoms.contains(&coin.denom))
    {
        return Err(ContractError::UnauthorizedDenom {
            allowed_denoms: allowed_denoms.to_vec(),
            to_send: to_send.iter().map(|coin| coin.denom.to_string()).collect(),
        });
    }
    Ok(())
}
