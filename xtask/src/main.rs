use std::env;
use std::error::Error;

use milner_xtask::context_pack::{ContextPackConfig, generate_context_pack};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "context-pack".to_string());
    let action = args.next().unwrap_or_else(|| "generate".to_string());

    match (command.as_str(), action.as_str()) {
        ("context-pack", "generate") => {
            let root = env::current_dir()?;
            let pack = generate_context_pack(&ContextPackConfig::new(root))?;
            pack.write_to_disk()?;
            println!(
                "generated Milner context pack with {} sources",
                pack.manifest.sources.len()
            );
            Ok(())
        }
        ("context-pack", "validate") => {
            let root = env::current_dir()?;
            let pack = generate_context_pack(&ContextPackConfig::new(root))?;
            milner_xtask::context_pack::validate_context_pack(&pack)?;
            println!(
                "validated Milner context pack with {} sources",
                pack.manifest.sources.len()
            );
            Ok(())
        }
        _ => Err(format!("unsupported xtask command `{command} {action}`").into()),
    }
}
