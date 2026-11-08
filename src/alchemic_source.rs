use std::{collections::HashMap, fmt::Display, task::Waker};

use bevy::ecs::{
    observer::On, query::Has, resource::Resource, system::{Commands, NonSend, NonSendMut, ResMut},
};
use laya::{Answer, QType, Question, RLAgent};
use serde_json::{Value, json};

pub(crate) struct AlchemicSource {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ClassicalElement {
    Water,
    Earth,
    Fire,
    Air,
    Aether,
}

impl Display for ClassicalElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ClassicalElement::Water => "water",
            ClassicalElement::Fire => "fire",
            ClassicalElement::Air => "air",
            ClassicalElement::Earth => "earth",
            ClassicalElement::Aether => "aether",
        })
    }
}

struct IngredientAnswers {
    ingredient: String,
    element: ClassicalElement,
    fit: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum AlchemicClassification {
    NotFitForAlchemy,
    ProducesElement(ClassicalElement),
}

impl AlchemicSource {
    pub(crate) fn classify(
        &mut self,
        agent: &RLAgent,
        inputs: [&str; 4],
    ) -> anyhow::Result<HashMap<String, AlchemicClassification>> {
        let mut out = HashMap::new();
        let res = self.ask_questions(agent, inputs);
        if let Ok(answers) = res {
            for IngredientAnswers {
                ingredient,
                element,
                fit,
            } in answers
            {
                if !fit {
                    out.insert(ingredient, AlchemicClassification::NotFitForAlchemy);
                } else {
                    out.insert(ingredient, AlchemicClassification::ProducesElement(element));
                }
            }
        }
        Ok(out)
    }

    fn ask_questions(
        &mut self,
        agent: &RLAgent,
        inputs: [&str; 4],
    ) -> anyhow::Result<Vec<IngredientAnswers>> {
        let mut result = vec![];
        for i in inputs {
            result.push(self.ask_questions_for_ingredient(agent, i)?);
        }
        return Ok(result);
    }

    fn ask_questions_for_ingredient(
        &mut self,
        agent: &RLAgent,
        i: &str,
    ) -> anyhow::Result<IngredientAnswers> {
        let state = json!({
            "ingredient": i,
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

        let answers = agent.system_one(
            &state,
            &[
                (element_id.to_string(), element),
                (fit_into_cauldron_id.to_string(), fit_into_cauldron),
            ],
        )?;

        let mut result = IngredientAnswers {
            ingredient: i.into(),
            element: ClassicalElement::Aether,
            fit: false,
        };

        for (id, answer) in answers {
            match answer {
                Answer::Choice { choice, .. } => {
                    // println!("{id}: {choice} (confidence={confidence:.3}, act_p={act_probability:.3})");
                    // for (option, p) in probabilities {
                    //     println!("    {option}: {p:.3}");
                    // }
                    if id == element_id {
                        result.element = match choice.as_str() {
                            "fire" => ClassicalElement::Fire,
                            "water" => ClassicalElement::Water,
                            "earth" => ClassicalElement::Earth,
                            "air" => ClassicalElement::Air,
                            _ => ClassicalElement::Aether,
                        }
                    }
                    if id == fit_into_cauldron_id {
                        result.fit = match choice.as_str() {
                            "yes" => true,
                            _ => false,
                        }
                    }
                }
                _ => (),
            }
        }

        Ok(result)
    }
}

pub(crate) fn get_questions(
    ingredient: &str,
) -> (Value, Vec<(String, laya::Question)>) {
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

    let questions = vec![
        (element_id.into(), element.into()),
        (fit_into_cauldron_id.into(), fit_into_cauldron.into()),
    ];
    (state, questions)
}
