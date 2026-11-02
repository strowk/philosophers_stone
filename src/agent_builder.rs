use std::path::PathBuf;

use laya::RLAgent;

pub(crate) fn get_agent() -> anyhow::Result<RLAgent> {
    download_model_to_cache()?;
    RLAgent::load(get_cache_folder())
}

fn get_cache_folder() -> PathBuf {
    let dir = dirs::cache_dir().expect("no cache dir");
    dir.join("philosopher_stone").join("laya-model-cache")
}

fn download_model_to_cache() -> anyhow::Result<()> {
    let dir = get_cache_folder();

    for file in [
        "rl_agent_config.json",
        "encoder/config.json",
        "tokenizer/tokenizer.json",
        "tokenizer/tokenizer_config.json",
        "model.safetensors",
    ] {
        println!("Getting model file {file}");
        let target = dir.join(file);
        if target.is_dir() {
            anyhow::bail!("target is directory");
        }
        if target.is_file() {
            println!("File already found");
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(&parent)?;
        }
        let partial_out = target.with_extension("part");
        let mut response = ureq::get(format!("https://huggingface.co/convaiinnovations/laya-typed-decisions/resolve/main/{file}")).call()?;
        let mut out = std::fs::File::create(&partial_out)?;
        let body = response.body_mut();
        let mut reader = body.as_reader();
        std::io::copy(&mut reader, &mut out)?;
        std::fs::rename(&partial_out, &target)?;
    }

    Ok(())
}
