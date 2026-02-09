// src/env_check.rs

use anyhow::{Context, Result, anyhow};
use openrgb2::OpenRgbClient;
use std::net::SocketAddr;

const OPENRGB_PROTOCOL_VERSION: u32 = 4; // OpenRGB 0.9+ uses Protocol v4

pub fn validate_dependencies(host: &str, port: u16) -> Result<()> {
    if let Err(e) = validate_openrgb_connection(host, port) {
        return Err(anyhow!(
            "Could not connect to OpenRGB SDK server at {}:{}: {}. \
            Ensure OpenRGB is running and the SDK Server is enabled (Protocol v{}).",
            host,
            port,
            e,
            OPENRGB_PROTOCOL_VERSION
        ));
    }

    Ok(())
}

fn validate_openrgb_connection(host: &str, port: u16) -> Result<()> {
    // Reuse the runtime from executor instead of creating a new one
    let rt = crate::executor::get_runtime();

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().context("Invalid Socket Address")?;

    rt.block_on(async {
        let client = OpenRgbClient::connect_to(addr, OPENRGB_PROTOCOL_VERSION)
            .await
            .context("Connection refused - is OpenRGB running with SDK server enabled?")?;

        // Test basic functionality
        let _count = client
            .get_controller_count()
            .await
            .context("Failed to query controller count")?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
