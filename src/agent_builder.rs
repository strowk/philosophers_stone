use std::{cell::RefCell, io::Write, path::PathBuf, rc::Rc};

use ::bevy::log::error;
use anyhow::Context;
use laya::RLAgent;

pub(crate) trait ProgressReporter {
    fn report(&self, progress_percent: u8, current_item: String);
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProgressOnChannel {
    pub(crate) progress_percent: u8,
    pub(crate) current_item: String,
}

struct ChannelReporter {
    sender: crossbeam_channel::Sender<ProgressOnChannel>,
}

impl ProgressReporter for ChannelReporter {
    fn report(&self, progress_percent: u8, current_item: String) {
        if let Err(err) = self.sender.send(ProgressOnChannel {
            progress_percent,
            current_item,
        }) {
            error!("Failed to send progress: {err:?}")
        }
    }
}

pub(crate) struct SystemReporter {}

#[cfg(not(target_arch = "wasm32"))]
impl ProgressReporter for SystemReporter {
    fn report(&self, progress_percent: u8, current_item: String) {
        println!("{current_item}: {progress_percent}%")
    }
}

#[cfg(target_arch = "wasm32")]
impl ProgressReporter for SystemReporter {
    fn report(&self, progress_percent: u8, current_item: String) {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "{current_item}: {progress_percent}%"
        )))
    }
}

pub(crate) async fn download_model_and_create_agent<R>(
    progress_reporter: R,
) -> anyhow::Result<RLAgent>
where
    R: ProgressReporter,
{
    let client = reqwest::Client::new();

    let config_files = [
        "rl_agent_config.json",
        "tokenizer/tokenizer_config.json",
        "encoder/config.json",
    ];

    let mut res = vec![];

    for file in config_files {
        let down = download_file_with_forced_cache(
            &client,
            &format!("https://huggingface.co/convaiinnovations/laya/resolve/main/{file}"),
            &progress_reporter,
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
        let down = download_file_with_forced_cache(
            &client,
            &format!("https://huggingface.co/convaiinnovations/laya/resolve/main/{file}"),
            &progress_reporter,
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
        res.get(0).expect("did not get rl agent config"),
        res.get(1).expect("did not get tokenizer config"),
        &tokenizer.expect("did not tokenizer bytes"),
        res.get(2).expect("did not encoder config"),
        &tensors.expect("did not get weights"),
    )?)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_cache_folder() -> PathBuf {
    let dir = dirs::cache_dir().expect("no cache dir");
    dir.join("philosopher_stone").join("laya")
}

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{WorkerGlobalScope, js_sys};

#[cfg(target_arch = "wasm32")]
fn js_to_anyhow(e: JsValue) -> anyhow::Error {
    anyhow::format_err!("{:?}", e)
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn download_file_with_forced_cache(
    client: &reqwest::Client,
    url: &str,
    progress_reporter: &impl ProgressReporter,
    file: &str,
) -> anyhow::Result<Vec<u8>> {
    use futures_util::StreamExt;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Cache, Response};

    // try to hit the cache
    let cache_name = "laya-models-cache";
    let cache = {
        match web_sys::window() {
            Some(window) => {
                let cache: Cache = JsFuture::from(
                    window
                        .caches()
                        .expect("did not get caches")
                        .open(cache_name),
                )
                .await
                .map_err(js_to_anyhow)?
                .dyn_into()
                .expect("did not get cache");
                cache
            }
            None => {
                // we may be in worker
                let scope: WorkerGlobalScope = js_sys::global().unchecked_into();
                let cache: Cache =
                    JsFuture::from(scope.caches().expect("did not get caches").open(cache_name))
                        .await
                        .map_err(js_to_anyhow)?
                        .dyn_into()
                        .expect("did not get cache");
                cache
            }
        }
    };

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

    // TODO: clean this up!
    // this thing eats shitton of memory (~2-3x from model size, so circa 2Gb) cause we do lots of ugly copying
    // js -> wasm -> js and not even streaming..
    // It would make more sense to ditch reqwest here and just use fetch natively, stream directly to cache in js
    // without wasm hop, then simply re-read from cache again as above
    // (at which point entire model would load to wasm memory, but only one time)
    // Note: we cannot simply force built-in cache here, because browsers may or may not respect that,
    // so using Cache API is really needed

    // Reqwest setup
    let req = client.get(url);

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
use serde::{Deserialize, Serialize};

    use crate::agent_builder::{ChannelReporter, ProgressOnChannel};

    #[derive(Resource)]
    pub(crate) struct ModelDownloadChannels {
        sender: crossbeam_channel::Sender<ModelDownloaded>,
        pub(crate) receiver: crossbeam_channel::Receiver<ModelDownloaded>,

        progress_sender: crossbeam_channel::Sender<ProgressOnChannel>,
        progress_receiver: crossbeam_channel::Receiver<ProgressOnChannel>,
    }

    pub(crate) struct ModelDownloaded {
        pub(crate) agent: Option<RLAgent>,
    }

    #[derive(Event, Debug, Serialize, Deserialize)]
    pub(crate) struct ModelDownloadProgress {
        pub(crate) progress: ProgressOnChannel,
    }

    pub(crate) fn plugin(app: &mut App) {
        app.add_systems(
            Startup,
            (setup_channel, spawn_download_task.after(setup_channel)),
        );
        app.add_systems(Update, pass_progress);
    }

    pub(crate) fn pass_progress(channels: Res<ModelDownloadChannels>, mut commands: Commands) {
        for msg in channels.progress_receiver.try_iter() {
            commands.trigger(ModelDownloadProgress { progress: msg })
        }
    }

    fn setup_channel(mut commands: Commands) {
        let (sender, receiver) = crossbeam_channel::unbounded::<ModelDownloaded>();
        let (progress_sender, progress_receiver) =
            crossbeam_channel::unbounded::<ProgressOnChannel>();
        commands.insert_resource(ModelDownloadChannels {
            sender,
            receiver,
            progress_sender,
            progress_receiver,
        });
    }

    pub(crate) fn spawn_download_task(channel: Res<ModelDownloadChannels>) {
        let pool = AsyncComputeTaskPool::get();
        let sender = channel.sender.clone();
        let progress_sender = channel.progress_sender.clone();
        pool.spawn(async move {
            let progress_reporter = ChannelReporter {
                sender: progress_sender,
            };
            match crate::agent_builder::download_model_and_create_agent(progress_reporter).await {
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
    mut progress_reporter: &impl ProgressReporter,
    file: &str,
) -> anyhow::Result<Vec<u8>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
    let data = rt.block_on(async move {
        use ::bevy::log::info;
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
                let done_percent =
                    ((downloaded as f32) * 100.0 / (total_size as f32)).round() as u8;
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
        Ok(data)
    });

    data
}
