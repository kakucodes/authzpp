use crate::error::ContractError;
use crate::helpers::{
    denoms_allowed, validate_grantee_address, validate_granter_address, validate_receiver_address,
};
use crate::msg::{
    AllowedDenomsSendSettings, ExecuteMsg, ExecuteSettings, InstantiateMsg, QueryMsg,
};
use crate::queries::{self, query_active_grant};
use crate::state::GRANTS;
use authzpp_utils::helpers::Expirable;
use authzpp_utils::msg_gen::exec_msg;
use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::traits::MessageExt;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:authzpp-allowlist-send";
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
        ExecuteMsg::Grant(grant_settings) => {
            // validate the addresses
            let grantee_addr = validate_grantee_address(deps.api, &grant_settings.grantee)?;

            // store the grant in state under the address of the user that executed the contract
            GRANTS.save(deps.storage, (&info.sender, &grantee_addr), &grant_settings)?;

            Ok(Response::default()
                .add_attribute("action", "grant_allowlist_send")
                .add_attribute("granter", info.sender)
                .add_attribute("grantee", grantee_addr.to_string()))
        }
        ExecuteMsg::Revoke(receiver) => {
            let receiver_addr = validate_receiver_address(deps.api, &receiver)?;

            // remove the grant from state
            GRANTS.remove(deps.storage, (&info.sender, &receiver_addr));

            Ok(Response::default()
                .add_attribute("action", "revoke")
                .add_attribute("granter", info.sender))
        }
        ExecuteMsg::Execute(ExecuteSettings {
            granter,
            grantee,
            receiver,
            amount,
        }) => {
            // validate the addresses
            let grantee_addr = validate_grantee_address(deps.api, &grantee)?;
            let receiver_addr = validate_receiver_address(deps.api, &receiver)?;
            let granter_addr = validate_granter_address(deps.api, &granter)?;

            // query the grant settings, this will error if there is no active/unexpired grant
            let grant = query_active_grant(deps.storage, &env.block, &granter_addr, &grantee_addr)?;

            // validate that the funds being sent are within those that were granted
            denoms_allowed(&grant.allowed_denoms, &amount)?;

            // generate the actual send message wrapped in the appropriate authz exec message
            let send_msg = exec_msg(
                &info.sender,
                vec![MsgSend {
                    from_address: granter_addr.to_string(),
                    to_address: receiver_addr.to_string(),
                    amount: amount
                        .into_iter()
                        .map(|coin| Coin {
                            denom: coin.denom,
                            amount: coin.amount.into(),
                        })
                        .collect(),
                }
                .to_any()?],
            )?;

            Ok(Response::default()
                .add_message(send_msg)
                .add_attribute("action", "send_in_allowlist")
                .add_attribute("granter", granter)
                .add_attribute("grantee", grantee))
        }
        ExecuteMsg::ProcessExecuteWithoutBroadcast(ExecuteSettings {
            granter,
            grantee,
            receiver,
            ..
        }) => {
            // validate the addresses
            let grantee_addr = validate_grantee_address(deps.api, &grantee)?;
            let _receiver_addr = validate_receiver_address(deps.api, &receiver)?;
            let granter_addr = validate_granter_address(deps.api, &granter)?;

            // query the grant settings, this will error if there is no active/unexpired grant
            let _ = query_active_grant(deps.storage, &env.block, &granter_addr, &grantee_addr)?;

            // everything is good and clear with this grant so we can just return an ok response
            // since we're not broadcasting there's no actual message to put together
            Ok(Response::default()
                .add_attribute("action", "send_in_allowlist_without_broadcast")
                .add_attribute("granter", granter)
                .add_attribute("grantee", grantee))
        }
        ExecuteMsg::PruneExpiredGrants() => {
            let expired_grants: Vec<(Addr, Addr)> = GRANTS
                .range(deps.storage, None, None, Order::Ascending)
                .filter_map(|result| match result {
                    Ok((key, grant)) if grant.is_expired(&env.block) => Some(key),
                    _ => None,
                })
                .collect();

            for k in expired_grants.iter() {
                GRANTS.remove(deps.storage, (&k.0, &k.1));
            }

            Ok(Response::default()
                .add_attribute("action", "prune_expired_grants")
                .add_attribute("count", expired_grants.len().to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Version {} => to_binary(&queries::query_version()).map_err(ContractError::Std),
        QueryMsg::ActiveGrantsByGranter(granter) => {
            let granter_addr = validate_granter_address(deps.api, &granter)?;

            let grants: Vec<AllowedDenomsSendSettings> = GRANTS
                .prefix(&granter_addr)
                .range(deps.storage, None, None, Order::Ascending)
                .filter_map(|result| match result {
                    Ok((_, grant)) if grant.is_not_expired(&env.block) => Some(grant),
                    _ => None,
                })
                .collect();

            to_binary(&grants).map_err(ContractError::Std)
        }
        QueryMsg::ActiveGrantsByGrantee(grantee) => {
            let grantee_addr = validate_grantee_address(deps.api, &grantee)?;

            let grants: Vec<AllowedDenomsSendSettings> = GRANTS
                .range(deps.storage, None, None, Order::Ascending)
                .filter_map(|result| match result {
                    Ok((_, grant))
                        if grant.grantee == grantee_addr && grant.is_not_expired(&env.block) =>
                    {
                        Some(grant)
                    }
                    _ => None,
                })
                .collect::<Vec<AllowedDenomsSendSettings>>();

            to_binary(&grants).map_err(ContractError::Std)
        }
        QueryMsg::Grant { granter, grantee } => {
            let granter_addr = validate_granter_address(deps.api, &granter)?;
            let grantee_addr = validate_receiver_address(deps.api, &grantee)?;

            let grant = GRANTS.load(deps.storage, (&granter_addr, &grantee_addr));

            let grant = match grant {
                Ok(grant) => Some(grant),
                Err(_) => None,
            };

            to_binary(&grant).map_err(ContractError::Std)
        }
    }
}
