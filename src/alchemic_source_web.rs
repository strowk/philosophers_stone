use std::{collections::HashMap, fmt::Display, task::Waker};

use bevy::ecs::{
    observer::On,
    query::Has,
    system::{Commands, NonSend, NonSendMut, Res, ResMut},
};
use laya::{Answer, QType, Question, RLAgent};
use serde_json::{Value, json};

use crate::{
    IngredientsPlaced, OutputReceived,
    agent_readiness::AgentReadiness,
    alchemic_source::{AlchemicClassification, ClassicalElement, get_questions},
    laya_processing::LayaProcessingResource,
    laya_worker::{AnswerDto, LayaRequest, LayaRequestsBatch, LayaResponsesBatch, LayaWorkersPool},
};

pub(crate) fn ask_laya_for_output(
    placed: On<IngredientsPlaced>,
    worker: NonSendMut<LayaWorkersPool>,
    mut commands: Commands,
    processing: Res<LayaProcessingResource>,
    readiness: Res<AgentReadiness>,
) {
    if placed.ingredient_1 == "iddqd" {
        return;
    }
    if processing.in_progress > 0 {
        commands.trigger(OutputReceived {
            output: "Cauldron is currently processing...".to_string(),
        });
        return;
    }
    let c = placed.clone();
    for ingredient in [
        c.ingredient_1,
        c.ingredient_2,
        c.ingredient_3,
        c.ingredient_4,
    ] {
        if ingredient.trim() == "" {
            commands.trigger(OutputReceived {
                output: "All 4 ingredients have to be entered for cauldron to work.".to_string(),
            });
            return;
        }
    }
    if !readiness.ready {
        commands.trigger(OutputReceived {
            output:
                "Wait until cauldron is fully readied, then type in 4 ingredients and press Enter."
                    .to_string(),
        });
        return;
    }
    commands.trigger(get_questions_for_ingredients(placed, worker));
    commands.trigger(OutputReceived {
        output: "...".to_string(),
    })
}

pub(crate) fn get_questions_for_ingredients(
    placed: On<IngredientsPlaced>,
    mut worker: NonSendMut<LayaWorkersPool>,
) -> LayaRequestsBatch {
    let mut res = LayaRequestsBatch::default();
    for ingredient in [
        &placed.ingredient_1,
        &placed.ingredient_2,
        &placed.ingredient_3,
        &placed.ingredient_4,
    ] {
        let (state, questions) = get_questions(ingredient);
        let questions = questions.into_iter().map(|(s, q)| (s, q.into())).collect();
        res.reqs.push(worker.new_request(state, questions));
    }
    res
}

pub(crate) fn transform_laya_response_to_classification(
    batch_responses: On<LayaResponsesBatch>,
) -> HashMap<String, AlchemicClassification> {
    let mut classes = HashMap::new();
    for resp in &batch_responses.responses {
        let mut element: Option<ClassicalElement> = None;
        let mut fit = false;
        for (question_id, answer) in &resp.data {
            if question_id == "element" {
                element = match answer {
                    AnswerDto::Choice { choice, .. } => Some(match choice.as_str() {
                        "fire" => ClassicalElement::Fire,
                        "water" => ClassicalElement::Water,
                        "earth" => ClassicalElement::Earth,
                        "air" => ClassicalElement::Air,
                        _ => ClassicalElement::Aether,
                    }),
                    _ => {
                        panic!("received strange answer of unexpected type");
                    }
                }
            }
            if question_id == "fit_into_cauldron" {
                fit = match answer {
                    AnswerDto::Choice { choice, .. } => choice == "yes",
                    _ => false,
                }
            }
        }
        let ingredient = resp
            .request
            .state
            .as_object()
            .unwrap()
            .get("ingredient")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        if !fit {
            classes.insert(ingredient, AlchemicClassification::NotFitForAlchemy);
        } else {
            if let Some(element) = element {
                classes.insert(ingredient, AlchemicClassification::ProducesElement(element));
            }
        }
    }
    return classes;
}
