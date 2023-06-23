use cosmwasm_std::{to_binary, Addr, CosmosMsg, QuerierWrapper, StdResult, WasmMsg};

use crate::{
    msg::{ExecuteMsg, ExecuteSettings, GrantQueryResponse, QueryMsg, SimulateExecuteResponse},
    ContractError,
};

pub struct WithdrawRewardsTaxClient {
    /// Address of the grant contract
    contract_addr: Addr,
    /// Address of the delegator/granter
    delegator_addr: Addr,
}

impl WithdrawRewardsTaxClient {
    pub fn simulate(
        &self,
        querier: QuerierWrapper,
        execute_settings: &ExecuteSettings,
    ) -> StdResult<SimulateExecuteResponse> {
        let simulation: StdResult<SimulateExecuteResponse> = querier.query_wasm_smart(
            self.contract_addr.clone(),
            &QueryMsg::SimulateExecute(execute_settings.clone()),
        );

        simulation
    }

    pub fn execute(&self, execute_settings: &ExecuteSettings) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contract_addr.to_string(),
            msg: to_binary(&ExecuteMsg::Execute(execute_settings.clone()))?,
            funds: vec![],
        }))
    }

    pub fn simulate_with_contract_execute(
        &self,
        querier: QuerierWrapper,
        execute_settings: ExecuteSettings,
    ) -> Result<(SimulateExecuteResponse, CosmosMsg), ContractError> {
        Ok((
            self.simulate(querier, &execute_settings)?,
            self.execute(&execute_settings)?,
        ))
    }

    pub fn query_grant(&self, querier: QuerierWrapper) -> StdResult<GrantQueryResponse> {
        querier.query_wasm_smart(
            self.contract_addr.clone(),
            &QueryMsg::ActiveGrantsByDelegator(self.delegator_addr.to_string()),
        )
    }
}
