use cosmwasm_std::Addr;

use crate::grants::GrantSpec;

pub trait AuthzppGrantable {
    type GrantSettings;
    type ExecuteSettings;

    fn get_grant_spec(
        &self,
        settings: &Self::GrantSettings,
        contract_addr: &Addr,
    ) -> Vec<GrantSpec>;
}
