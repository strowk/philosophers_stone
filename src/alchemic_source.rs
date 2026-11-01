use std::{collections::HashMap, fmt::Display, task::Waker};

use laya::{Answer, QType, Question, RLAgent};
use serde_json::json;

pub(crate) struct AlchemicSource {
    agent: RLAgent,
}

#[derive(Debug)]
pub(crate) enum Element {
    Water,
    Earth,
    Fire,
    Air,
    Aether,
}

struct IngredientAnswers {
    ingredient: String,
    element: Element,
    fit: bool,
}

#[derive(Debug)]
pub(crate) enum AlchemicClassification {
    NotFitForAlchemy,
    ProducesElement(Element),
}

impl AlchemicSource {
    pub(crate) fn from_agent(agent: RLAgent) -> AlchemicSource {
        return AlchemicSource { agent };
    }

    pub(crate) fn classify(
        &mut self,
        inputs: [&str; 4],
    ) -> anyhow::Result<HashMap<String, AlchemicClassification>> {
        let mut out = HashMap::new();
        let res = self.ask_questions(inputs);
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

    fn ask_questions(&mut self, inputs: [&str; 4]) -> anyhow::Result<Vec<IngredientAnswers>> {
        let mut result = vec![];
        for i in inputs {
            result.push(self.ask_questions_for_ingredient(i)?);
        }
        return Ok(result);
    }

    fn ask_questions_for_ingredient(&mut self, i: &str) -> anyhow::Result<IngredientAnswers> {
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

        let answers = self.agent.system_one(
            &state,
            &[
                (element_id.to_string(), element),
                (fit_into_cauldron_id.to_string(), fit_into_cauldron),
            ],
        )?;

        let mut result = IngredientAnswers {
            ingredient: i.into(),
            element: Element::Aether,
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
                            "fire" => Element::Fire,
                            "water" => Element::Water,
                            "earth" => Element::Earth,
                            "air" => Element::Air,
                            _ => Element::Aether,
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
