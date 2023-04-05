use schemars::{schema::RootSchema, schema_for};

#[path = "./message.rs"]
mod message;

use message::{ToClient, ToServer};

fn main() -> std::io::Result<()> {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let e = std::fs::DirBuilder::new().create(&dir);
    if let Err(e) = e {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(e);
        }
    }

    let schema = schema_for!(ToClient);
    write_schema(&dir, "to_client", &schema)?;

    let schema = schema_for!(ToServer);
    write_schema(&dir, "to_server", &schema)?;
    println!("Wrote schemas to {}", dir.to_string_lossy());

    Ok(())
}

fn write_schema(dir: &std::path::Path, name: &str, schema: &RootSchema) -> std::io::Result<()> {
    let output = serde_json::to_string_pretty(schema).unwrap();
    let output_path = dir.join(format!("{}.json", name));
    std::fs::write(output_path, output)
}
