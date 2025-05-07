use std::io::{self, Read, Write};
use std::str::FromStr;
use std::sync::Arc;

use ldk_node::bitcoin::Network;
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::config::Config;
use ldk_node::logger::LogLevel;
use ldk_node::Builder;

use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};

fn main() {
    // --- basic node config --------------------------------------------------
    let mut config = Config::default();
    config.network = Network::Signet;

    let mut builder = Builder::from_config(config);
    builder
        .set_storage_dir_path("/tmp/ldk_node_liquidity_poc/".to_string())
        // TRACE-level logs to <storage_dir>/ldk_node.log
        .set_filesystem_logger(None, Some(LogLevel::Trace))
        //  └─ or use `.set_log_facade_logger()` + env_logger::init()
        .set_chain_source_esplora("https://mutinynet.com/api/".to_string(), None);

    // --- LSPS2 LSP ----------------------------------------------------------
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

    // --- build & start ------------------------------------------------------
    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    // print every LDK-node event
    let event_node = Arc::clone(&node);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        println!("GOT NEW EVENT: {:?}", event);
        let _ = event_node.event_handled();
    });

    // --- create a JIT-channel invoice --------------------------------------
    let description = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".to_string()).unwrap(),
    );

    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &description, 3_600, None)
        .unwrap();

    println!("GOT JIT INVOICE: {}", invoice);

    pause();
    node.stop().unwrap();
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // wait for one byte
    let _ = stdin.read(&mut [0u8]).unwrap();
}