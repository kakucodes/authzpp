use cosmwasm_std::{Coin, Decimal};

use crate::helpers::{partition_coins_by_percentage, sum_coins};

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
