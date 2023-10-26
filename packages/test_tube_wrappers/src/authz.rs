use cosmos_sdk_proto::traits::Message;
use osmosis_std::shim::{Any, Timestamp};
use osmosis_std::types::cosmos::authz::v1beta1::{
    GenericAuthorization, Grant, MsgExec, MsgExecResponse, MsgGrant, MsgGrantResponse, MsgRevoke,
    MsgRevokeResponse,
};
use osmosis_std::types::cosmos::authz::v1beta1::{QueryGrantsRequest, QueryGrantsResponse};

use osmosis_std::types::cosmos::bank::v1beta1::SendAuthorization;
use osmosis_std::types::cosmos::base::v1beta1::Coin;

use osmosis_test_tube::{fn_execute, fn_query, RunnerExecuteResult, SigningAccount};
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
    // pub fn create_generic_grant(
    //     &self,
    //     grantee: String,
    //     granter: String,
    //     msg_type: String,
    //     expiration: Option<Timestamp>,
    //     signer: &SigningAccount,
    // ) -> RunnerExecuteResult<MsgGrantResponse> {
    //     self.runner.execute(
    //         MsgGrant {
    //             granter,
    //             grantee,
    //             grant: Some(Grant {
    //                 authorization: Some(Any {
    //                     type_url: GenericAuthorization::TYPE_URL.to_string(),
    //                     value: GenericAuthorization { msg: msg_type }.encode_to_vec(),
    //                 }),
    //                 expiration,
    //             }),
    //         },
    //         MsgGrant::TYPE_URL,
    //         signer,
    //     )
    // }

    // pub fn create_send_authorization(
    //     &self,
    //     grantee: String,
    //     granter: String,
    //     spend_limit: Vec<Coin>,
    //     expiration: Option<Timestamp>,
    //     signer: &SigningAccount,
    // ) -> RunnerExecuteResult<MsgGrantResponse> {
    //     self.runner.execute(
    //         MsgGrant {
    //             granter,
    //             grantee,
    //             grant: Some(Grant {
    //                 authorization: Some(Any {
    //                     type_url: SendAuthorization::TYPE_URL.to_string(),
    //                     value: SendAuthorization { spend_limit }.encode_to_vec(),
    //                 }),
    //                 expiration,
    //             }),
    //         },
    //         MsgGrant::TYPE_URL,
    //         signer,
    //     )
    // }

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
