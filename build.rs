use anyhow::Result;
use vergen::{vergen, Config, TimestampKind};

fn main() -> Result<()> {
    let mut cfg = Config::default();
    *cfg.git_mut().sha_kind_mut() = vergen::ShaKind::Short;
    *cfg.build_mut().kind_mut() = TimestampKind::All;
    vergen(cfg)
}
