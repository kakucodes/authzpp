use crate::msg::{
    ActiveGrantsByDelegatorResponse, AllowedWithdrawlSettings, ExecuteMsg, ExecuteSettings,
    GrantQueryResponse,
};
use crate::{
    msg::{QueryMsg, SimulateExecuteResponse},
    tests::integration_helpers::upload_contract,
};
use authzpp_tt_wrappers::authz::Authz;
use authzpp_tt_wrappers::distribution::Distribution;
use authzpp_tt_wrappers::staking::Staking;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};
use osmosis_std::types::cosmos::{
    base::v1beta1::Coin as OsmosisCoin, staking::v1beta1::MsgDelegate,
};
use osmosis_test_tube::{
    cosmrs::proto::cosmos::{
        bank::v1beta1::QueryBalanceRequest,
        distribution::v1beta1::QueryDelegationTotalRewardsRequest,
        staking::v1beta1::QueryValidatorsRequest,
    },
    Account, Bank, Module, OsmosisTestApp, Wasm,
};
use std::str::FromStr;

#[test]
fn execute_split_happy_path() {
    // create new osmosis appchain instance.
    let app = OsmosisTestApp::new();

    // create new account with initial funds
    // wallet that will be used to upload the contract
    let admin_addr = app
        .init_account(&[Coin::new(100_000_000_000, "uosmo")])
        .unwrap();
    // wallet that the withdraw split will be sent to
    let take_rate_addr = app.init_account(&[]).unwrap();
    // the wallet that will be delegating/receiving rewards and sharing the rewards with `take_rate_addr`
    let delegator_addr = app
        .init_account(&[Coin::new(5_000_000_000, "uosmo")])
        .unwrap();
    // the wallet that will execute the withdraw/withdraw split
    let grantee_addr = app
        .init_account(&[Coin::new(1_000_862_500, "uosmo")])
        .unwrap();

    // initialize the modules we'll work with
    let wasm = Wasm::new(&app);
    let bank = Bank::new(&app);
    let authz = Authz::new(&app);
    let staking = Staking::new(&app);
    let distribution = Distribution::new(&app);

    let existing_validators = staking
        .query_validators(&QueryValidatorsRequest {
            status: "".to_string(),
            pagination: None,
        })
        .unwrap();

    // set aside the first validator for testing
    let validator = existing_validators.validators.first().unwrap().clone();

    let contract_addr = upload_contract(
        &wasm,
        "../../target/wasm32-unknown-unknown/release/withdraw_rewards_tax_grant.wasm",
        &admin_addr,
    );

    // delegate some tokens so accruing rewards can start
    let delegate_tokens = staking.delegate(
        MsgDelegate {
            delegator_address: delegator_addr.address(),
            validator_address: validator.operator_address,
            amount: Some(OsmosisCoin {
                denom: "uosmo".to_string(),
                amount: 1000.to_string(),
            }),
        },
        &delegator_addr,
    );

    assert!(
        delegate_tokens.is_ok(),
        "delegation failed {:#?}",
        delegate_tokens
    );

    // allow staking rewards to accrue for an hour
    app.increase_time(60u64 * 60u64);

    // give the contract the permission for withdrawing rewards
    let withdraw_rewards_grant = authz.create_generic_grant(
        contract_addr.to_string(),
        delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &delegator_addr,
    );

    // give the contract the permission for setting withdraw address
    let withdraw_address_grant = authz.create_generic_grant(
        contract_addr.to_string(),
        delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &delegator_addr,
    );

    assert!(
        withdraw_rewards_grant.is_ok(),
        "grant withdraw rewards failed"
    );
    assert!(
        withdraw_address_grant.is_ok(),
        "grant withdraw address failed"
    );

    // set the withdrawl split grant in the authzpp contract
    let withdraw_contract_grant = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedWithdrawlSettings {
            grantee: grantee_addr.address(),
            max_fee_percentage: Decimal::percent(5),
            // Saturday, January 1, 2033 12:00:00 PM
            expiration: Timestamp::from_seconds(1988193600u64),
            taxation_address: take_rate_addr.address(),
        }),
        &[],
        &delegator_addr,
    );

    assert!(
        withdraw_contract_grant.is_ok(),
        "grant withdraw contract failed"
    );

    // query contract state to check if contract instantiation works properly
    let first_user_grant = wasm.query::<QueryMsg, ActiveGrantsByDelegatorResponse>(
        &contract_addr,
        &QueryMsg::ActiveGrantsByDelegator(delegator_addr.address()),
    );

    assert_eq!(
        first_user_grant,
        Ok(Some(GrantQueryResponse {
            delegator_addr: Addr::unchecked(delegator_addr.address()),
            allowed_withdrawls: AllowedWithdrawlSettings {
                grantee: grantee_addr.address(),
                max_fee_percentage: Decimal::percent(5),
                expiration: Timestamp::from_seconds(1988193600u64),
                taxation_address: take_rate_addr.address(),
            }
        }))
    );

    let _ = distribution
        .query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: delegator_addr.address(),
        })
        .unwrap();

    // println!(
    //     "one day later user_staking_rewards: {:#?}",
    //     user_staking_rewards
    //         .unwrap()
    //         .total
    //         .iter()
    //         .map(|x| Coin::new(
    //             Uint128::from_str(&x.amount).unwrap().u128() / 1_000_000_000_000_000_000u128,
    //             x.denom.clone()
    //         ))
    //         .collect::<Vec<Coin>>()
    // );

    // check that the authzpp contract simulates the withdraw split successfully
    let simulated_split = wasm.query::<QueryMsg, SimulateExecuteResponse>(
        &contract_addr,
        &QueryMsg::SimulateExecute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
    );

    assert!(
        simulated_split.is_ok(),
        "simulated split failed: {:#?}",
        simulated_split
    );

    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_ok(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let user_staking_rewards = distribution
        .query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: delegator_addr.address(),
        })
        .unwrap();

    // check if the delegators' rewards have been claimed
    assert_eq!(user_staking_rewards.total, []);

    let take_rate_wallet_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: take_rate_addr.address(),
            denom: "uosmo".to_string(),
        })
        .unwrap();

    // check if the take rate wallet has received tokens
    assert!(
        Uint128::from_str(&take_rate_wallet_balance.balance.clone().unwrap().amount)
            .unwrap()
            .gt(&Uint128::zero()),
        "after take_rate_wallet_balance: {:#?}",
        take_rate_wallet_balance
    );
}

#[test]
fn multiple_granters() {
    // create new osmosis appchain instance.
    let app = OsmosisTestApp::new();

    // create new account with initial funds
    // wallet that will be used to upload the contract
    let admin_addr = app
        .init_account(&[Coin::new(100_000_000_000, "uosmo")])
        .unwrap();
    // wallets that the withdraw split will be sent to
    let take_rate_addr = app.init_account(&[]).unwrap();
    let second_take_rate_addr = app.init_account(&[]).unwrap();

    // the wallets that will be delegating/receiving rewards and sharing the rewards with `take_rate_addr`
    let delegator_addr = app
        .init_account(&[Coin::new(5_000_000_000, "uosmo")])
        .unwrap();
    let second_delegator_addr = app
        .init_account(&[Coin::new(5_000_000_000, "uosmo")])
        .unwrap();
    let third_delegator_addr = app
        .init_account(&[Coin::new(5_000_000_000, "uosmo")])
        .unwrap();

    // the wallets that will execute the withdraw/withdraw split
    let grantee_addr = app
        .init_account(&[Coin::new(3_000_862_500, "uosmo")])
        .unwrap();
    let second_grantee_addr = app
        .init_account(&[Coin::new(3_000_862_500, "uosmo")])
        .unwrap();

    // initialize the modules we'll work with
    let wasm = Wasm::new(&app);
    let bank = Bank::new(&app);
    let authz = Authz::new(&app);
    let staking = Staking::new(&app);
    let distribution = Distribution::new(&app);

    let existing_validators = staking
        .query_validators(&QueryValidatorsRequest {
            status: "".to_string(),
            pagination: None,
        })
        .unwrap();

    // set aside the first validator for testing
    let validator = existing_validators.validators.first().unwrap().clone();

    let contract_addr = upload_contract(
        &wasm,
        "../../target/wasm32-unknown-unknown/release/withdraw_rewards_tax_grant.wasm",
        &admin_addr,
    );

    // delegate some tokens so accruing rewards can start
    let _ = staking.delegate(
        MsgDelegate {
            delegator_address: delegator_addr.address(),
            validator_address: validator.operator_address.clone(),
            amount: Some(OsmosisCoin {
                denom: "uosmo".to_string(),
                amount: 1000.to_string(),
            }),
        },
        &delegator_addr,
    );
    let _ = staking.delegate(
        MsgDelegate {
            delegator_address: second_delegator_addr.address(),
            validator_address: validator.operator_address.clone(),
            amount: Some(OsmosisCoin {
                denom: "uosmo".to_string(),
                amount: 2000.to_string(),
            }),
        },
        &second_delegator_addr,
    );
    let _ = staking.delegate(
        MsgDelegate {
            delegator_address: third_delegator_addr.address(),
            validator_address: validator.operator_address,
            amount: Some(OsmosisCoin {
                denom: "uosmo".to_string(),
                amount: 3000.to_string(),
            }),
        },
        &third_delegator_addr,
    );

    // allow staking rewards to accrue for an hour
    app.increase_time(60u64 * 60u64);

    // give the contract the permission for withdrawing rewards
    let _ = authz.create_generic_grant(
        contract_addr.to_string(),
        delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &delegator_addr,
    );

    // give the contract the permission for setting withdraw address
    let _ = authz.create_generic_grant(
        contract_addr.to_string(),
        delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &delegator_addr,
    );
    // give the contract the permission for withdrawing rewards
    let _ = authz.create_generic_grant(
        contract_addr.to_string(),
        second_delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &second_delegator_addr,
    );

    // give the contract the permission for setting withdraw address
    let _ = authz.create_generic_grant(
        contract_addr.to_string(),
        second_delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &second_delegator_addr,
    );
    // give the contract the permission for withdrawing rewards
    let _ = authz.create_generic_grant(
        contract_addr.to_string(),
        third_delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &third_delegator_addr,
    );

    // give the contract the permission for setting withdraw address
    let _ = authz.create_generic_grant(
        contract_addr.to_string(),
        third_delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &third_delegator_addr,
    );

    // set the withdrawl split grant in the authzpp contract
    let _ = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedWithdrawlSettings {
            grantee: grantee_addr.address(),
            max_fee_percentage: Decimal::percent(5),
            // Saturday, January 1, 2033 12:00:00 PM
            expiration: Timestamp::from_seconds(1988193600u64),
            taxation_address: take_rate_addr.address(),
        }),
        &[],
        &delegator_addr,
    );

    // set the withdrawl split grant in the authzpp contract
    let _ = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedWithdrawlSettings {
            grantee: grantee_addr.address(),
            max_fee_percentage: Decimal::percent(5),
            // Saturday, January 1, 2033 12:00:00 PM
            expiration: Timestamp::from_seconds(1988193600u64),
            taxation_address: second_take_rate_addr.address(),
        }),
        &[],
        &second_delegator_addr,
    );

    // set the withdrawl split grant in the authzpp contract
    let withdraw_contract_grant = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedWithdrawlSettings {
            grantee: second_grantee_addr.address(),
            max_fee_percentage: Decimal::percent(5),
            // Saturday, January 1, 2033 12:00:00 PM
            expiration: Timestamp::from_seconds(1988193600u64),
            taxation_address: second_take_rate_addr.address(),
        }),
        &[],
        &third_delegator_addr,
    );

    assert!(
        withdraw_contract_grant.is_ok(),
        "grant withdraw contract failed"
    );

    // query contract state to check if contract instantiation works properly
    let first_user_grant = wasm.query::<QueryMsg, ActiveGrantsByDelegatorResponse>(
        &contract_addr,
        &QueryMsg::ActiveGrantsByDelegator(delegator_addr.address()),
    );

    assert_eq!(
        first_user_grant,
        Ok(Some(GrantQueryResponse {
            delegator_addr: Addr::unchecked(delegator_addr.address()),
            allowed_withdrawls: AllowedWithdrawlSettings {
                grantee: grantee_addr.address(),
                max_fee_percentage: Decimal::percent(5),
                expiration: Timestamp::from_seconds(1988193600u64),
                taxation_address: take_rate_addr.address(),
            }
        }))
    );

    // query contract state to check if contract instantiation works properly
    let second_user_grant = wasm.query::<QueryMsg, ActiveGrantsByDelegatorResponse>(
        &contract_addr,
        &QueryMsg::ActiveGrantsByDelegator(second_delegator_addr.address()),
    );

    assert_eq!(
        second_user_grant,
        Ok(Some(GrantQueryResponse {
            delegator_addr: Addr::unchecked(second_delegator_addr.address()),
            allowed_withdrawls: AllowedWithdrawlSettings {
                grantee: grantee_addr.address(),
                max_fee_percentage: Decimal::percent(5),
                expiration: Timestamp::from_seconds(1988193600u64),
                taxation_address: second_take_rate_addr.address(),
            }
        }))
    );

    ////// execute withdraw for first delegator ////

    // attempt to execute withdraw split with a grantee that doesn't have permission
    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &second_grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_err(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_ok(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let user_staking_rewards = distribution
        .query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: delegator_addr.address(),
        })
        .unwrap();

    // check if the delegators' rewards have been claimed
    assert_eq!(user_staking_rewards.total, []);

    let take_rate_wallet_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: take_rate_addr.address(),
            denom: "uosmo".to_string(),
        })
        .unwrap();

    // check if the take rate wallet has received tokens
    assert!(
        Uint128::from_str(&take_rate_wallet_balance.balance.clone().unwrap().amount)
            .unwrap()
            .gt(&Uint128::zero()),
        "after take_rate_wallet_balance: {:#?}",
        take_rate_wallet_balance
    );

    ////// execute withdraw for second delegator ////

    // attempt to execute withdraw split with a grantee that doesn't have permission
    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: second_delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &second_grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_err(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: second_delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_ok(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let user_staking_rewards = distribution
        .query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: second_delegator_addr.address(),
        })
        .unwrap();

    // check if the delegators' rewards have been claimed
    assert_eq!(user_staking_rewards.total, []);

    let second_take_rate_wallet_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: second_take_rate_addr.address(),
            denom: "uosmo".to_string(),
        })
        .unwrap();

    // check if the take rate wallet has received tokens
    assert!(
        Uint128::from_str(
            &second_take_rate_wallet_balance
                .balance
                .clone()
                .unwrap()
                .amount
        )
        .unwrap()
        .gt(&Uint128::zero()),
        "after take_rate_wallet_balance: {:#?}",
        take_rate_wallet_balance
    );

    ////// execute withdraw for third delegator ////

    // attempt to execute withdraw split with a grantee that doesn't have permission
    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: third_delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_err(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: third_delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &second_grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_ok(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    let user_staking_rewards = distribution
        .query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: third_delegator_addr.address(),
        })
        .unwrap();

    // check if the delegators' rewards have been claimed
    assert_eq!(user_staking_rewards.total, []);

    let take_rate_wallet_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: second_take_rate_addr.address(),
            denom: "uosmo".to_string(),
        })
        .unwrap();

    // check if the take rate wallet has received tokens
    assert!(
        Uint128::from_str(&take_rate_wallet_balance.balance.clone().unwrap().amount)
            .unwrap()
            .gt(
                &Uint128::from_str(&second_take_rate_wallet_balance.balance.unwrap().amount)
                    .unwrap()
            ),
        "after take_rate_wallet_balance: {:#?}",
        take_rate_wallet_balance
    );
}

#[test]
fn unauthorized_users_cannot_execute() {
    // create new osmosis appchain instance.
    let app = OsmosisTestApp::new();

    // create new account with initial funds
    // wallet that will be used to upload the contract
    let admin_addr = app
        .init_account(&[Coin::new(100_000_000_000, "uosmo")])
        .unwrap();
    // wallet that the withdraw split will be sent to
    let take_rate_addr = app.init_account(&[]).unwrap();
    // the wallet that will be delegating/receiving rewards and sharing the rewards with `take_rate_addr`
    let delegator_addr = app
        .init_account(&[Coin::new(5_000_000_000, "uosmo")])
        .unwrap();
    // the wallet that will execute the withdraw/withdraw split
    let grantee_addr = app
        .init_account(&[Coin::new(1_000_862_500, "uosmo")])
        .unwrap();

    // random address that should never be able to call execution of delegator_addr's grant
    let rando_addr = app
        .init_account(&[Coin::new(100_000_000_000, "uosmo")])
        .unwrap();

    // initialize the modules we'll work with
    let wasm = Wasm::new(&app);
    // let bank = Bank::new(&app);
    let authz = Authz::new(&app);
    let staking = Staking::new(&app);
    // let distribution = Distribution::new(&app);

    let existing_validators = staking
        .query_validators(&QueryValidatorsRequest {
            status: "".to_string(),
            pagination: None,
        })
        .unwrap();

    // set aside the first validator for testing
    let validator = existing_validators.validators.first().unwrap().clone();

    let contract_addr = upload_contract(
        &wasm,
        "../../target/wasm32-unknown-unknown/release/withdraw_rewards_tax_grant.wasm",
        &admin_addr,
    );

    // delegate some tokens so accruing rewards can start
    let delegate_tokens = staking.delegate(
        MsgDelegate {
            delegator_address: delegator_addr.address(),
            validator_address: validator.operator_address,
            amount: Some(OsmosisCoin {
                denom: "uosmo".to_string(),
                amount: 1000.to_string(),
            }),
        },
        &delegator_addr,
    );

    assert!(
        delegate_tokens.is_ok(),
        "delegation failed {:#?}",
        delegate_tokens
    );

    // allow staking rewards to accrue for an hour
    app.increase_time(60u64 * 60u64);

    // give the contract the permission for withdrawing rewards
    let withdraw_rewards_grant = authz.create_generic_grant(
        contract_addr.to_string(),
        delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &delegator_addr,
    );

    // give the contract the permission for setting withdraw address
    let withdraw_address_grant = authz.create_generic_grant(
        contract_addr.to_string(),
        delegator_addr.address(),
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &delegator_addr,
    );

    assert!(
        withdraw_rewards_grant.is_ok(),
        "grant withdraw rewards failed"
    );
    assert!(
        withdraw_address_grant.is_ok(),
        "grant withdraw address failed"
    );

    // set the withdrawl split grant in the authzpp contract
    let withdraw_contract_grant = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedWithdrawlSettings {
            grantee: grantee_addr.address(),
            max_fee_percentage: Decimal::percent(5),
            // Saturday, January 1, 2033 12:00:00 PM
            expiration: Timestamp::from_seconds(1988193600u64),
            taxation_address: take_rate_addr.address(),
        }),
        &[],
        &delegator_addr,
    );

    assert!(
        withdraw_contract_grant.is_ok(),
        "grant withdraw contract failed"
    );

    // ensure that random users cannot withdraw rewards
    let unauthorized_withdraw = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &rando_addr,
    );

    assert!(
        unauthorized_withdraw.is_err(),
        "unauthorized withdraw should fail"
    );

    // ensure that the take_rate wallet also cannot withdraw the rewards
    let unauthorized_withdraw = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &take_rate_addr,
    );

    assert!(
        unauthorized_withdraw.is_err(),
        "unauthorized withdraw should fail"
    );

    let execute_withdraw_split = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &grantee_addr,
    );

    assert!(
        execute_withdraw_split.is_ok(),
        "execute withdraw split failed: {:#?}",
        execute_withdraw_split
    );

    // push time forward again to accrue staking rewards
    app.increase_time(86400u64);

    // ensure that random users cannot withdraw rewards
    let unauthorized_withdraw = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            delegator: delegator_addr.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        &rando_addr,
    );

    assert!(
        unauthorized_withdraw.is_err(),
        "unauthorized withdraw should fail"
    );
}
