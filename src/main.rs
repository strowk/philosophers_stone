use std::{collections::HashMap, fmt::Display, task::Waker};

use laya::{Answer, QType, Question, RLAgent};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // A checkpoint directory downloaded from Hugging Face (convaiinnovations/laya,
    // -typed-decisions, or -multilingual) — rl_agent_config.json, tokenizer/, encoder/, model.safetensors.
    let dir = dirs::cache_dir().expect("no cache dir");
    let agent = RLAgent::load(dir.join("laya-rs").join("laya-typed-decisions"))?;

    let input = [
        "table",
        "meteorite",
        "discovery",
        "soup",
    ];

    let mut source = AlchemicSource {
        agent,
    };

    let res = source.classify(input)?;
    for (ingredient, classification) in res {
        println!("{ingredient}: {classification:?}")
    }

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

    Ok(())
}

struct AlchemicSource {
    agent: RLAgent,   
}

#[derive(Debug)]
enum Element {
    Water,
    Earth,
    Fire,
    Air,
    Aether,
}

struct IngredientAnswers{
    ingredient: String,
    element: Element,
    fit: bool,
}

#[derive(Debug)]
enum AlchemicClassification {
    NotFitForAlchemy,
    ProducesElement(Element),
}

impl AlchemicSource {

fn classify(&mut self, inputs: [&str; 4]) -> anyhow::Result<HashMap<String, AlchemicClassification>> {
    let mut out = HashMap::new();
    let res = self.ask_questions(inputs);
    if let Ok(answers) = res {
        for IngredientAnswers{ingredient, element, fit} in answers {
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
    return Ok(result)
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
            ("fire".into(), Some("heated, burning, spicy, energetic".into())),
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
            ("no".into(), Some("something abstract like 'invention'".into())),
        ],
        score_criteria: vec![],
        noul_true: None,
        noul_false: None,
    };

    let element_id = "element";
    let fit_into_cauldron_id = "fit_into_cauldron";

    let answers = self.agent.system_one(&state, &[
        (element_id.to_string(), element),
        (fit_into_cauldron_id.to_string(), fit_into_cauldron),
    ])?;

    let mut result = IngredientAnswers{
        ingredient: i.into(),
        element: Element::Aether,
        fit: false,
    };

    for (id, answer) in answers {
        match answer {
            Answer::Choice {
                choice,
                ..
            } => {
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