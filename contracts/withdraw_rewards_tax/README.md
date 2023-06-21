# Authzpp Withdraw Rewards Tax Grant

Contract that allows a granter to grant a grantee the ability to withdraw the granter's rewards (to the granter's wallet) but to have some percentage of those rewards going to a specified 3rd party wallet/address. This is useful for delegators who want to share their rewards with a service that is making use of their staking rewards for them.

## Message Flow

1. Granter/delegator creates a pair of (GenericAuthorizations)[https://docs.cosmos.network/main/modules/authz#genericauthorization] for both `/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward` and `/cosmos.distribution.v1beta1.MsgSetWithdrawAddress`, both with the grantee set to the Authzpp Withdraw Rewards Tax contract address.

2. Granter executes the `Grant` method of the Authzpp Withdraw Rewards Tax contract with the grantee set to the wallet they expect to use the permission on their behalf and the 3rd party address to receive the rewards.

   - Note that a granter may only have one grant active to the withdraw rewards tax contract at any one point in time and re-issuing a grant to a different grantee will overwrite the previous grant.

3. Grantee can now execute the `Execute` method of the Authzpp Withdraw Rewards Tax contract to execute the action on behalf of the granter and withdraw the granter's rewards to the granter's wallet but with a portion of the rewards going to the 3rd party address until the expiration of the grant.

## Contract Parameters

### Contract Grant Settings (AllowedWithdrawlSettings)

- `grantee`: The address of the grantee that will be executing the action on behalf of the granter.
- `taxation_address`: The address that will receive the portion of the rewards.
- `max_fee_percentage`: The maximum percentage of the rewards that can be taken as a fee. This is to set an upper limit but allow less to be taken if the grantee is so incline.
- `expiration`: The expiration time of the grant.

### Contract Queries

- `ActiveGrantsByDelegator`

  - Parameter `delegator`/string: The address of the delegator/granter.
  - Returns `Option<GrantQueryResponse>`

- `ActiveGrantsByGrantee`

  - Parameter `grantee`/string: The address of the grantee.
  - Returns `Vec<GrantQueryResponse>`

- `SimulateExecute`
  - Parameter `delegator`/string: The address of the grantee/grantee.
  - Parameter `percentage`/Option<Decimal>: The percentage to take.
  - Returns `SimulateExecuteResponse`
