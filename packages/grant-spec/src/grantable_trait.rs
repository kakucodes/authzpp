use crate::grants::GrantRequirement;
use cosmwasm_std::{Addr, StdResult, Timestamp};

// use crate::grants::GrantSpec;

// pub trait AuthzppGrantable {
//     type GrantSettings;
//     type ExecuteSettings;

//     fn get_grant_spec(
//         &self,
//         settings: &Self::GrantSettings,
//         contract_addr: &Addr,
//     ) -> Vec<GrantSpec>;
// }

pub trait Grantable {
    type GrantSettings;

    fn query_grants(grant: GrantStructure<Self::GrantSettings>)
        -> StdResult<Vec<GrantRequirement>>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrantStructure<T> {
    pub granter: Addr,
    pub grantee: Addr,
    pub expiration: Timestamp,
    pub grant_contract: Addr,
    pub grant_data: T,
}
