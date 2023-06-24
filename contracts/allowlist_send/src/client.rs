use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, QuerierWrapper, StdResult, WasmMsg};

use crate::msg::{
    ActiveGrantsResponse, AllowlistSendSettings, ExecuteMsg, ExecuteSettings, QueryMsg,
};

pub struct AllowlistSendClient<'a> {
    /// The allowlist_send contract address
    authzpp_contract_addr: &'a Addr,
    /// The address of the user that created the grant
    granter_addr: &'a Addr,
    /// The address of the contract using this client and will be executing the grant
    grantee_addr: &'a Addr,
}

impl<'a> AllowlistSendClient<'a> {
    /// Creates a new client for the allowlist_send contract
    pub fn new(
        authzpp_contract_addr: &'a Addr,
        granter_addr: &'a Addr,
        contract_addr: &'a Addr,
    ) -> Self {
        Self {
            authzpp_contract_addr,
            granter_addr,
            grantee_addr: contract_addr,
        }
    }

    /// Queries the contract for the active grant for the delegator, if any
    pub fn active_grants_by_granter(
        &self,
        querier: QuerierWrapper,
    ) -> StdResult<ActiveGrantsResponse> {
        querier.query_wasm_smart(
            self.authzpp_contract_addr.to_string(),
            &QueryMsg::ActiveGrantsByGranter(self.granter_addr.to_string()),
        )
    }

    /// queries the list of all active grants
    pub fn active_grants_for_grantee(
        &self,
        querier: QuerierWrapper,
    ) -> StdResult<ActiveGrantsResponse> {
        querier.query_wasm_smart(
            self.authzpp_contract_addr.to_string(),
            &QueryMsg::ActiveGrantsByGrantee(self.grantee_addr.to_string()),
        )
    }

    /// Checks if the given receiver is allowed to receive funds from the granter
    pub fn active_grant(
        &self,
        querier: QuerierWrapper,
        receiver: &Addr,
    ) -> StdResult<Option<AllowlistSendSettings>> {
        let grant: StdResult<Option<AllowlistSendSettings>> = querier.query_wasm_smart(
            self.authzpp_contract_addr.to_string(),
            &QueryMsg::Grant {
                granter: self.granter_addr.to_string(),
                receiver: receiver.to_string(),
            },
        );

        grant.map(|res| {
            res.filter(|AllowlistSendSettings { grantee, .. }| *self.grantee_addr == *grantee)
        })
    }

    /// Generates the execute message to send the funds from the granter to the receiver
    pub fn execute_send(&self, receiver: &Addr, amount: Vec<Coin>) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.authzpp_contract_addr.to_string(),
            msg: to_binary(&ExecuteMsg::Execute(ExecuteSettings {
                receiver: receiver.to_string(),
                granter: self.granter_addr.to_string(),
                grantee: self.grantee_addr.to_string(),
                amount,
            }))?,
            funds: vec![],
        }))
    }
}
