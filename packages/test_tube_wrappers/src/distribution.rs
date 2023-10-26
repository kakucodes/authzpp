use osmosis_test_tube::cosmrs::proto::cosmos::distribution::v1beta1::{
    QueryDelegationTotalRewardsRequest, QueryDelegationTotalRewardsResponse,
    QueryDelegatorWithdrawAddressRequest, QueryDelegatorWithdrawAddressResponse,
};
use osmosis_test_tube::fn_query;
use osmosis_test_tube::{Module, Runner};

// Boilerplate code, copy and rename should just do the trick
pub struct Distribution<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Distribution<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}
// End Boilerplate code

impl<'a, R> Distribution<'a, R>
where
    R: Runner<'a>,
{
    // fn_execute! {
    //     pub widthdraw_delegator_reward: MsgWithdrawDelegatorReward => MsgWithdrawDelegatorRewardResponse
    // }
    // fn_execute! {
    //     pub set_widthdraw_address: MsgSetWithdrawAddress => MsgSetWtihdrawAddressResponse
    // }
    //msgwithdrawvalidatorcommission
    //msgfundcommunitypool

    // macro for creating query function
    // fn_query! {
    //     pub query_delegator_withdraw_address ["/cosmos.distribution.v1beta1.Query/DelegatorWithdrawAddress"]: QueryDelegatorWithdrawAddressRequest => QueryDelegatorWithdrawAddressResponse
    // }
    // fn_query! {
    //     pub query_delegation_total_rewards ["/cosmos.distribution.v1beta1.Query/DelegationTotalRewards"]: QueryDelegationTotalRewardsRequest => QueryDelegationTotalRewardsResponse
    // }
}
