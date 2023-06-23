use cosmwasm_std::{to_binary, Addr, CosmosMsg, Decimal, QuerierWrapper, StdResult, WasmMsg};

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
        percentage: Option<Decimal>,
    ) -> StdResult<SimulateExecuteResponse> {
        let simulation: StdResult<SimulateExecuteResponse> = querier.query_wasm_smart(
            self.contract_addr.clone(),
            &QueryMsg::SimulateExecute(ExecuteSettings {
                delegator: self.delegator_addr.to_string(),
                percentage,
            }),
        );

        simulation
    }

    pub fn execute(&self, percentage: Option<Decimal>) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contract_addr.to_string(),
            msg: to_binary(&ExecuteMsg::Execute(ExecuteSettings {
                delegator: self.delegator_addr.to_string(),
                percentage,
            }))?,
            funds: vec![],
        }))
    }

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

    pub fn query_grant(&self, querier: QuerierWrapper) -> StdResult<GrantQueryResponse> {
        querier.query_wasm_smart(
            self.contract_addr.clone(),
            &QueryMsg::ActiveGrantsByDelegator(self.delegator_addr.to_string()),
        )
    }
}
