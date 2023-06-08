use cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec;
use cosmos_sdk_proto::traits::Message;
use cosmos_sdk_proto::{prost::EncodeError, Any};
use cosmwasm_std::{Addr, Binary, CosmosMsg};

/// Creates a MsgExec message
pub fn exec_msg(grantee: &Addr, any_msgs: Vec<Any>) -> Result<CosmosMsg, EncodeError> {
    let exec = MsgExec {
        grantee: grantee.to_string(),
        msgs: any_msgs,
    };

    Ok(CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
        value: Binary::from(exec.encode_to_vec()),
    })
}
