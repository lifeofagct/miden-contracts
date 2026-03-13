use rand::RngCore;
use std::sync::Arc;
use std::collections::HashMap;
use miden_client::{
    account::{AccountBuilder, AccountStorageMode, AccountType, StorageSlot, StorageSlotName},
    builder::ClientBuilder,
    keystore::FilesystemKeyStore,
    rpc::{Endpoint, GrpcClient},
    transaction::{TransactionRequestBuilder, TransactionScript},
    ClientError, Felt,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;

fn read_slots(dbg: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let mut search = dbg;
    while let Some(st) = search.find("name: \"") {
        let ns = st + 7;
        let ne = search[ns..].find("\"").unwrap() + ns;
        let name = search[ns..ne].to_string();
        let rest = &search[ne..];
        if let Some(vs) = rest.find("content: Value(Word([") {
            let abs = vs + 21;
            let end = rest[abs..].find("])").unwrap() + abs;
            let nums: Vec<u64> = rest[abs..end].split(",")
                .filter_map(|s| s.trim().parse().ok()).collect();
            map.insert(name, *nums.last().unwrap_or(&0));
        }
        search = &search[ne..];
    }
    map
}

fn make_script(vals: &[u64; 4]) -> String {
    let hex: String = vals.iter().flat_map(|v| v.to_le_bytes()).map(|b| format!("{:02x}", b)).collect();
    format!("begin\n    call.0x{}\nend", hex)
}

#[tokio::main]
async fn main() -> Result<(), miden_client::ClientError> {
    let endpoint = miden_client::rpc::Endpoint::new("https".to_string(), "rpc.testnet.miden.io".to_string(), Some(443));
    let rpc_client = std::sync::Arc::new(miden_client::rpc::GrpcClient::new(&endpoint, 10_000));
    let keystore = std::sync::Arc::new(miden_client::keystore::FilesystemKeyStore::new(std::path::PathBuf::from("./keystore")).unwrap());
    let mut client = miden_client::builder::ClientBuilder::new()
        .rpc(rpc_client)
        .sqlite_store(std::path::PathBuf::from("./store.sqlite3"))
        .authenticator(keystore.clone())
        .build().await?;
    client.sync_state().await?;
    let code = std::fs::read_to_string("masm/accounts/auction.masm").unwrap();
    let cc = miden_client::assembly::CodeBuilder::new()
        .compile_component_code("external_contract::auction_contract", &code).unwrap();
    let comp = miden_client::account::AccountComponent::new(cc, vec![
        miden_client::account::StorageSlot::with_value(miden_client::account::StorageSlotName::new("miden::auction::highest_bid").unwrap(), [miden_client::Felt::new(0);4].into()),
        miden_client::account::StorageSlot::with_value(miden_client::account::StorageSlotName::new("miden::auction::total_bids").unwrap(), [miden_client::Felt::new(0);4].into()),
    ]).unwrap().with_supports_all_types();
    let mut seed = [0_u8; 32];
    use rand::RngCore;
    client.rng().fill_bytes(&mut seed);
    let contract = miden_client::account::AccountBuilder::new(seed)
        .account_type(miden_client::account::AccountType::RegularAccountImmutableCode)
        .storage_mode(miden_client::account::AccountStorageMode::Public)
        .with_component(comp)
        .with_auth_component(miden_client::auth::NoAuth)
        .build().unwrap();
    client.add_account(&contract, false).await.unwrap();
    let id = contract.id();
    println!("Deployed! {}", id.to_bech32(miden_client::address::NetworkId::Testnet));
    let procs: Vec<[u64;4]> = contract.code().procedures().iter().skip(1).map(|p| {
        let d = format!("{:?}", p);
        let i = &d[d.find("Word([").unwrap()+6..d.find("])").unwrap()];
        let v: Vec<u64> = i.split(",").filter_map(|s| s.trim().parse().ok()).collect();
        [v[0], v[1], v[2], v[3]]
    }).collect();
    println!("\nAuction: 4 bids - 10, 25, 10, 50. Winner should be 50.");
    let actions: Vec<(usize, &str)> = vec![
        (0, "Alice   bids 10 MID (new high!)"),
        (1, "Bob     bids 25 MID (new high!)"),
        (0, "Alice   bids 10 MID (too low, ignored)"),
        (2, "Charlie bids 50 MID (new high! winner)"),
    ];
    for (idx, label) in &actions {
        println!("  -> {}...", label);
        let hex: String = procs[*idx].iter().flat_map(|v| v.to_le_bytes()).map(|b| format!("{:02x}", b)).collect();
        let script = format!("begin\n    call.0x{}\nend", hex);
        let prog = miden_client::assembly::Assembler::default().assemble_program(&script).unwrap();
        let req = miden_client::transaction::TransactionRequestBuilder::new()
            .custom_script(miden_client::transaction::TransactionScript::new(prog)).build().unwrap();
        match client.submit_new_transaction(id, req).await {
            Ok(_) => { client.sync_state().await?; println!("     OK"); }
            Err(e) => println!("     FAILED: {}", e),
        }
    }
    let rec = client.get_account(id).await?.unwrap();
    let slots = read_slots(&format!("{:?}", rec.account_data()));
    let highest = slots.get("miden::auction::highest_bid").unwrap_or(&0);
    let total = slots.get("miden::auction::total_bids").unwrap_or(&0);
    println!("\nAuction Results:");
    println!("  Highest Bid: {} MID", highest);
    println!("  Total Bids:  {}", total);
    if *highest == 50 { println!("  Winner:      Charlie! (50 MID)"); }
    println!("  Expected:    highest=50 total=4");
    Ok(())
}
