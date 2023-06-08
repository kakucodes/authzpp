use osmosis_std::types::cosmos::authz::v1beta1::{
    MsgExec, MsgExecResponse, MsgGrant, MsgGrantResponse, MsgRevoke, MsgRevokeResponse,
};
use osmosis_std::types::cosmos::authz::v1beta1::{QueryGrantsRequest, QueryGrantsResponse};

use osmosis_test_tube::{fn_execute, fn_query};
use osmosis_test_tube::{Module, Runner};

// Boilerplate code, copy and rename should just do the trick
pub struct Authz<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Authz<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}
// End Boilerplate code

impl<'a, R> Authz<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub create_grant: MsgGrant => MsgGrantResponse
    }
    fn_execute! {
        pub execute_grant: MsgExec => MsgExecResponse
    }
    fn_execute! {
        pub revoke_grant: MsgRevoke => MsgRevokeResponse
    }

    fn_query! {
        pub query_grant ["/cosmos.authz.v1beta1.Query/Grants"]: QueryGrantsRequest => QueryGrantsResponse
    }
}
