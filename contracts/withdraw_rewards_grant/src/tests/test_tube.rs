use authzpp_tt_wrappers::test_tube_wrappers::authz_wrapper::Authz;
use cosmos_sdk_proto::traits::Message;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};

use osmosis_std::{
    shim::Any,
    types::cosmos::{
        authz::v1beta1::{GenericAuthorization, Grant, MsgGrant},
        base::v1beta1::Coin as OsmosisCoin,
        staking::v1beta1::MsgDelegate,
    },
};
use osmosis_test_tube::{
    cosmrs::proto::cosmos::{
        bank::v1beta1::QueryBalanceRequest,
        distribution::v1beta1::QueryDelegationTotalRewardsRequest,
        staking::v1beta1::QueryValidatorsRequest,
    },
    Account, Bank, Module, OsmosisTestApp, Wasm,
};

use crate::msg::{
    ActiveGrantsByGranteeResponse, ActiveGrantsByGranterResponse, AllowedWithdrawlSettings,
    ExecuteMsg, ExecuteSettings, GrantQueryResponse, InstantiateMsg, QueryMsg,
};

#[test]
fn test_fn() {
    // create new osmosis appchain instance.
    let app = OsmosisTestApp::new();

    // create new account with initial funds
    let accs = app
        .init_accounts(&[Coin::new(4_000_000_000_000, "uosmo")], 4)
        .unwrap();

    // `Wasm` is the module we use to interact with cosmwasm releated logic on the appchain
    // it implements `Module` trait which you will see more later.
    let wasm = Wasm::new(&app);
    let bank = Bank::new(&app);
    let authz = Authz::new(&app);
    let staking = Staking::new(&app);
    let distribution = Distribution::new(&app);

    let admin = &accs[0];
    let take_rate_address = &accs[1];
    let first_user = &accs[2];
    let second_user = &accs[3];

    let existing_validators = staking
        .query_validators(&QueryValidatorsRequest {
            status: "".to_string(),
            pagination: None,
        })
        .unwrap();

    let validator = existing_validators.validators.first().unwrap().clone();

    let _ = staking.delegate(
        MsgDelegate {
            delegator_address: first_user.address(),
            validator_address: validator.operator_address,
            amount: Some(OsmosisCoin {
                denom: "uosmo".to_string(),
                amount: 100_000_000.to_string(),
            }),
        },
        first_user,
    );

    // allow staking rewards to accrue for a day
    app.increase_time(86400u64);

    let user_staking_rewards =
        distribution.query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: first_user.address(),
        });

    println!("user_staking_rewards: {:#?}", user_staking_rewards);

    // println!("delegation: {:#?}", user_delegation);

    // Load compiled wasm bytecode
    let wasm_byte_code =
        std::fs::read("../../target/wasm32-unknown-unknown/release/withdraw_rewards_grant.wasm")
            .unwrap();
    let code_id = wasm
        .store_code(&wasm_byte_code, None, take_rate_address)
        .unwrap()
        .data
        .code_id;

    // instantiate contract
    let contract_addr = wasm
        .instantiate(
            code_id,
            &InstantiateMsg {},
            None,  // contract admin used for migration, not the same as cw1_whitelist admin
            None,  // contract label
            &[],   // funds
            admin, // signer
        )
        .unwrap()
        .data
        .address;

    let _ = authz.create_grant(
        MsgGrant {
            granter: first_user.address(),
            grantee: contract_addr.to_string(),
            grant: Some(Grant {
                authorization: Some(Any {
                    type_url: GenericAuthorization::TYPE_URL.to_string(),
                    value: GenericAuthorization {
                        msg: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
                    }
                    .encode_to_vec(),
                }),
                expiration: Some(osmosis_std::shim::Timestamp {
                    seconds: 1988193600i64,
                    nanos: 100_000_000i32,
                }),
            }),
        },
        first_user,
    );

    let _ = authz.create_grant(
        MsgGrant {
            granter: first_user.address(),
            grantee: contract_addr.to_string(),
            grant: Some(Grant {
                authorization: Some(Any {
                    type_url: GenericAuthorization::TYPE_URL.to_string(),
                    value: GenericAuthorization {
                        msg: "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
                    }
                    .encode_to_vec(),
                }),
                expiration: Some(osmosis_std::shim::Timestamp {
                    seconds: 1988193600i64,
                    nanos: 100_000_000i32,
                }),
            }),
        },
        first_user,
    );

    let _ = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedWithdrawlSettings {
            grantee: second_user.address(),
            max_fee_percentage: Decimal::percent(5),
            // Saturday, January 1, 2033 12:00:00 PM
            expiration: Timestamp::from_seconds(1988193600u64),
            withdraw_fee_address: take_rate_address.address(),
        }),
        &[],
        first_user,
    );

    // println!("{:#?}", grant_execution);

    // debug_assert!(grant_execution.is_ok());

    // query contract state to check if contract instantiation works properly
    let first_user_grant = wasm.query::<QueryMsg, ActiveGrantsByGranterResponse>(
        &contract_addr,
        &QueryMsg::ActiveGrantsByGranter(first_user.address()),
    );

    debug_assert_eq!(
        first_user_grant,
        Ok(Some(GrantQueryResponse {
            granter: Addr::unchecked(first_user.address()),
            allowed_withdrawls: AllowedWithdrawlSettings {
                grantee: second_user.address(),
                max_fee_percentage: Decimal::percent(5),
                expiration: Timestamp::from_seconds(1988193600u64),
                withdraw_fee_address: take_rate_address.address(),
            }
        }))
    );

    // // query contract state to check if contract instantiation works properly
    let _ = wasm.query::<QueryMsg, ActiveGrantsByGranteeResponse>(
        &contract_addr,
        &QueryMsg::ActiveGrantsByGrantee(take_rate_address.address()),
    );

    // debug_assert_eq!(
    //     contract_grants,
    //     Ok(vec![GrantQueryResponse {
    //         granter: Addr::unchecked(first_user.address()),
    //         allowed_withdrawls: AllowedWithdrawlSettings {
    //             grantee: take_rate_address.address(),
    //             max_percentage: Decimal::percent(5),
    //             expiration: Timestamp::from_seconds(1988193600u64),
    //         }
    //     }])
    // );

    let take_rate_wallet_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: take_rate_address.address(),
            denom: "uosmo".to_string(),
        })
        .unwrap();
    println!(
        "before take_rate_wallet_balance: {:#?}",
        take_rate_wallet_balance
    );

    let _ = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            granter: first_user.address(),
            percentage: Some(Decimal::percent(5)),
        }),
        &[],
        second_user,
    );

    let user_staking_rewards = distribution
        .query_delegation_total_rewards(&QueryDelegationTotalRewardsRequest {
            delegator_address: first_user.address(),
        })
        .unwrap();

    assert_eq!(user_staking_rewards.total, []);

    let take_rate_wallet_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: take_rate_address.address(),
            denom: "uosmo".to_string(),
        })
        .unwrap();
    println!(
        "after take_rate_wallet_balance: {:#?}",
        take_rate_wallet_balance
    );

    // println!(
    //     "user_staking_rewards after execute: {:#?}",
    //     user_staking_rewards
    // );
}
