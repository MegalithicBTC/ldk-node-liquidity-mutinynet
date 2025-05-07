use std::{str::FromStr, sync::Arc, time::Duration};

use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::Config;
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use ldk_node::Builder;
use log::LevelFilter;

fn main() {
    // ── stdout logger ──────────────────────────────────────────────────────
    env_logger::Builder::new()
        .filter_level(LevelFilter::Warn)
        .filter_module("ldk_node::liquidity", LevelFilter::Info)
        .filter_module("ldk_node::payment",   LevelFilter::Info)
        .filter_module("ldk_node::chain",     LevelFilter::Info)
        .filter_module("ldk_node::peer_manager", LevelFilter::Info)
        .filter_module("ldk_node::node",      LevelFilter::Info)
        .init();

    // ── node config ────────────────────────────────────────────────────────
    let mut config = Config::default();
    config.network = Network::Signet;

    let mut builder = Builder::from_config(config);
    builder
        .set_storage_dir_path("/tmp/ldk_node_liquidity_poc".into())
        .set_log_facade_logger()
        .set_chain_source_esplora("https://mutinynet.com/api/".into(), None);

    // LSPS2 provider we trust for zero-conf channels
    let lsp_node_id = PublicKey::from_str(
        "02d71bd10286058cfb8c983f761c069a549d822ca3eb4a4c67d15aa8bec7483251",
    )
    .unwrap();
    let lsp_address = "143.198.63.18:9735".parse().unwrap();

    builder.set_liquidity_source_lsps2(
        lsp_node_id,
        lsp_address,
        Some("this-token-is-not-currently-used".to_string()),
    );

    // ── build & start ───────────────────────────────────────────────────────
    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    // graceful shutdown on Ctrl-C
    {
        let node = Arc::clone(&node);
        ctrlc::set_handler(move || {
            eprintln!("\nCTRL-C received → stopping node …");
            if let Err(e) = node.stop() {
                eprintln!("Failed to stop node cleanly: {:?}", e);
            }
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    // print every event
    {
        let event_node = Arc::clone(&node);
        std::thread::spawn(move || loop {
            let ev = event_node.wait_next_event();
            println!("EVENT ▶ {:?}", ev);
            let _ = event_node.event_handled();
        });
    }

    // create a JIT-channel invoice
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".to_string()).unwrap(),
    );
    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &desc, 3_600, None)
        .unwrap();
    println!("JIT INVOICE: {invoice}");

    // keep the main thread alive
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}