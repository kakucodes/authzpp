use cosmrs::tx::MessageExt;
use cosmwasm_std::to_binary;
use cw_orch::{
    anyhow,
    daemon::DaemonBuilder,
    prelude::{Stargate, *},
};
use tokio::runtime::Runtime;

use anybuf::Anybuf;

pub fn main() -> anyhow::Result<()> {
    let rt = Runtime::new().unwrap();
    dotenv::dotenv().ok();
    env_logger::init();

    let chain_config = networks::JUNO_1;

    let chain_daemon = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(chain_config.clone())
        .build()?;

    println!("connected to juno with sender: {}", chain_daemon.sender());

    let withdraw_tax = withdraw_rewards_tax_grant::WithdrawRewardsTaxGrant::new(
        "withdraw_rewards_tax_grant",
        chain_daemon.clone(),
    );

    withdraw_tax.upload_if_needed()?;

    println!("withdraw tax grant codeid: {}", withdraw_tax.code_id()?);

    if withdraw_tax.address().is_err() {
        withdraw_tax.instantiate(
            &withdraw_rewards_tax_grant::msg::InstantiateMsg {},
            Some(&Addr::unchecked(chain_daemon.sender().to_string())),
            None,
        )?;

        // this seems to sometimes need an increased gas multiplier in the .env to work
        chain_daemon
            .commit_any::<cosmrs::Any>(
                vec![feeshare_msg(
                    withdraw_tax.address().unwrap().to_string(),
                    chain_daemon.sender().to_string(),
                    chain_daemon.sender().to_string(),
                )],
                None,
            )
            .unwrap();
    } else {
        withdraw_tax.migrate_if_needed(&withdraw_rewards_tax_grant::msg::InstantiateMsg {})?;
    }

    println!("withdraw tax grant: {}", withdraw_tax.addr_str()?);

    Ok(())
}

pub fn feeshare_msg(
    contract_address: String,
    deployer_address: String,
    withdrawer_address: String,
) -> cosmrs::Any {
    cosmrs::Any {
        type_url: "/juno.feeshare.v1.MsgRegisterFeeShare".to_string(),
        value: Anybuf::new()
            .append_string(1, contract_address)
            .append_string(2, deployer_address)
            .append_string(3, withdrawer_address)
            .into_vec(),
    }
}

// #[derive(Clone, PartialEq, ::prost::Message)]
// pub struct MsgRegisterFeeShare {
//     #[prost(string, tag = "1")]
//     pub contract_address: ::prost::alloc::string::String,
//     /// Unique ID number for this person.
//     #[prost(string, tag = "2")]
//     pub deployer_address: ::prost::alloc::string::String,
//     #[prost(string, tag = "3")]
//     pub withdrawer_address: ::prost::alloc::string::String,
// }

// impl MessageExt for MsgRegisterFeeShare {
//     fn to_any(&self) -> cosmrs::Any {
//         let mut any = cosmrs::Any {
//             type_url: "/juno.feeshare.v1.MsgRegisterFeeShare".to_string(),
//             value: vec![],
//         };
//         let mut buf = Vec::new();
//         prost::Message::encode(self, &mut buf).unwrap();
//         any.value = buf;
//         any
//     }

//     fn to_bytes(&self) -> Result<Vec<u8>, cosmrs::proto::prost::EncodeError> {
//         todo!()
//     }
// }
