use anyhow::Result;
use vergen::{vergen, Config};

fn main() -> Result<()> {
    let mut cfg = Config::default();
    *cfg.git_mut().sha_kind_mut() = vergen::ShaKind::Short;
    vergen(cfg)
}
