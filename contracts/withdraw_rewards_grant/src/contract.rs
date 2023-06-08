use crate::error::ContractError;
use crate::execute::{execute_rewards_withdraw, RewardExecutionMsgs};
use crate::msg::{AllowedWithdrawlSettings, ExecuteMsg, ExecuteSettings, InstantiateMsg, QueryMsg};
use crate::queries::query_active_grants_by_grantee;
use crate::queries::{self, query_active_grants_by_granter, query_split_rewards};
use crate::state::GRANTS;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:withdraw-rewards-grant";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: InstantiateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Grant(grant_spec) => {
            // validate that the grantee address is valid
            let grantee_addr = deps
                .api
                .addr_validate(&grant_spec.grantee)
                .map_err(|_| ContractError::InvalidGranteeAddress(grant_spec.clone().grantee))?;

            // validate that the withdraw share address is valid
            let _ = deps
                .api
                .addr_validate(&grant_spec.withdraw_fee_address)
                .map_err(|_| {
                    ContractError::InvalidWithdrawShareAddress(
                        grant_spec.clone().withdraw_fee_address,
                    )
                })?;

            GRANTS.save(deps.storage, info.sender.clone(), &grant_spec)?;

            Ok(Response::default()
                .add_attribute("action", "grant")
                .add_attribute("granter", info.sender)
                .add_attribute("grantee", grantee_addr.to_string()))
        }
        ExecuteMsg::Revoke() => {
            // remove the grant from state
            GRANTS.remove(deps.storage, info.sender.clone());

            Ok(Response::default()
                .add_attribute("action", "revoke")
                .add_attribute("granter", info.sender))
        }
        ExecuteMsg::Execute(ExecuteSettings {
            granter,
            percentage,
        }) => {
            // validate the granter address
            let granter_addr = deps
                .api
                .addr_validate(&granter)
                .map_err(|_| ContractError::InvalidGranterAddress(granter.clone()))?;

            let RewardExecutionMsgs { msgs, grantee } = execute_rewards_withdraw(
                &deps.querier,
                deps.storage,
                env.block,
                info.sender,
                granter_addr.clone(),
                percentage,
            )?;

            Ok(Response::default()
                .add_messages(msgs)
                .add_attribute("action", "execute_withdraw_rewards_split")
                .add_attribute("granter", granter_addr)
                .add_attribute("grantee", grantee))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Version {} => to_binary(&queries::query_version()).map_err(ContractError::Std),
        QueryMsg::ActiveGrantsByGranter(granter) => {
            // validate the granter address
            let granter = deps
                .api
                .addr_validate(&granter)
                .map_err(|_| ContractError::InvalidGranterAddress(granter))?;

            let grant = query_active_grants_by_granter(deps.storage, env.block, granter)?;

            to_binary(&grant).map_err(ContractError::Std)
        }
        QueryMsg::ActiveGrantsByGrantee(grantee) => {
            // validate the grantee address
            let grantee = deps
                .api
                .addr_validate(&grantee)
                .map_err(|_| ContractError::InvalidGranteeAddress(grantee))?;

            // check all the grants to see if the grantee is the one being queried
            // and that the grant is active
            let grants = query_active_grants_by_grantee(deps.storage, env.block, grantee);

            to_binary(&grants).map_err(ContractError::Std)
        }
        QueryMsg::SimulateExecute(ExecuteSettings {
            granter,
            percentage: requested_percentage,
        }) => {
            // validate that the granter/delegator address is valid
            let delegator = deps
                .api
                .addr_validate(&granter)
                .map_err(|_| ContractError::InvalidGranterAddress(granter))?;

            // get the max fee percentage from the delegator's grant settings
            let AllowedWithdrawlSettings {
                max_fee_percentage, ..
            } = query_active_grants_by_granter(deps.storage, env.block, delegator.clone())?
                .unwrap()
                .allowed_withdrawls;

            // get the split rewards
            let (split_rewards, _) = query_split_rewards(
                &deps.querier,
                &delegator,
                max_fee_percentage,
                requested_percentage,
            )?;

            to_binary(&split_rewards).map_err(ContractError::Std)
        }
    }
}
