use std::{str::FromStr, sync::Arc, time::Duration};
use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::{AnchorChannelsConfig, Config};
use ldk_node::logger::LogLevel;
use ldk_node::Builder;

pub fn setup_node(storage_dir: String, log_path: String) -> Arc<ldk_node::Node> {
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
        .set_storage_dir_path(storage_dir)
        .set_filesystem_logger(Some(log_path), Some(LogLevel::Debug))
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

    node
}