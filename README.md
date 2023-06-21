# Authzpp

The purpose of this repo is to provide a set of contracts that can be used to extend the Cosmos SDK's [Authz](https://docs.cosmos.network/main/modules/authz) module. The contracts should be able to further constrain the actions that can be authorized by an account while being secure and maintained by the community. This should serve to encourage the proliferation of non-custodial apps that can be used accross the interchain wherever cosmwasm is supported.

## Protocol Usage

The expected flow should go as such:

1. A user creates one or more native Authz grant(s) (`MsgGrant`) with the Authzpp contract as it's grantee.
2. The user executes the `Grant` method of the Authzpp contract setting the grantee to be the wallet they expect to use the permission on their behalf.
3. The grantee address can now execute the `Execute` method of the Authzpp contract to execute the action on behalf of the granter.

For more concrete examples, see the READMEs of the individual contracts.

## Progress

| Contract                                                         | Description                                                                                                                                       | Status      |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| [withdraw_rewards_tax](contracts/withdraw_rewards_tax/README.md) | A contract that allows a delegators funds to be withdrawn by a grantee and to have a portion of the rewards to be split to a third party address. | In Progress |
| [allowlist_send](contracts/allowlist_send/README.md)             | A contract that allows a granter to specify a list of addresses that can be sent to.                                                              | In Progress |
| [allowed_denoms_send](contracts/allowed_denoms_send/README.md)   | A contract that allows a granter to specify a list of denoms that can be sent by the grantee.                                                     | In Progress |
| [combine_grant](contracts/combine_grant/README.md)               | A contract that allows a granter to combine multiple grants into a single grant.                                                                  | In Progress |

## Packages
