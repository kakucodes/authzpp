use crate::{
    msg::{AllowedDenomsSendSettings, ExecuteMsg, ExecuteSettings},
    tests::integration_helpers::upload_contract,
};
use authzpp_tt_wrappers::authz::Authz;
use cosmwasm_std::{Coin as CWCoin, Uint128};
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_test_tube::{
    cosmrs::proto::cosmos::bank::v1beta1::QueryBalanceRequest, Account, Bank, Module,
    OsmosisTestApp, Wasm,
};
use std::str::FromStr;

#[test]
fn execute_happy_path() {
    // create new osmosis appchain instance.
    let app = OsmosisTestApp::new();

    // create new account with initial funds
    // wallet that will be used to upload the contract
    let admin_addr = app
        .init_account(&[CWCoin::new(100_000_000_000, "uosmo")])
        .unwrap();
    // wallet that the authorized funds will be sent to
    let receiver_addr = app.init_account(&[]).unwrap();
    // the wallet that will be granting permissions and sending tokens
    let granter_addr = app
        .init_account(&[CWCoin::new(5_000_000_000_000, "uosmo")])
        .unwrap();
    // the wallet that will execute the send
    let grantee_addr = app
        .init_account(&[CWCoin::new(2_000_000_000_000, "uosmo")])
        .unwrap();

    // initialize the modules we'll work with
    let wasm = Wasm::new(&app);
    let bank = Bank::new(&app);
    let authz = Authz::new(&app);

    let contract_addr = upload_contract(
        &wasm,
        "../../target/wasm32-unknown-unknown/release/allowed_denoms_send.wasm",
        &admin_addr,
    );

    // create a send authorization so that the allowlist contract has access to sending tokens for granter
    let send_authorization = authz.create_send_authorization(
        grantee_addr.address(),
        contract_addr.clone(),
        vec![Coin {
            amount: 100_000_000_000u128.to_string(),
            denom: "uosmo".into(),
        }],
        Some(osmosis_std::shim::Timestamp {
            seconds: 1988193600i64,
            nanos: 100_000_000i32,
        }),
        &granter_addr,
    );

    assert!(
        send_authorization.is_ok(),
        "send authorization failed {:#?}",
        send_authorization
    );

    // create a grant on the allowlist contract
    let allowlist_grant = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Grant(AllowedDenomsSendSettings {
            grantee: grantee_addr.address(),
            allowed_denoms: vec!["uosmo".to_string()],
            expiration: cosmwasm_std::Timestamp::from_seconds(1988193600),
        }),
        &[],
        &granter_addr,
    );

    assert!(
        allowlist_grant.is_ok(),
        "allowlist grant failed {:#?}",
        allowlist_grant
    );

    let send_exec = wasm.execute(
        &contract_addr,
        &ExecuteMsg::Execute(ExecuteSettings {
            granter: granter_addr.address(),
            grantee: grantee_addr.address(),
            amount: vec![CWCoin {
                amount: 1_000_000u128.into(),
                denom: "uosmo".into(),
            }],
            receiver: receiver_addr.address(),
        }),
        &[],
        &grantee_addr,
    );

    assert!(send_exec.is_ok(), "send execution failed {:#?}", send_exec);

    let receiver_balance = bank.query_balance(&QueryBalanceRequest {
        address: receiver_addr.address(),
        denom: "uosmo".to_string(),
    });

    println!("receiver balance {:#?}", receiver_balance);

    assert_eq!(
        Uint128::from_str(receiver_balance.unwrap().balance.unwrap().amount.as_str()).unwrap(),
        Uint128::from(1_000_000u128)
    );
}
