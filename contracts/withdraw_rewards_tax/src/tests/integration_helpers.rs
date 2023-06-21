use osmosis_test_tube::{OsmosisTestApp, SigningAccount, Wasm};

use crate::msg::InstantiateMsg;

/// uploads the contract and returns the contract address
pub fn upload_contract(
    wasm: &Wasm<OsmosisTestApp>,
    wasm_path: &str,
    signer: &SigningAccount,
) -> String {
    let wasm_byte_code = std::fs::read(wasm_path).unwrap();

    // uploads the contract and returns the code id
    let code_id = wasm
        .store_code(&wasm_byte_code, None, signer)
        .unwrap()
        .data
        .code_id;

    // instantiates the contract and returns the generated address
    wasm.instantiate(
        code_id,
        &InstantiateMsg {},
        None,   // contract admin used for migration, not the same as cw1_whitelist admin
        None,   // contract label
        &[],    // funds
        signer, // signer
    )
    .unwrap()
    .data
    .address
}
