use std::{collections::HashMap, fmt::Display, task::Waker};

use bevy::ecs::{
    observer::On,
    query::Has,
    system::{Commands, NonSend, NonSendMut},
};
use laya::{Answer, QType, Question, RLAgent};
use serde_json::{Value, json};

use crate::{
    IngredientsPlaced, alchemic_source::{AlchemicClassification, ClassicalElement}, laya_worker::{AnswerDto, LayaRequest, LayaRequestsBatch, LayaResponsesBatch, LayaWorker},
};

pub(crate) fn ask_laya_for_output(
    placed: On<IngredientsPlaced>,
    worker: NonSendMut<LayaWorker>,
    mut commands: Commands,
) {
    commands.trigger(get_questions_for_ingredients(placed, worker));
}

pub(crate) fn get_questions_for_ingredients(
    placed: On<IngredientsPlaced>,
    mut worker: NonSendMut<LayaWorker>,
) -> LayaRequestsBatch {
    let mut res = LayaRequestsBatch::default();
    for ingredient in [
        &placed.ingredient_1,
        &placed.ingredient_2,
        &placed.ingredient_3,
        &placed.ingredient_4,
    ] {
        let state = json!({
            "ingredient": ingredient,
        });

        let element = Question {
            qtype: QType::Choice,
            instructions: format!("Which classical element an ingredient most associated with?"),
            choice_criteria: vec![
                ("water".into(), Some("liquid, wet, cold, aquatic".into())),
                (
                    "fire".into(),
                    Some("heated, burning, spicy, energetic".into()),
                ),
                ("earth".into(), Some("solid, mineral, heavy".into())),
                ("air".into(), Some("gas, light, fleeting, breathing".into())),
                ("aether".into(), Some("celestial".into())),
            ],
            score_criteria: vec![],
            noul_true: None,
            noul_false: None,
        };

        let fit_into_cauldron = Question {
            qtype: QType::Choice,
            instructions: format!("Is an ingredient a physical item?"),
            choice_criteria: vec![
                ("yes".into(), None),
                (
                    "no".into(),
                    Some("something abstract like 'invention'".into()),
                ),
            ],
            score_criteria: vec![],
            noul_true: None,
            noul_false: None,
        };

        let element_id = "element";
        let fit_into_cauldron_id = "fit_into_cauldron";

        res.reqs.push(worker.new_request(
            state,
            vec![
                (element_id.into(), element.into()),
                (fit_into_cauldron_id.into(), fit_into_cauldron.into()),
            ],
        ));
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
