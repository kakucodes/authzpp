use cosmwasm_std::{Addr, BlockInfo, StdResult, Timestamp};
use cw_grant_spec::grants::GrantSpec;

pub trait Expirable {
    fn is_expired(&self, block: &BlockInfo) -> bool;
    fn is_not_expired(&self, block: &BlockInfo) -> bool;
}

// pub trait AuthzppGrant {
//     type GrantSettings;
//     type ExecuteSettings;

//     // fn execute_without_broadcast(&self, execute_settings: Self::ExecuteSettings) -> StdResult<()>;
//     // fn revoke_grant(&self, grantee: &Addr, settings: GrantSettings) -> StdResult<()>;
//     // fn grant_structure(&self, grantee: &Addr, granter: &Addr) -> StdResult<Vec<GrantStructure<>>;

//     fn query_requisite_grants(&self, settings: GrantSettings) -> Vec<GrantSpec>;

//     // fn execute_grant(
//     //     &self,
//     //     execute_settings: Self::ExecuteSettings,
//     //     contract_addr: Addr,
//     // ) -> StdResult<()>;
// }

// pub trait Grantable<GrantSettings> {
//     // type GrantSettings;

//     fn query_grants(&self, grant: GrantStructure<GrantSettings>) -> Vec<GrantSpec>;
// }

pub struct GrantStructure<T> {
    pub granter: Addr,
    pub grantee: Addr,
    pub expiration: Timestamp,
    // pub grant_contract: Addr,
    pub grant_data: T,
}
