use std::collections::HashMap;

use bevy::{prelude::*, transform::plugins};
use laya::RLAgent;
use serde::{Deserialize, Serialize};
use superui::prelude::*;

mod agent_builder;
mod alchemic_source;

use crate::{
    agent_builder::bevy::ModelDownloadChannel,
    alchemic_source::{AlchemicClassification, AlchemicSource, Element},
};

fn plugin(app: &mut App) {
    app.init_resource::<LayaAgentResource>();
    app.add_systems(Startup, (setup));

    app.add_systems(Update, update_agent_when_model_is_ready);

    app.add_superui_command::<IngredientsPlaced>("ingredientsPlaced")
        .add_superui_event::<OutputReceived>("outputReceived")
        .add_observer(on_ingredients_placed);
}

#[derive(Event, Serialize)]
pub(crate) struct OutputReceived {
    output: String,
}

#[derive(Event, Deserialize, Debug)]
pub(crate) struct IngredientsPlaced {
    pub(crate) ingredient_1: String,
    pub(crate) ingredient_2: String,
    pub(crate) ingredient_3: String,
    pub(crate) ingredient_4: String,
}

fn on_ingredients_placed(on: On<IngredientsPlaced>, mut commands: Commands, res: Res<LayaAgentResource>) {
    match run_cauldron(on.event(), &res) {
        Err(e) => {
            error!("Failed to calculate {e:?}");
        }
        Ok(classes) => {
            let output = get_output(classes);
            commands.trigger(OutputReceived { output })
        }
    }
}

fn get_output(classes: HashMap<String, AlchemicClassification>) -> String {
    let mut has_fire = 0;
    let mut has_water = 0;
    let mut has_earth = 0;
    let mut has_air = 0;
    let mut has_aether = 0;
    let mut has_non_fit = 0;
    for (_, classification) in classes {
        match classification {
            AlchemicClassification::ProducesElement(Element::Water) => {
                has_water += 1;
            }
            AlchemicClassification::ProducesElement(Element::Fire) => {
                has_fire += 1;
            }
            AlchemicClassification::ProducesElement(Element::Earth) => {
                has_earth += 1;
            }
            AlchemicClassification::ProducesElement(Element::Air) => {
                has_air += 1;
            }
            AlchemicClassification::ProducesElement(Element::Aether) => {
                has_aether += 1;
            }
            _ => {
                has_non_fit += 1;
            }
        }
    }

    if has_non_fit > 0 {
        return "nothing in particular".to_string();
    }

    if has_water > 0 && has_fire > 0 && has_earth == 0 && has_air == 0 {
        return "quickly dissipated steam".to_string();
    }

    if has_earth > 0 && has_air > 0 && has_water == 0 && has_fire == 0 {
        return "quickly dissipated dust".to_string();
    }

    if has_earth >= 3 && has_air == 0 {
        return "ESSENCE of earth".to_string();
    }

    if has_air >= 3 && has_earth == 0 {
        return "ESSENCE of air".to_string();
    }

    if has_water >= 3 && has_fire == 0 {
        return "ESSENCE of water".to_string();
    }

    if has_fire >= 3 && has_water == 0 {
        return "ESSENCE of fire".to_string();
    }

    if has_aether >= 3 {
        return "ESSENCE of aether".to_string();
    }

    return "brown sludge".to_string();
}

#[derive(Resource)]
pub(crate) struct LayaAgentResource {
    agent: Option<RLAgent>,
}

impl Default for LayaAgentResource {
    fn default() -> Self {
        Self {
            agent: Default::default(),
        }
    }
}

fn update_agent_when_model_is_ready(
    channel: Res<ModelDownloadChannel>,
    mut agent: ResMut<LayaAgentResource>,
) {
    for msg in channel.receiver.try_iter() {
        let none = msg.agent.is_none();
        info!("Received initialized agent none: {none}");
        agent.agent = msg.agent;
    }
}

fn run_cauldron(
    placed: &IngredientsPlaced,
    agent: &LayaAgentResource,
) -> anyhow::Result<HashMap<String, AlchemicClassification>> {
    let agent = &agent.agent;
    let input = [
        placed.ingredient_1.as_str(),
        placed.ingredient_2.as_str(),
        placed.ingredient_3.as_str(),
        placed.ingredient_4.as_str(),
        // "table",
        // "meteorite",
        // "discovery",
        // "soup",
    ];

    let mut source = AlchemicSource {};

    if let Some(agent) = agent {
        let res = source.classify(agent, input)?;
        return Ok(res);
    }
    anyhow::bail!("No agent running yet");

    // for (ingredient, classification) in res.clone() {
    //     println!("{ingredient}: {classification:?}")
    // }
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(SuperUiRoot::from_asset_dir("ui", &assets));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    main();
}

fn web_asset_plugin(plugin: AssetPlugin) -> AssetPlugin {
    #[cfg(target_arch = "wasm32")]
    let plugin = AssetPlugin {
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..plugin
    };
    plugin
}

fn main() {
    let mut app = App::new();

    let default_plugins = DefaultPlugins.build().set(web_asset_plugin(default()));

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
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
    );

    app.add_plugins(agent_builder::bevy::plugin);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(default_plugins);

    app.add_plugins(SuperUiPlugin).add_plugins(plugin).run();

    // let input = [
    //     "computer",
    //     "small computer chip",
    //     "wooden table",
    //     "acid",
    //     "table",
    //     "plastic table",
    //     // "matches",
    //     "tabasco",
    //     "hot spoon",
    //     // "black powder",
    //     // "dynamite stick",
    //     "discovery",
    //     "huge meteorite",
    //     // "sun",
    //     // "eye of a newt",
    //     "soup",
    //     // "fish from a shop",
    //     "sushi roll",
    //     // "piece of meteorite",
    //     "burning twig",
    //     // ""
    //     // "water",
    //     // "coal",
    //     // "log",
    //     "lava",
    //     // "habanero",
    //     // "pepper",
    //     // "explosion",

    //     "dirt",
    //     "cooked meat",
    //     "some garbage",
    //     "plastic",

    //     "blood",
    //     "wine",
    //     "brick",

    //     "dynamite",
    //     "acid",
    //     "C4",
    //     "historical reference"
    // ];

    // for i in input {

    // let state = json!({
    //     "ingredient": i,
    // });

    // let element = Question {
    //     qtype: QType::Choice,
    //     instructions: format!("Which classical element an ingredient most associated with?"),
    //     choice_criteria: vec![
    //         ("water".into(), Some("liquid, wet, cold, aquatic".into())),
    //         ("fire".into(), Some("heated, burning, spicy, energetic".into())),
    //         ("earth".into(), Some("solid, mineral, heavy".into())),
    //         ("air".into(), Some("gas, light, fleeting, breathing".into())),
    //         ("aether".into(), Some("celestial".into())),

    //         // ("none".into(), Some("element is not clear".into()))
    //     ],
    //     score_criteria: vec![],
    //     noul_true: None,
    //     noul_false: None,
    // };

    // let fit_into_cauldron = Question {
    //     qtype: QType::Choice,
    //     instructions: format!("Is an ingredient a physical item?"),
    //     choice_criteria: vec![
    //         ("yes".into(), None),
    //         ("no".into(), Some("something abstract like 'invention'".into())),
    //     ],
    //     score_criteria: vec![],
    //     noul_true: None,
    //     noul_false: None,
    // };

    // let color = Question {
    //     qtype: QType::Choice,
    //     instructions: "Which color is this ingredient most associated with?".to_string(),
    //     choice_criteria: vec![
    //         ("yellow".into(), None),
    //         ("red".into(), None),
    //         ("green".into(), None),
    //         ("purple".into(), None),
    //         ("brown".into(), None),
    //         ("white".into(), None),
    //         ("gray".into(), None),
    //         ("blue".into(), None),
    //     ],
    //     score_criteria: vec![],
    //     noul_true: None,
    //     noul_false: None,
    // };

    // // Batched: pass as many (id, Question) pairs as you like in one forward pass.
    // let answers = agent.system_one(&state, &[
    //     ("element".to_string(), element),
    //     ("fit_into_cauldron".to_string(), fit_into_cauldron),
    //     // ("color".to_string(), color),
    // ])?;
    // println!("ingredient: {i}");
    // for (id, answer) in answers {
    //     match answer {
    //         Answer::Choice {
    //             choice,
    //             probabilities,
    //             confidence,
    //             act_probability,
    //         } => {
    //             println!("{id}: {choice} (confidence={confidence:.3}, act_p={act_probability:.3})");
    //             let p_len = probabilities.len() as f32;
    //             for (option, p) in probabilities {
    //                 println!("    {option}: {p:.3}");
    //             }
    //         }
    //         Answer::Score {
    //             score, confidence, ..
    //         } => println!("{id}: {score:.2} (confidence={confidence:.3})"),
    //         Answer::Noul { noul, .. } => println!("{id}: {noul:.3}"),
    //     }
    // }
    // println!("");

    // }
}
