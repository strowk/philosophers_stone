use bevy::prelude::*;

use crate::{
    Loading,
    laya_worker::{LayaRequestsBatch, LayaResponse, LayaResponsesBatch},
};

#[derive(Resource, Default)]
pub(crate) struct LayaProcessingResource {
    max: usize,
    pub(crate) in_progress: usize,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<LayaProcessingResource>();
    app.add_observer(on_batch_req);
    app.add_observer(on_batch_res);
}

fn on_batch_req(
    on: On<LayaRequestsBatch>,
    mut processing: ResMut<LayaProcessingResource>,
    commands: Commands,
) {
    processing.max += on.reqs.len();
    processing.in_progress += on.reqs.len();
    send_loading(processing, commands);
}

fn on_batch_res(
    _: On<LayaResponse>,
    mut processing: ResMut<LayaProcessingResource>,
    commands: Commands,
) {
    processing.in_progress -= 1;
    if processing.in_progress <= 0 {
        processing.max = 0
    }
    send_loading(processing, commands);
}

fn send_loading(processing: ResMut<'_, LayaProcessingResource>, mut commands: Commands<'_, '_>) {
    if processing.in_progress > 0 && processing.max > 0 && processing.max >= processing.in_progress {
        let done = processing.max - processing.in_progress;
        let pct = (done * 100).div_ceil(processing.max);
        commands.trigger(Loading {
            show: true,
            name: "combining ingredients...".to_string(),
            percent: pct as u8,
        })
    } else {
        commands.trigger(Loading {
            show: false,
            name: "".to_string(),
            percent: 0,
        })
    }
}
