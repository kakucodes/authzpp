# Authzpp Address Allowlist Send Grant

Contract that allows a granter authorize/grant a grantee to send tokens to anyone in a list of addresses.

## Message Flow

1. Granter creates either a (GenericAuthorization)[https://docs.cosmos.network/main/modules/authz#genericauthorization] (`/cosmos.bank.v1beta1.MsgSend`) or (SendAuthorization)[https://docs.cosmos.network/main/modules/authz#sendauthorization] with the grantee set to the Authzpp Allowlist Send contract address.

2. Granter executes the `Grant` method of the Authzpp Allowlist Send contract with the grantee set to the wallet they expect to use the permission on their behalf.

3. Grantee can now execute the `Execute` method of the Authzpp Allowlist Send contract to execute the action on behalf of the granter and send tokens from the grantee's wallets to any of the allowed addresses until the expiration of the grant.
