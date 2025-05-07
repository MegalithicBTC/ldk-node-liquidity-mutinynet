use std::{str::FromStr, sync::Arc, time::Duration};

use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::{AnchorChannelsConfig, Config};
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use ldk_node::logger::LogLevel;
use ldk_node::Builder;

fn main() {
    // ── paths ──────────────────────────────────────────────────────────────
    let storage_dir = "tmp".to_string();   // project-local
    let log_path    = format!("{}/ldk_node.log", storage_dir);

    // ── base config ────────────────────────────────────────────────────────
    let mut cfg = Config::default();
    cfg.network = Network::Signet;

    // LSPS2 peer we absolutely trust (no local reserve for anchor channels)
    let lsp_pubkey = PublicKey::from_str(
        "02d71bd10286058cfb8c983f761c069a549d822ca3eb4a4c67d15aa8bec7483251",
    )
    .expect("invalid pubkey");

    let mut anchor_cfg = AnchorChannelsConfig::default();
    anchor_cfg.trusted_peers_no_reserve.push(lsp_pubkey);
    cfg.anchor_channels_config = Some(anchor_cfg);

    // ── builder ────────────────────────────────────────────────────────────
    let mut builder = Builder::from_config(cfg);
    builder
        .set_storage_dir_path(storage_dir.clone())
        .set_filesystem_logger(Some(log_path.clone()), Some(LogLevel::Debug))
        .set_chain_source_esplora("https://mutinynet.com/api/".to_string(), None)
        .set_liquidity_source_lsps2(
            lsp_pubkey,
            "143.198.63.18:9735".parse().unwrap(),
            None,
        );

    // ── build & start ──────────────────────────────────────────────────────
    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    // Ctrl-C → clean shutdown
    {
        let n = Arc::clone(&node);
        ctrlc::set_handler(move || {
            let _ = n.stop();
            std::process::exit(0);
        })
        .unwrap();
    }

    // ── create invoice right away ──────────────────────────────────────────
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".into()).unwrap(),
    );
    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &desc, 3_600, None)
        .unwrap();

    println!("JIT INVOICE:\n{invoice}\n");
    println!("Logs ➜ {log_path}");

    // keep running so the LSP can actually open the channel
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}