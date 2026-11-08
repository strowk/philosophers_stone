use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<AgentReadiness>();
    app.add_observer(|_: On<AgentIsReady>, mut res: ResMut<AgentReadiness>| {
        res.ready = true;
    });
}

#[derive(Event)]
pub(crate) struct AgentIsReady {}

#[derive(Resource)]
pub(crate) struct AgentReadiness {
    pub(crate) ready: bool,
}

impl Default for AgentReadiness {
    fn default() -> Self {
        Self { ready: false }
    }
}
