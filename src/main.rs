use std::{collections::HashMap, env::args};

use bevy::{prelude::*, transform::plugins};
use laya::{Answer::Choice, Question, RLAgent};
use serde::{Deserialize, Serialize};
use superui::prelude::*;
use tokio::runtime::{Builder, Runtime};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

mod agent_builder;
mod alchemic_source;
#[cfg(target_arch = "wasm32")]
mod alchemic_source_web;
mod cauldron;
mod laya_agent_resource;
#[cfg(target_arch = "wasm32")]
mod laya_worker;

#[cfg(target_arch = "wasm32")]
mod iddqd;
#[cfg(target_arch = "wasm32")]
mod laya_processing;

use crate::{
    agent_builder::{
        SystemReporter,
        bevy::{ModelDownloadChannels, ModelDownloadProgress},
        download_model_and_create_agent,
    },
    alchemic_source::{AlchemicClassification, AlchemicSource, ClassicalElement},
};
#[cfg(target_arch = "wasm32")]
use crate::{
    alchemic_source_web::ask_laya_for_output,
    laya_worker::{AnswerDto, LayaRequest, LayaResponse, QTypeDto},
};

use crate::alchemic_source::get_questions;

fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
    app.init_resource::<ProducedElements>();

    #[cfg(target_arch = "wasm32")]
    app.add_superui_command::<IngredientsPlaced>("ingredientsPlaced")
        .add_superui_event::<OutputReceived>("outputReceived");

    app.add_superui_event::<Loading>("loading");
    app.add_superui_event::<EssencesToShow>("essences");
    // .add_observer(on_ingredients_placed)

    app.add_observer(on_received_all_elements);
    app.add_superui_event::<GotStone>("gotStone");

    #[cfg(target_arch = "wasm32")]
    app.add_observer(ask_laya_for_output)
        .add_observer(iddqd::process_iddqd)
        .add_observer(process_laya_output);

    app.add_observer(
        |model_download_progress: On<ModelDownloadProgress>, mut commands: Commands| {
            let current_item = &model_download_progress.progress.current_item;
            let name = format!("downloading - {current_item}");
            let percent = model_download_progress.progress.progress_percent;
            commands.trigger(Loading {
                show: percent < 100,
                name,
                percent,
            })
        },
    );
}

#[derive(Event, Serialize)]
struct Loading {
    show: bool,
    name: String,
    percent: u8,
}

#[derive(Event)]
struct ReceivedAllElements {}

#[derive(Event, Serialize, Deserialize)]
struct GotStone {}

enum CauldronOutput {
    Element(ClassicalElement),
    Steam,
    Dust,
    Sludge,
    Nothing,
}

#[cfg(target_arch = "wasm32")]
fn process_laya_output(
    resp: On<laya_worker::LayaResponsesBatch>,
    mut commands: Commands,
    mut produced_elements: ResMut<ProducedElements>,
) {
    let classes = alchemic_source_web::transform_laya_response_to_classification(resp);
    let (output, output_class) = get_output(&classes);

    if let CauldronOutput::Element(el) = output_class {
        let new_count = produced_elements.elements.get(&el).unwrap_or(&0) + 1;
        produced_elements.elements.insert(el, new_count);
    }

    let elements_count = produced_elements.elements.len();
    info!("Have {elements_count}");
    if elements_count >= 4 {
        // total possible is 4
        // this means we got them all
        info!("Received all elements");
        commands.trigger(ReceivedAllElements {});
    } else {
        commands.trigger(OutputReceived { output });
    }

    let mut essences: Vec<String> = vec![];
    for (element, count) in produced_elements.elements.iter() {
        if *count > 0 {
            essences.push(format!("{element}"));
        }
    }
    commands.trigger(EssencesToShow { essences });
}

struct EssenceInInventory {
    name: String,
    count: usize,
}

#[derive(Event, Serialize)]
struct EssencesToShow {
    essences: Vec<String>,
}

fn on_received_all_elements(_: On<ReceivedAllElements>, mut commands: Commands) {
    info!("Received all elements, got the stone.");
    commands.trigger(GotStone {});
}

#[derive(Event, Serialize)]
pub(crate) struct OutputReceived {
    output: String,
}

#[derive(Event, Deserialize, Debug, Clone)]
pub(crate) struct IngredientsPlaced {
    pub(crate) ingredient_1: String,
    pub(crate) ingredient_2: String,
    pub(crate) ingredient_3: String,
    pub(crate) ingredient_4: String,
}

#[derive(Resource, Default)]
struct ProducedElements {
    elements: HashMap<ClassicalElement, u64>,
}

// fn on_ingredients_placed(
//     on: On<IngredientsPlaced>,
//     mut commands: Commands,
//     res: Res<laya_agent_resource::LayaAgentResource>,
// ) {
//     match cauldron::run_cauldron(on.event(), &res) {
//         Err(e) => {
//             error!("Failed to calculate {e:?}");
//         }
//         Ok(classes) => {
//             let output = get_output(&classes);
//             commands.trigger(OutputReceived { output })
//         }
//     }
// }

fn get_output(classes: &HashMap<String, AlchemicClassification>) -> (String, CauldronOutput) {
    let mut has_fire = 0;
    let mut has_water = 0;
    let mut has_earth = 0;
    let mut has_air = 0;
    let mut has_aether = 0;
    let mut has_non_fit = 0;
    for (_, classification) in classes {
        match classification {
            AlchemicClassification::ProducesElement(ClassicalElement::Water) => {
                has_water += 1;
            }
            AlchemicClassification::ProducesElement(ClassicalElement::Fire) => {
                has_fire += 1;
            }
            AlchemicClassification::ProducesElement(ClassicalElement::Earth) => {
                has_earth += 1;
            }
            AlchemicClassification::ProducesElement(ClassicalElement::Air) => {
                has_air += 1;
            }
            AlchemicClassification::ProducesElement(ClassicalElement::Aether) => {
                has_aether += 1;
            }
            _ => {
                has_non_fit += 1;
            }
        }
    }

    if has_non_fit > 0 {
        return ("nothing in particular".to_string(), CauldronOutput::Nothing);
    }

    if has_water > 0 && has_fire > 0 && has_earth == 0 && has_air == 0 {
        return ("quickly dissipated steam".to_string(), CauldronOutput::Steam);
    }

    if has_earth > 0 && has_air > 0 && has_water == 0 && has_fire == 0 {
        return ("quickly dissipated dust".to_string(), CauldronOutput::Dust);
    }

    if has_earth >= 3 && has_air == 0 {
        return ("ESSENCE of earth".to_string(), CauldronOutput::Element(ClassicalElement::Earth));
    }

    if has_air >= 3 && has_earth == 0 {
        return ("ESSENCE of air".to_string(), CauldronOutput::Element(ClassicalElement::Air));
    }

    if has_water >= 3 && has_fire == 0 {
        return ("ESSENCE of water".to_string(), CauldronOutput::Element(ClassicalElement::Water));
    }

    if has_fire >= 3 && has_water == 0 {
        return ("ESSENCE of fire".to_string(), CauldronOutput::Element(ClassicalElement::Fire));
    }

    if has_aether >= 3 {
        return ("ESSENCE of aether".to_string(), CauldronOutput::Element(ClassicalElement::Aether));
    }

    return ("brown sludge".to_string(), CauldronOutput::Sludge);
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(SuperUiRoot::from_asset_dir("ui", &assets));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn wasm_start_main_app() {
    main_app();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn wasm_start_worker() {
    laya_worker::worker_main();
}

fn web_asset_plugin(plugin: AssetPlugin) -> AssetPlugin {
    #[cfg(target_arch = "wasm32")]
    let plugin = AssetPlugin {
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..plugin
    };
    plugin
}

async fn check_element_association(item: String) -> anyhow::Result<()> {
    let agent = download_model_and_create_agent(SystemReporter {}).await?;
    let (state, questions) = get_questions(&item);
    let questions: Vec<(String, Question)> =
        questions.into_iter().map(|(s, q)| (s, q.into())).collect();
    let output = agent.system_one(&state, &questions[..])?;
    for (_, answer) in output {
        match answer {
            Choice {
                choice,
                probabilities,
                confidence,
                act_probability,
            } => {
                println!("choice: {choice}");
                println!("probabilities: {probabilities:?}");
                println!("confidence: {confidence}");
                println!("act_probability: {act_probability}");
            }
            _ => (),
        }
    }
    Ok(())
}

fn main() {
    let args = args();
    if args.len() > 1 {
        let last = args.last();
        if let Some(last) = last {
            println!("checking.. {last}");
            let rt = Builder::new_current_thread().build().unwrap();
            rt.block_on(async {
                if let Err(err) = check_element_association(last).await {
                    println!("error: {err}");
                }
            });
            return;
        }
    }
    main_app();
}

fn main_app() {
    let mut app = App::new();

    let default_plugins = DefaultPlugins.build().set(web_asset_plugin(default()));

    #[cfg(target_arch = "wasm32")]
    app.add_plugins((
        default_plugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // fixing annoying meta logs stuff
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
        laya_worker::plugin,
        laya_processing::plugin,
    ));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins((default_plugins, agent_builder::bevy::plugin));

    app.add_plugins((
        // agent_builder::bevy::plugin,
        SuperUiPlugin,
        // laya_agent_resource::plugin,
        plugin,
    ))
    .run();
}
