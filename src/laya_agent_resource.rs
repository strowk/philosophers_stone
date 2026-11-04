use bevy::{prelude::*};
use laya::RLAgent;

use crate::agent_builder::bevy::ModelDownloadChannel;

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
    channel: Res<ModelDownloadChannel>,
    mut agent: ResMut<LayaAgentResource>,
) {
    for msg in channel.receiver.try_iter() {
        let none = msg.agent.is_none();
        info!("Received initialized agent none: {none}");
        agent.agent = msg.agent;
    }
}