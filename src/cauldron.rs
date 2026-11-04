use std::collections::HashMap;

use crate::{
    IngredientsPlaced,
    alchemic_source::{AlchemicClassification, AlchemicSource},
    laya_agent_resource,
};

pub(crate) fn run_cauldron(
    placed: &IngredientsPlaced,
    agent: &laya_agent_resource::LayaAgentResource,
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
