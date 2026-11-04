use bevy::{prelude::*, transform::plugins};

use crate::{
    IngredientsPlaced,
    laya_worker::{self, AnswerDto, LayaRequest, LayaResponse},
};

pub(crate) fn process_iddqd(placed: On<IngredientsPlaced>, mut commands: Commands) {
    if placed.ingredient_1 == "iddqd" {
        for element in ["fire", "water", "earth", "air"] {
            let mut responses = vec![];

            for ingredient in ["x", "y", "z", "omega"] {
                responses.push(LayaResponse {
                    id: 1,
                    data: vec![
                        (
                            "element".to_string(),
                            AnswerDto::Choice {
                                choice: element.to_string(),
                                probabilities: vec![],
                                confidence: 0.0,
                                act_probability: 0.0,
                            },
                        ),
                        (
                            "fit_into_cauldron".to_string(),
                            AnswerDto::Choice {
                                choice: "yes".to_string(),
                                probabilities: vec![],
                                confidence: 0.0,
                                act_probability: 0.0,
                            },
                        ),
                    ],
                    request: LayaRequest {
                        id: 1,
                        questions: vec![], // does not matter for iddqd
                        state: serde_json::json!(
                            {
                                "ingredient": ingredient,
                            }
                        ),
                    },
                });
            }

            commands.trigger(laya_worker::LayaResponsesBatch { responses });
        }
    }
}
