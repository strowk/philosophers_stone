use std::{io::Write, path::PathBuf};

use anyhow::Context;
use laya::RLAgent;

pub(crate) trait ProgressReporter {
    fn report(&mut self, progress_percent: u8, current_item: String);
}

struct OutputReporter {}

impl ProgressReporter for OutputReporter {
    fn report(&mut self, progress_percent: u8, current_item: String) {
        println!("{current_item}: {progress_percent}%")
    }
}

#[cfg(target_arch = "wasm32")]
struct ConsoleReporter {}

#[cfg(target_arch = "wasm32")]
impl ProgressReporter for ConsoleReporter {
    fn report(&mut self, progress_percent: u8, current_item: String) {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "{current_item}: {progress_percent}%"
        )))
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn get_agent_web(
    rl_agent_config_json: &str,
    tokenizer_config_json: &str,
    tokenizer_bytes: &[u8],
    encoder_config_json: &str,
    weights: &[u8],
) -> anyhow::Result<RLAgent> {
    Ok(RLAgent::load_from_bytes("", "", &[], "", &[])?)
}

// fn get_reqwest_client() {
//     let client = reqwest::ClientBuilder::new()..build();

// }

fn get_reqwest_reporter() -> impl ProgressReporter {
    #[cfg(target_arch = "wasm32")]
    {
        ConsoleReporter {}
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        OutputReporter {}
    }
}

async fn download_model_and_create_agent() -> anyhow::Result<RLAgent> {
    let client = reqwest::Client::new();

    let config_files = [
        "rl_agent_config.json",
        "tokenizer/tokenizer_config.json",
        "encoder/config.json",
    ];

    let mut res = vec![];

    for file in config_files {
        let progress_reporter = get_reqwest_reporter();
        let down = download_file_with_forced_cache(
            &client,
            &format!("https://huggingface.co/convaiinnovations/laya/resolve/main/{file}"),
            progress_reporter,
            file,
        )
        .await?;
        let down = down;
        let down = String::from_utf8(down)?;
        res.push(down);
    }

    let binary_files = ["tokenizer/tokenizer.json", "model.safetensors"];
    let mut tokenizer = None;
    let mut tensors = None;

    for file in binary_files {
        let progress_reporter = get_reqwest_reporter();
        let down = download_file_with_forced_cache(
            &client,
            &format!("https://huggingface.co/convaiinnovations/laya/resolve/main/{file}"),
            progress_reporter,
            file,
        )
        .await?;
        let down = down;
        match file {
            "tokenizer/tokenizer.json" => {
                tokenizer = Some(down);
            }
            "model.safetensors" => {
                tensors = Some(down);
            }
            _ => {
                panic!("not reachable");
            }
        }
    }

    Ok(RLAgent::load_from_bytes(
        res.get(0).unwrap(),
        res.get(1).unwrap(),
        &tokenizer.unwrap(),
        res.get(2).unwrap(),
        &tensors.unwrap(),
    )?)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_cache_folder() -> PathBuf {
    let dir = dirs::cache_dir().expect("no cache dir");
    dir.join("philosopher_stone").join("laya")
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys;

#[cfg(target_arch = "wasm32")]
fn js_to_anyhow(e: JsValue) -> anyhow::Error {
    anyhow::format_err!("{:?}", e)
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn download_file_with_forced_cache(
    client: &reqwest::Client,
    url: &str,
    mut progress_reporter: impl ProgressReporter,
    file: &str,
) -> anyhow::Result<Vec<u8>> {
    use futures_util::StreamExt;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Cache, Response};

    // try to hit the cache
    let window = web_sys::window().unwrap();
    let cache: Cache = JsFuture::from(window.caches().unwrap().open("laya-models-cache"))
        .await
        .map_err(js_to_anyhow)?
        .dyn_into()
        .unwrap();

    let from_c = JsFuture::from(cache.match_with_str(url))
        .await
        .map_err(js_to_anyhow)?;

    if !from_c.is_undefined() {
        let resp: Response = from_c.dyn_into().map_err(js_to_anyhow)?;
        let buffer = JsFuture::from(resp.array_buffer().map_err(js_to_anyhow)?)
            .await
            .map_err(js_to_anyhow)?;
        // no streamin from cache
        progress_reporter.report(100, format!("{url} (from cache)"));
        return Ok(js_sys::Uint8Array::new(&buffer).to_vec());
    }

    // Reqwest setup
    let mut req = client.get(url);

    let res = req
        .send()
        .await
        .with_context(|| format!("Failed to GET from '{}'", &url))?;
    let total_size = res.content_length();

    let mut data: Vec<u8> = Vec::with_capacity(total_size.unwrap_or(0) as usize);

    // download chunks
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    let mut reported = 0;
    while let Some(item) = stream.next().await {
        let chunk = item.with_context(|| format!("Error while downloading file"))?;
        data.extend_from_slice(&chunk);

        if let Some(total_size) = total_size {
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            let done_percent = ((downloaded as f32) * 100.0 / (total_size as f32)).round() as u8;
            if reported < done_percent {
                progress_reporter.report(done_percent, url.to_string());
                reported = done_percent;
            }
        }
    }

    // store in the cache
    let for_cache_body = js_sys::Uint8Array::from(&data[..]);
    let for_cache_resp =
        Response::new_with_opt_buffer_source(Some(&for_cache_body)).map_err(js_to_anyhow)?;
    JsFuture::from(cache.put_with_str(url, &for_cache_resp))
        .await
        .map_err(js_to_anyhow)?;

    return Ok(data);
}

pub(crate) mod bevy {
    use bevy::{
        app::{App, Startup},
        ecs::resource::Resource,
        prelude::*,
        tasks::AsyncComputeTaskPool,
    };
    use laya::RLAgent;
    #[derive(Resource)]
    pub(crate) struct ModelDownloadChannel {
        sender: crossbeam_channel::Sender<ModelDownloaded>,
        pub(crate) receiver: crossbeam_channel::Receiver<ModelDownloaded>,
    }

    pub(crate) struct ModelDownloaded {
        pub(crate) agent: Option<RLAgent>,
    }

    pub(crate) fn plugin(app: &mut App) {
        use bevy::prelude::*;
        app.add_systems(
            Startup,
            (setup_channel, spawn_download_task.after(setup_channel)),
        );
    }

    fn setup_channel(mut commands: Commands) {
        let (sender, receiver) = crossbeam_channel::unbounded::<ModelDownloaded>();
        commands.insert_resource(ModelDownloadChannel { sender, receiver });
    }

    pub(crate) fn spawn_download_task(channel: Res<ModelDownloadChannel>) {
        let pool = AsyncComputeTaskPool::get();
        let sender = channel.sender.clone();
        pool.spawn(async move {
            match crate::agent_builder::download_model_and_create_agent().await {
                Err(e) => {
                    error!("Failed to download model: {e:?}");
                    if let Err(e) = sender.send(ModelDownloaded { agent: None }) {
                        error!("Failed to send task download result: {e:?}")
                    }
                }
                Ok(rl_agent) => {
                    if let Err(e) = sender.send(ModelDownloaded {
                        agent: Some(rl_agent),
                    }) {
                        error!("Failed to send task download result: {e:?}")
                    }
                }
            }
        })
        .detach();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn download_file_with_forced_cache(
    client: &reqwest::Client,
    url: &str,
    mut progress_reporter: impl ProgressReporter,
    file: &str,
) -> anyhow::Result<Vec<u8>> {
    use futures_util::StreamExt;
    let dir = get_cache_folder();
    let target = dir.join(file);
    if target.is_dir() {
        anyhow::bail!("target is directory");
    }
    if target.is_file() {
        // found file in cache, should read it now
        return Ok(std::fs::read(target)?);
    }

    // Reqwest setup
    let req = client.get(url);

    let res = req
        .send()
        .await
        .with_context(|| format!("Failed to GET from '{}'", &url))?;
    let total_size = res.content_length();

    // download chunks
    let mut data: Vec<u8> = Vec::with_capacity(total_size.unwrap_or(0) as usize);

    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    let mut reported = 0;
    while let Some(item) = stream.next().await {
        let chunk = item.with_context(|| format!("Error while downloading file"))?;
        data.extend_from_slice(&chunk);

        if let Some(total_size) = total_size {
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            let done_percent = ((downloaded as f32) * 100.0 / (total_size as f32)).round() as u8;
            if reported < done_percent {
                progress_reporter.report(done_percent, url.to_string());
                reported = done_percent;
            }
        }
    }

    // save to cache
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(&parent)?;
    }
    let partial_out = target.with_extension("part");
    let mut out = std::fs::File::create(&partial_out)?;
    out.write(&data)?;
    std::fs::rename(&partial_out, &target)?;

    return Ok(data);
}
