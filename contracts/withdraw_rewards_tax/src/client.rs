use cosmwasm_std::{to_binary, Addr, CosmosMsg, Decimal, QuerierWrapper, StdResult, WasmMsg};

use crate::{
    msg::{ExecuteMsg, ExecuteSettings, GrantQueryResponse, QueryMsg, SimulateExecuteResponse},
    ContractError,
};

pub struct WithdrawRewardsTaxClient<'a> {
    /// Address of the grant contract
    authzpp_contract_addr: &'a Addr,

    /// Address of the delegator/granter
    delegator_addr: &'a Addr,
}

impl<'a> WithdrawRewardsTaxClient<'a> {
    /// Creates a new client for the withdraw_rewards_tax contract
    pub fn new(authzpp_contract_addr: &'a Addr, delegator_addr: &'a Addr) -> Self {
        Self {
            authzpp_contract_addr,
            delegator_addr,
        }
    }

    /// Queries the contract for a simulation of the grant execution for the given delegator.
    /// Returns both the amount expected to go to the delegator and the taxation address
    pub fn simulate(
        &self,
        querier: QuerierWrapper,
        percentage: Option<Decimal>,
    ) -> StdResult<SimulateExecuteResponse> {
        let simulation: StdResult<SimulateExecuteResponse> = querier.query_wasm_smart(
            self.authzpp_contract_addr.to_string(),
            &QueryMsg::SimulateExecute(ExecuteSettings {
                delegator: self.delegator_addr.to_string(),
                percentage,
            }),
        );
        simulation
    }

    /// Generates the execute message to execute the grant on behalf of the given delegator
    pub fn execute(&self, percentage: Option<Decimal>) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.authzpp_contract_addr.to_string(),
            msg: to_binary(&ExecuteMsg::Execute(ExecuteSettings {
                delegator: self.delegator_addr.to_string(),
                percentage,
            }))?,
            funds: vec![],
        }))
    }

    /// Simulates and executes the contract returning both the simulation response and the execute message to execute
    pub fn simulate_with_contract_execute(
        &self,
        querier: QuerierWrapper,
        percentage: Option<Decimal>,
    ) -> Result<(SimulateExecuteResponse, CosmosMsg), ContractError> {
        Ok((
            self.simulate(querier, percentage)?,
            self.execute(percentage)?,
        ))
    }

    /// Queries the contract for the active grant for the delegator, if any
    pub fn query_grant(&self, querier: QuerierWrapper) -> StdResult<GrantQueryResponse> {
        querier.query_wasm_smart(
            self.authzpp_contract_addr.to_string(),
            &QueryMsg::ActiveGrantsByDelegator(self.delegator_addr.to_string()),
        )
    }
}
