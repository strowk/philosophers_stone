use bevy::prelude::*;
use laya::RLAgent;

use crate::{Loading, agent_builder::bevy::ModelDownloadChannels};

#[derive(Resource)]
pub(crate) struct LayaAgentResource {
    pub(crate) agent: Option<RLAgent>,
}

impl Default for LayaAgentResource {
    fn default() -> Self {
        Self {
            agent: Default::default(),
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<LayaAgentResource>();
    app.add_systems(Update, update_agent_when_model_is_ready);
}

fn update_agent_when_model_is_ready(
    channel: Res<ModelDownloadChannels>,
    mut agent: ResMut<LayaAgentResource>,
    mut commands: Commands,
) {
    for msg in channel.receiver.try_iter() {
        let none = msg.agent.is_none();
        info!("Received initialized agent none: {none}");
        agent.agent = msg.agent;
        commands.trigger(Loading {
            name: "".to_string(),
            percent: 0,
            show: false, // download is done
        });
    }
}
