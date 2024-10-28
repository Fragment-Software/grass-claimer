use std::{fs, io::Write, path::Path};

fn main() {
    let data_path = Path::new("./data");
    let private_keys_path = data_path.join("private_keys.txt");
    let proxies_path = data_path.join("proxies.txt");
    let cex_addresses_path = data_path.join("cex_addresses.txt");
    let config_path = data_path.join("config.toml");

    if !data_path.exists() {
        fs::create_dir_all(data_path).unwrap();
    }

    if !private_keys_path.exists() {
        fs::File::create(&private_keys_path).unwrap();
    }

    if !proxies_path.exists() {
        fs::File::create(&proxies_path).unwrap();
    }

    if !cex_addresses_path.exists() {
        fs::File::create(&cex_addresses_path).unwrap();
    }

    if !config_path.exists() {
        let mut config_file = fs::File::create(&config_path).unwrap();
        let config_content = r#"SOLANA_RPC_URL = ""         # rpc url
USE_JITO = false            # send tx with jito bundle
JITO_BLOCK_ENGINE_URL = ""  # jito block engine url
WITHDRAW_TO_CEX = true      # withdraw allocation to external address
MOBILE_PROXIES = false      # whether you're using mobile proxies or not
SWAP_IP_LINK = ""           # if you're using mobile proxies put the change ip link in here
CLAIM_SLEEP_RANGE = [4, 10] # sleep range between each claim (seconds)
"#;
        config_file.write_all(config_content.as_bytes()).unwrap();
    }

    println!("cargo:rerun-if-changed=build.rs");
}
