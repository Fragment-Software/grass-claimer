use solana_client::rpc_client::SerializableTransaction;

pub fn encode_transaction_to_bs58(tx: &impl SerializableTransaction) -> String {
    let serialized_tx = bincode::serialize(&tx).unwrap();
    solana_sdk::bs58::encode(serialized_tx).into_string()
}
