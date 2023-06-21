# Authzpp Allowed Denoms Send Grant

Contract that allows a granter to grant a grantee to send tokens to anyone so long as the tokens are in the allow list of denoms.

## Message Flow

1. Granter creates either a (GenericAuthorization)[https://docs.cosmos.network/main/modules/authz#genericauthorization] (`/cosmos.bank.v1beta1.MsgSend`) or (SendAuthorization)[https://docs.cosmos.network/main/modules/authz#sendauthorization] with the grantee set to the Authzpp Allowed Denoms Send contract address.

2. Granter executes the `Grant` method of the Authzpp Allowed Denoms Send contract with the grantee set to the wallet they expect to use the permission on their behalf.

3. Grantee can now execute the `Execute` method of the Authzpp Allowed Denoms Send contract to execute the action on behalf of the granter and send allowed denoms to any address until the expiration of the grant.
