use cosmwasm_std::Addr;

// pub trait ExecutableGrant<E> {
//     fn execute_without_broadcast(&self, execute_settings: E) -> StdResult<()>;
//     fn revoke_grant(&self, grantee: &Addr) -> StdResult<()>;
//     // fn grant_structure(&self, grantee: &Addr, granter: &Addr) -> StdResult<Vec<GrantStructure<>>;
// }

// pub trait QueryableGrant<T> {
//     fn query_grant(&self, granter: &Addr, grantee: &Addr) -> Option<T>;
// }

pub struct GrantStructure<T> {
    pub granter: Addr,
    pub grantee: Addr,
    pub grant_contract: Addr,
    pub grant: T,
}
