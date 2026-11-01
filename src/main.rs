use laya::{Answer, QType, Question, RLAgent};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // A checkpoint directory downloaded from Hugging Face (convaiinnovations/laya,
    // -typed-decisions, or -multilingual) — rl_agent_config.json, tokenizer/, encoder/, model.safetensors.
    let dir = dirs::cache_dir().expect("no cache dir");
    let agent = RLAgent::load(dir.join("laya-rs").join("laya-typed-decisions"))?;

    let input = [
        "computer",
        "small computer chip",
        "wooden table",
        "acid",
        "table",
        "plastic table",
        // "matches",
        "tabasco",
        "hot spoon",
        // "black powder",
        // "dynamite stick",
        "discovery",
        "huge meteorite",
        // "sun",
        // "eye of a newt",
        "soup",
        // "fish from a shop",
        "sushi roll",
        // "piece of meteorite",
        "burning twig",
        // ""
        // "water",
        // "coal",
        // "log",
        "lava",
        // "habanero",
        // "pepper",
        // "explosion",

        "dirt",
        "cooked meat",
        "some garbage",
        "plastic",

        "blood",
        "wine",
        "brick",

        "dynamite",
        "acid",
        "C4",
        "historical reference"
    ];


    for i in input {

    let state = json!({
        // "agent_description": "You are a game AI tasked to make a decisions about alchemical outcomes of player making something from ingredients",
        // "examples": {
        //     "black powder": {
        //         "element": "fire",
        //         "fit_into_cauldron": "yes",
        //         "description": "black powder is explosive substance, so it is associated with fire"
        //     },
        //     "invention": {
        //         "fit_into_cauldron": "no",
        //         "description": "abstract term, not specific enough to be put into cauldron"
        //     },
        //     "meteorite": {
        //         "fit_into_cauldron": "no",
        //         "description": "meteorite is too big to fit into cauldron"
        //     },
        //     "piece of meteorite": {
        //         "element": "aether",
        //         "fit_into_cauldron": "yes",
        //         "description": "piece of meteorite can be small enough to fit into cauldron an it has came from space where celestical objects are"
        //     },
        //     "historical reference": {
        //         "fit_into_cauldron": "not",
        //         "description": "abstract term that does not refer to any physical item"
        //     },
        //     "burning twig": {
        //         "element": "fire",
        //         "fit_into_cauldron": "yes",
        //         "description": "specific item small enough that can be placed in cauldron and since it is burning - associated with fire"
        //     },
        //     "table": {
        //         "fit_into_cauldron": "no",
        //         "description": "the whole table would not fit into cauldron regardless of material it is made of"
        //     },
        //     "piece of table": {
        //         "element": "earth",
        //         "fit_into_cauldron": "yes",
        //         "description": "part of the table might be small enough to fit into cauldron and since tables often made from wood could be associated with earth"
        //     },
        //     "habanero pepper": {
        //         "element": "fire",
        //         "fit_into_cauldron": "yes",
        //         "description": "pepper would fit into cauldron and since it is very spicy, it is associated with fire"
        //     },
        //     "dirt": {
        //         "element": "earth",
        //         "fit_into_cauldron": "yes",
        //         "description": "a bit of dirt can fit into cauldron and since it comes from earth, it is associated with it"
        //     },
        //     "wine": {
        //         "element": "water",
        //         "fit_into_cauldron": "yes",
        //         "description": "wine is a liquid, so it can be associated with water"
        //     },
        //     "fish from a shop": {
        //         "element": "water",
        //         "fit_into_cauldron": "yes",
        //         "description": "fish lives in water and hence can be associated with water"
        //     },
        //     "paper plane": {
        //         "element": "air",
        //         "fit_into_cauldron": "yes",
        //         "description": "planes fly in the air and as flying thing it is therefore associated with air"
        //     }
        // },
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
            
            // ("none".into(), Some("element is not clear".into()))
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

    let color = Question {
        qtype: QType::Choice,
        instructions: "Which color is this ingredient most associated with?".to_string(),
        choice_criteria: vec![
            ("yellow".into(), None),
            ("red".into(), None),
            ("green".into(), None),
            ("purple".into(), None),
            ("brown".into(), None),
            ("white".into(), None),
            ("gray".into(), None),
            ("blue".into(), None),
        ],
        score_criteria: vec![],
        noul_true: None,
        noul_false: None,
    };

    // Batched: pass as many (id, Question) pairs as you like in one forward pass.
    let answers = agent.system_one(&state, &[
        ("element".to_string(), element),
        ("fit_into_cauldron".to_string(), fit_into_cauldron),
        // ("color".to_string(), color),
    ])?;
    println!("ingredient: {i}");
    for (id, answer) in answers {
        match answer {
            Answer::Choice {
                choice,
                probabilities,
                confidence,
                act_probability,
            } => {
                println!("{id}: {choice} (confidence={confidence:.3}, act_p={act_probability:.3})");
                let p_len = probabilities.len() as f32;
                for (option, p) in probabilities {
                    println!("    {option}: {p:.3}");
                }
            }
            Answer::Score {
                score, confidence, ..
            } => println!("{id}: {score:.2} (confidence={confidence:.3})"),
            Answer::Noul { noul, .. } => println!("{id}: {noul:.3}"),
        }
    }
    println!("");

    }

    Ok(())
}
