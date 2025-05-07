use std::{str::FromStr, sync::Arc, time::Duration};

use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::Config;
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use ldk_node::Builder;

use log::LevelFilter;           // <-- comes from the `log` crate

fn main() {
    // ── 1. Initialise a stdout logger for the whole process ────────────────
    env_logger::Builder::new()
    // quiet by default
		.filter_level(log::LevelFilter::Warn)

		// let channel + HTLC messages through
		.filter_module("ldk_node::liquidity",     log::LevelFilter::Info)
		.filter_module("ldk_node::payment",       log::LevelFilter::Info)
		.filter_module("ldk_node::chain",         log::LevelFilter::Info)
		.filter_module("ldk_node::peer_manager",  log::LevelFilter::Info)
		.filter_module("ldk_node::node",          log::LevelFilter::Info)
		.init();

    // ── 2. Basic node config ───────────────────────────────────────────────
    let mut config = Config::default();
    config.network = Network::Signet;

    let mut builder = Builder::from_config(config);
    builder
        .set_storage_dir_path("/tmp/ldk_node_liquidity_poc".into())
        .set_log_facade_logger()                            // <-- no arg now
        .set_chain_source_esplora("https://mutinynet.com/api/".into(), None);

    // ── 3. Tell the node which LSPS2 provider to trust for 0-conf channels ─
    let lsp_node_id = PublicKey::from_str(
        "02d71bd10286058cfb8c983f761c069a549d822ca3eb4a4c67d15aa8bec7483251",
    )
    .expect("invalid LSP node id");
    let lsp_address = "143.198.63.18:9735".parse().unwrap();

    builder.set_liquidity_source_lsps2(
        lsp_node_id,
        lsp_address,
        Some("this-token-is-not-currently-used".to_string()),
    );

    // ── 4. Build and start ─────────────────────────────────────────────────
    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    // print every event so you can see the channel-open handshake
    let event_node = Arc::clone(&node);
    std::thread::spawn(move || loop {
        let ev = event_node.wait_next_event();
        println!("EVENT ▶ {:?}", ev);
        let _ = event_node.event_handled();
    });

    // ── 5. Create a JIT-channel invoice (so the LSP opens a channel to us) ─
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".to_string()).unwrap(),
    );

    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &desc, 3_600, None)
        .unwrap();

    println!("JIT INVOICE: {invoice}");

    // ── 6. Keep running until you hit Ctrl-C ───────────────────────────────
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}