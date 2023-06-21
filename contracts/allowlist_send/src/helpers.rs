use crate::msg::AllowlistSendSettings;
use crate::ContractError;
use authzpp_utils::helpers::Expirable;
use cosmwasm_std::{Addr, Api, BlockInfo};

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

impl Expirable for AllowlistSendSettings {
    fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expiration > block.time
    }
    fn is_not_expired(&self, block: &BlockInfo) -> bool {
        self.expiration <= block.time
    }
}
