use cw_orch::{anyhow, daemon::DaemonBuilder, prelude::*};
use tokio::runtime::Runtime;

pub fn main() -> anyhow::Result<()> {
    let rt = Runtime::new().unwrap();
    dotenv::dotenv().ok();
    env_logger::init();

    let juno = networks::JUNO_1;

    let juno_chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(juno.clone())
        .build()?;

    println!("connected to juno with sender: {}", juno_chain.sender());

    let withdraw_tax = withdraw_rewards_tax_grant::WithdrawRewardsTaxGrant::new(
        "withdraw_rewards_tax_grant",
        juno_chain.clone(),
    );

    withdraw_tax.upload_if_needed()?;

    println!("withdraw tax grant codeid: {}", withdraw_tax.code_id()?);

    // withdraw_tax
    //     .upload_and_migrate_if_needed(&withdraw_rewards_tax_grant::msg::InstantiateMsg {})?;

    if withdraw_tax.address().is_ok() && withdraw_tax.is_running_latest()? {
        println!("withdraw_tax is already latest, no need to reinstantiate");
    } else {
        withdraw_tax.instantiate(
            &withdraw_rewards_tax_grant::msg::InstantiateMsg {},
            None,
            None,
        )?;
    }

    println!("withdraw tax grant: {}", withdraw_tax.addr_str()?);

    Ok(())
}
