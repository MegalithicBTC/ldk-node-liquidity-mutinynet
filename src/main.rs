use std::{str::FromStr, sync::Arc, time::Duration};

use crossbeam_channel::{bounded, Receiver};
use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::Config;
use ldk_node::event::Event;                 // <─ event enum lives here
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use ldk_node::Builder;
use log::LevelFilter;

fn main() {
    // ── logging ───────────────────────────────────────────────────────────
    env_logger::Builder::new()
        .filter_level(LevelFilter::Warn)
        .filter_module("ldk_node::liquidity", LevelFilter::Info)
        .filter_module("ldk_node::payment",   LevelFilter::Info)
        .filter_module("ldk_node::chain",     LevelFilter::Info)
        .init();

    // ── config & builder ───────────────────────────────────────────────────
    let mut cfg = Config::default();
    cfg.network = Network::Signet;

    let mut builder = Builder::from_config(cfg);
    builder
        .set_storage_dir_path("/tmp/ldk_node_liquidity_poc")
        .set_log_facade_logger()
        .set_chain_source_esplora("https://mutinynet.com/api/", None);

    let lsp_id = PublicKey::from_str(
        "02d71bd10286058cfb8c983f761c069a549d822ca3eb4a4c67d15aa8bec7483251",
    )
    .unwrap();
    builder.set_liquidity_source_lsps2(
        lsp_id,
        "143.198.63.18:9735".parse().unwrap(),
        None,
    );

    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    // ── ctrl-c handler ─────────────────────────────────────────────────────
    {
        let n = Arc::clone(&node);
        ctrlc::set_handler(move || {
            eprintln!("\nCTRL-C → shutting down …");
            let _ = n.stop();
            std::process::exit(0);
        })
        .unwrap();
    }

    // ──  event pump → channel  ─────────────────────────────────────────────
    let (tx, rx) = bounded::<Event>(128);
    {
        let n = Arc::clone(&node);
        std::thread::spawn(move || loop {
            let ev = n.wait_next_event();
            let _ = tx.send(ev);
        });
    }

    wait_for_initial_sync(&rx);

    // ── create invoice only after sync complete ────────────────────────────
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".into()).unwrap(),
    );
    let inv = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &desc, 3_600, None)
        .unwrap();
    println!("READY → JIT INVOICE: {inv}");

    // keep alive
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}

/// Blocks until we have seen both the on-chain wallet sync and gossip sync.
fn wait_for_initial_sync(rx: &Receiver<Event>) {
    let mut onchain_done = false;
    let mut gossip_done  = false;

    println!("Waiting for first wallet / gossip sync to finish …");
    while let Ok(ev) = rx.recv() {
        match ev {
            Event::OnchainSyncCompleted { .. } => {
                onchain_done = true;
                println!("✓ on-chain wallet sync completed");
            }
            Event::GossipSyncCompleted { .. } => {
                gossip_done = true;
                println!("✓ gossip sync completed");
            }
            _ => { /* ignore other events for now */ }
        }
        if onchain_done && gossip_done {
            break;
        }
    }
}