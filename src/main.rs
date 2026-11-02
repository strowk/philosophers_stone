mod alchemic_source;
mod agent_builder;

use crate::alchemic_source::AlchemicSource;

fn main() -> anyhow::Result<()> {
    let agent = agent_builder::get_agent()?;
    let input = [
        "table",
        "meteorite",
        "discovery",
        "soup",
    ];

    let mut source = AlchemicSource::from_agent(agent);

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
