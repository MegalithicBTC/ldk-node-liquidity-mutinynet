use std::{str::FromStr, sync::Arc, time::Duration};

use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::Config;
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use ldk_node::logger::LogLevel;
use ldk_node::Builder;

fn main() {
    // where we want the log
    let log_path = "./tmp/ldk_node.log".to_string();

    // basic config
    let mut cfg = Config::default();
    cfg.network = Network::Signet;

    let mut builder = Builder::from_config(cfg);
    builder
        .set_storage_dir_path("/tmp/ldk_node_liquidity_poc".to_string())
        .set_filesystem_logger(Some(log_path.clone()), Some(LogLevel::Info))
        .set_chain_source_esplora("https://mutinynet.com/api/".to_string(), None);

    // LSPS2 we trust for zero-conf channels
    let lsp_id = PublicKey::from_str(
        "02d71bd10286058cfb8c983f761c069a549d822ca3eb4a4c67d15aa8bec7483251",
    )
    .unwrap();
    builder.set_liquidity_source_lsps2(lsp_id, "143.198.63.18:9735".parse().unwrap(), None);

    // build & start
    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    // graceful Ctrl-C
    {
        let n = Arc::clone(&node);
        ctrlc::set_handler(move || {
            let _ = n.stop();
            std::process::exit(0);
        })
        .unwrap();
    }

    // create invoice right away
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".into()).unwrap(),
    );
    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &desc, 3_600, None)
        .unwrap();

    println!("JIT INVOICE:\n{invoice}\n");
    println!("Logs ↦ {log_path}");

    // keep alive so the LSP can actually open the channel
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}