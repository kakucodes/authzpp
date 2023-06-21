use authzpp_utils::msg_gen::exec_msg;
use cosmos_sdk_proto::{
    cosmos::{
        authz::v1beta1::MsgExec,
        distribution::v1beta1::{MsgSetWithdrawAddress, MsgWithdrawDelegatorReward},
    },
    traits::{Message, MessageExt},
};
use cosmwasm_std::{Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Timestamp};

use crate::{
    execute::{
        create_withdraw_rewards_exec_msg, generate_reward_withdrawl_msgs, RewardExecutionMsgs,
    },
    helpers::{partition_coins_by_percentage, split_rewards, sum_coins},
    msg::{AllowedWithdrawlSettings, SimulateExecuteResponse},
    queries::{AllPendingRewards, PendingReward},
};

// unit tests for the sum_coins helper function
#[test]
fn sum_coins_test() {
    let xs = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 100u128.into(),
        },
    ];
    let ys = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 100u128.into(),
        },
    ];
    let expected = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 200u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 200u128.into(),
        },
    ];
    assert_eq!(sum_coins(xs, ys), expected);

    let xs = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 100u128.into(),
        },
    ];
    let ys = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "ubtc".to_string(),
            amount: 100u128.into(),
        },
    ];
    let expected = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 200u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "ubtc".to_string(),
            amount: 100u128.into(),
        },
    ];
    assert_eq!(sum_coins(xs, ys), expected);
}

#[test]
fn partition_coins() {
    let coins = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 200u128.into(),
        },
    ];
    let (coins_to_send, coins_to_remain) =
        partition_coins_by_percentage(Decimal::percent(25), coins);
    let expected_to_send = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 25u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 50u128.into(),
        },
    ];
    let expected_to_remain = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 75u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 150u128.into(),
        },
    ];
    assert_eq!(coins_to_send, expected_to_send);
    assert_eq!(coins_to_remain, expected_to_remain);

    // test rounding //
    let coins = vec![Coin {
        denom: "ujuno".to_string(),
        amount: 10u128.into(),
    }];
    let (coins_to_send, coins_to_remain) =
        partition_coins_by_percentage(Decimal::percent(33), coins);
    let expected_to_send = vec![Coin {
        denom: "ujuno".to_string(),
        amount: 3u128.into(),
    }];
    let expected_to_remain = vec![Coin {
        denom: "ujuno".to_string(),
        amount: 7u128.into(),
    }];
    assert_eq!(coins_to_send, expected_to_send);
    assert_eq!(coins_to_remain, expected_to_remain);
}

#[test]
pub fn split_rewards_test() {
    let rewards = vec![
        Coin {
            denom: "ujuno".to_string(),
            amount: 100u128.into(),
        },
        Coin {
            denom: "uosmo".to_string(),
            amount: 200u128.into(),
        },
    ];
    let sim_rewards = split_rewards(rewards, Decimal::percent(5), &Some(Decimal::percent(5)));
    let expected_sim_rewards = SimulateExecuteResponse {
        delegator_rewards: vec![
            Coin {
                denom: "ujuno".to_string(),
                amount: 95u128.into(),
            },
            Coin {
                denom: "uosmo".to_string(),
                amount: 190u128.into(),
            },
        ],
        taxation_address_rewards: vec![
            Coin {
                denom: "ujuno".to_string(),
                amount: 5u128.into(),
            },
            Coin {
                denom: "uosmo".to_string(),
                amount: 10u128.into(),
            },
        ],
    };
    assert_eq!(sim_rewards, expected_sim_rewards);

    // test rounding //
    let rewards = vec![Coin {
        denom: "ujuno".to_string(),
        amount: 10u128.into(),
    }];
    let sim_rewards = split_rewards(
        rewards.clone(),
        Decimal::percent(33),
        &Some(Decimal::percent(33)),
    );
    let expected_sim_rewards = SimulateExecuteResponse {
        delegator_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 7u128.into(),
        }],
        taxation_address_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 3u128.into(),
        }],
    };
    assert_eq!(sim_rewards, expected_sim_rewards);

    // test falling back to the max percentage //
    let sim_rewards = split_rewards(rewards.clone(), Decimal::percent(33), &None);
    let expected_sim_rewards = SimulateExecuteResponse {
        delegator_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 7u128.into(),
        }],
        taxation_address_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 3u128.into(),
        }],
    };
    assert_eq!(sim_rewards, expected_sim_rewards);

    // test using a percentage that's less than the maxiumum
    let sim_rewards = split_rewards(
        rewards.clone(),
        Decimal::percent(33),
        &Some(Decimal::percent(10)),
    );
    let expected_sim_rewards = SimulateExecuteResponse {
        delegator_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 9u128.into(),
        }],
        taxation_address_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 1u128.into(),
        }],
    };
    assert_eq!(sim_rewards, expected_sim_rewards);

    // test using a percentage that's greater than the maxiumum
    let sim_rewards = split_rewards(rewards, Decimal::percent(33), &Some(Decimal::percent(50)));
    let expected_sim_rewards = SimulateExecuteResponse {
        delegator_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 7u128.into(),
        }],
        taxation_address_rewards: vec![Coin {
            denom: "ujuno".to_string(),
            amount: 3u128.into(),
        }],
    };
    assert_eq!(sim_rewards, expected_sim_rewards);
}

#[test]
fn withdraw_to_contract_msgs() {
    let contract_addr = Addr::unchecked("contract");

    let granter_addr = Addr::unchecked("granter");
    let validator1 = "validator1".to_string();

    let generated_msg = create_withdraw_rewards_exec_msg(
        &granter_addr,
        &contract_addr,
        &[PendingReward {
            amount: vec![Coin {
                denom: "ujuno".to_string(),
                amount: 100u128.into(),
            }],
            validator: validator1.to_string(),
        }],
    )
    .unwrap();

    assert_eq!(
        generated_msg,
        CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
            value: Binary::from(
                MsgExec {
                    grantee: contract_addr.to_string(),
                    msgs: vec![
                        MsgSetWithdrawAddress {
                            delegator_address: granter_addr.to_string(),
                            withdraw_address: contract_addr.to_string(),
                        }
                        .to_any()
                        .unwrap(),
                        MsgWithdrawDelegatorReward {
                            delegator_address: granter_addr.to_string(),
                            validator_address: validator1,
                        }
                        .to_any()
                        .unwrap(),
                        MsgSetWithdrawAddress {
                            delegator_address: granter_addr.to_string(),
                            withdraw_address: granter_addr.to_string(),
                        }
                        .to_any()
                        .unwrap()
                    ]
                }
                .encode_to_vec()
            ),
        }
    )
}

#[test]
fn gen_reward_withdrawl_msgs() {
    let contract_addr = Addr::unchecked("contract");
    let grantee_addr = Addr::unchecked("grantee");
    let granter_addr = Addr::unchecked("granter");
    let take_rate_addr = Addr::unchecked("take_rate");
    let validator1 = "validator1".to_string();

    // test the generate_rewards_withdrawl_msgs function
    let generated_msgs = generate_reward_withdrawl_msgs(
        AllPendingRewards {
            rewards: vec![PendingReward {
                amount: vec![Coin {
                    denom: "ujuno".to_string(),
                    amount: 100u128.into(),
                }],
                validator: validator1.to_string(),
            }],
            total: vec![Coin {
                denom: "ujuno".to_string(),
                amount: 100u128.into(),
            }],
        },
        AllowedWithdrawlSettings {
            grantee: grantee_addr.to_string(),
            taxation_address: take_rate_addr.to_string(),
            max_fee_percentage: Decimal::percent(15),
            expiration: Timestamp::from_seconds(1000),
        },
        &grantee_addr,
        &contract_addr,
        &granter_addr,
        None,
    )
    .unwrap();

    let expected_msgs = RewardExecutionMsgs {
        msgs: vec![
            exec_msg(
                &contract_addr,
                vec![
                    MsgSetWithdrawAddress {
                        delegator_address: granter_addr.to_string(),
                        withdraw_address: contract_addr.to_string(),
                    }
                    .to_any()
                    .unwrap(),
                    MsgWithdrawDelegatorReward {
                        validator_address: validator1,
                        delegator_address: granter_addr.to_string(),
                    }
                    .to_any()
                    .unwrap(),
                    MsgSetWithdrawAddress {
                        delegator_address: granter_addr.to_string(),
                        withdraw_address: granter_addr.to_string(),
                    }
                    .to_any()
                    .unwrap(),
                ],
            )
            .unwrap(),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: take_rate_addr.to_string(),
                amount: vec![Coin {
                    denom: "ujuno".to_string(),
                    amount: 15u128.into(),
                }],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: granter_addr.to_string(),
                amount: vec![Coin {
                    denom: "ujuno".to_string(),
                    amount: 85u128.into(),
                }],
            }),
        ],
        grantee: grantee_addr.to_string(),
    };

    assert_eq!(generated_msgs, expected_msgs);
}
