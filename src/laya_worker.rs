use core::time::Duration;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use bevy::{app::PluginsState, asset::uuid, log::LogPlugin, platform::time::Instant, prelude::*};

use laya::{Answer, QType, Question};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{
    DedicatedWorkerGlobalScope, MessageEvent, Worker, WorkerGlobalScope, WorkerOptions, js_sys,
};

use crate::{
    agent_builder,
    laya_agent_resource::{self, LayaAgentResource},
};

struct PendingBatch {
    responses: Vec<LayaResponse>,
    request_ids: Vec<u64>,
}

struct LayaWebWorker {
    web_worker: Worker,
    queue: VecDeque<u64>,
}

struct InFlightRequest {
    worker_index: usize,
}

pub(crate) struct LayaWorkersPool {
    web_workers: Vec<LayaWebWorker>,
    in_flight_reqs: HashMap<u64, InFlightRequest>,
    inbox: Rc<RefCell<VecDeque<LayaResponse>>>,
    next_id: u64,
    pending_batches: HashMap<uuid::Uuid, PendingBatch>,
    req_to_batch: HashMap<u64, uuid::Uuid>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_laya_worker);
    app.add_systems(Update, poll_laya_worker);
    app.add_observer(
        |on: On<LayaRequest>, mut pool: NonSendMut<LayaWorkersPool>| {
            let req =
                serde_json::to_string(on.event()).expect("failed to encode laya request as json");
            if let Some((worker_index, worker)) = pool.get_soonest_to_free_worker() {
                if let Err(err) = worker.web_worker.post_message(&JsValue::from_str(&req)) {
                    error!("Failed to post worker message: {err:?}");
                } else {
                    worker.queue.push_back(on.id);
                    pool.in_flight_reqs
                        .insert(on.id, InFlightRequest { worker_index });
                }
            }
        },
    );
    app.add_observer(
        |on: On<LayaRequestsBatch>, mut pool: NonSendMut<LayaWorkersPool>| {
            let reqs = &on.event().reqs;
            let mut req_ids = vec![];
            let batch_id = uuid::Uuid::new_v4();
            for req in reqs.iter() {
                pool.req_to_batch.insert(req.id, batch_id);
                req_ids.push(req.id);
                let req_ser =
                    serde_json::to_string(&req).expect("failed to encode laya request as json");
                if let Some((worker_index, worker)) = pool.get_soonest_to_free_worker() {
                    if let Err(err) = worker.web_worker.post_message(&JsValue::from_str(&req_ser)) {
                        error!("Failed to post worker message: {err:?}");
                    } else {
                        worker.queue.push_back(req.id);
                        pool.in_flight_reqs
                            .insert(req.id, InFlightRequest { worker_index });
                    }
                }
            }
            pool.pending_batches.insert(
                batch_id,
                PendingBatch {
                    responses: Vec::with_capacity(reqs.len()),
                    request_ids: req_ids,
                },
            );
        },
    );
}

impl LayaWorkersPool {
    pub(crate) fn new_request(
        &mut self,
        state: Value,
        questions: Vec<(String, QuestionDto)>,
    ) -> LayaRequest {
        LayaRequest {
            id: self.generate_next_id(),
            state: state,
            questions: questions,
        }
    }

    fn generate_next_id(&mut self) -> u64 {
        let give = self.next_id;
        self.next_id += 1;
        return give;
    }

    fn get_soonest_to_free_worker(&mut self) -> Option<(usize, &mut LayaWebWorker)> {
        let mut shortest_queue = usize::MAX;
        let mut soonest_worker = None;
        for (idx, worker) in self.web_workers.iter_mut().enumerate() {
            let queue_len = worker.queue.len();
            if queue_len < 1 {
                return Some((idx, worker));
            }
            if queue_len < shortest_queue {
                shortest_queue = queue_len;
                soonest_worker = Some((idx, worker));
            }
        }
        return soonest_worker;
    }
}

fn poll_laya_worker(worker: NonSendMut<LayaWorkersPool>, mut commands: Commands) {
    let worker = worker.into_inner();
    while let Some(resp) = worker.inbox.borrow_mut().pop_front() {
        let resp_id = resp.id;
        info!("Received response from laya id={resp_id}");
        if let Some(batch_id) = worker.req_to_batch.get(&resp_id) {
            if let Some(batch) = worker.pending_batches.get_mut(batch_id) {
                batch.responses.push(resp);
                info!("Matches response to batch id={batch_id}");
                if batch.responses.len() >= batch.request_ids.len() {
                    if let Some(batch) = worker.pending_batches.remove(batch_id) {
                        // removing so we own pending batch to not clone
                        commands.trigger(LayaResponsesBatch {
                            responses: batch.responses,
                        });
                    } else {
                        error!("Failed to remove batch that we have just seen, very strange.")
                    }
                }
            } else {
                error!("Could not find pending batch {batch_id}, dropping response")
            }
            worker.req_to_batch.remove(&resp_id);
        } else {
            commands.trigger(resp);
        }

        if let Some(inf) = worker.in_flight_reqs.remove(&resp_id) {
            if let Some(web_worker) = worker.web_workers.get_mut(inf.worker_index) {
                if let Some(popped_id) = web_worker.queue.pop_front() {
                    if popped_id != resp_id {
                        // this probably does not matter since actual values are moot anyway, but would be interesting to know
                        warn!(
                            "Popped id from queue does not match response id, do they arrive in wrong order?"
                        )
                    }
                }
            } else {
                warn!("Could not find web worker matching the in flight request");
            }
        } else {
            warn!("Could not find in flight request matching response id");
        }
    }
}

fn setup_laya_worker(world: &mut World) {
    let mut web_workers = vec![];
    let inbox = Rc::new(RefCell::new(VecDeque::new()));

    for _ in 0..1 {
        // temporarily fix to one worker, should scale depending on navigator.deviceMemory and navigator.hardwareConcurrency
        // and also make sure to start only one while model is downloading first time

        let options = WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        let new_worker_result = Worker::new_with_options("./init_worker.js", &options);
        match new_worker_result {
            Err(err) => {
                error!("Failed to create a worker: {err:?}");
            }
            Ok(web_worker) => {
                let inbox_cb = inbox.clone();
                let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                    let Some(text) = e.data().as_string() else {
                        error!("Received not a string data");
                        return;
                    };
                    let resp: LayaResponse =
                        serde_json::from_str(&text).expect("response sent was not parsed as json");
                    inbox_cb.borrow_mut().push_back(resp);
                });
                web_worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();
                web_workers.push(LayaWebWorker {
                    queue: VecDeque::new(),
                    web_worker,
                });
            }
        }
    }

    world.insert_non_send(LayaWorkersPool {
        web_workers,
        in_flight_reqs: HashMap::new(),
        inbox,
        next_id: 1,
        pending_batches: HashMap::new(),
        req_to_batch: HashMap::new(),
    });
}

pub(crate) fn worker_main() {
    let mut app = App::new();
    app.add_plugins((
        LogPlugin::default(),
        TaskPoolPlugin::default(),
        agent_builder::bevy::plugin,
        laya_agent_resource::plugin,
        runner_plugin,
    ));
    info!("Laya Worker start");
    app.run();
}

fn runner_plugin(app: &mut App) {
    app.set_runner(worker_runner);
}

fn worker_runner(app: App) -> AppExit {
    let app = Rc::new(RefCell::new(app));
    worker_loop(app);
    AppExit::Success
}

#[derive(Serialize, Deserialize, Debug, Clone, Event)]
pub(crate) struct LayaRequest {
    pub id: u64,
    pub state: Value,
    pub questions: Vec<(String, QuestionDto)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Event)]
pub(crate) struct LayaRequestsBatch {
    pub(crate) reqs: Vec<LayaRequest>,
}

impl Default for LayaRequestsBatch {
    fn default() -> Self {
        Self {
            reqs: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestionDto {
    pub qtype: QTypeDto,
    pub instructions: String,
    #[serde(default)]
    pub choice_criteria: Vec<(String, Option<String>)>,
    #[serde(default)]
    pub score_criteria: Vec<String>,
    #[serde(default)]
    pub noul_true: Option<String>,
    #[serde(default)]
    pub noul_false: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum QTypeDto {
    Choice,
    Score,
    Noul,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AnswerDto {
    Choice {
        choice: String,
        probabilities: Vec<(String, f32)>,
        confidence: f32,
        act_probability: f32,
    },
    Score {
        score: f32,
        legend: Vec<String>,
        probabilities: Vec<f32>,
        confidence: f32,
        act_probability: f32,
    },
    Noul {
        noul: f32,
        act_probability: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Event)]
pub(crate) struct LayaResponse {
    pub(crate) id: u64,
    pub(crate) data: Vec<(String, AnswerDto)>,
    pub(crate) request: LayaRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone, Event)]
pub(crate) struct LayaResponsesBatch {
    pub(crate) responses: Vec<LayaResponse>,
}

impl From<QTypeDto> for QType {
    fn from(q: QTypeDto) -> Self {
        match q {
            QTypeDto::Choice => QType::Choice,
            QTypeDto::Score => QType::Score,
            QTypeDto::Noul => QType::Noul,
        }
    }
}

impl From<QType> for QTypeDto {
    fn from(q: QType) -> Self {
        match q {
            QType::Choice => QTypeDto::Choice,
            QType::Score => QTypeDto::Score,
            QType::Noul => QTypeDto::Noul,
        }
    }
}

impl From<QuestionDto> for Question {
    fn from(q: QuestionDto) -> Self {
        Question {
            qtype: q.qtype.into(),
            instructions: q.instructions,
            choice_criteria: q.choice_criteria,
            score_criteria: q.score_criteria,
            noul_true: q.noul_true,
            noul_false: q.noul_false,
        }
    }
}

impl From<Question> for QuestionDto {
    fn from(q: Question) -> Self {
        QuestionDto {
            qtype: q.qtype.into(),
            instructions: q.instructions,
            choice_criteria: q.choice_criteria,
            score_criteria: q.score_criteria,
            noul_true: q.noul_true,
            noul_false: q.noul_false,
        }
    }
}

impl From<Answer> for AnswerDto {
    fn from(value: Answer) -> Self {
        match value {
            Answer::Choice {
                choice,
                probabilities,
                confidence,
                act_probability,
            } => AnswerDto::Choice {
                choice,
                probabilities,
                confidence,
                act_probability,
            },
            Answer::Noul {
                noul,
                act_probability,
            } => AnswerDto::Noul {
                noul,
                act_probability,
            },
            Answer::Score {
                score,
                legend,
                probabilities,
                confidence,
                act_probability,
            } => AnswerDto::Score {
                score,
                legend,
                probabilities,
                confidence,
                act_probability,
            },
        }
    }
}

fn worker_loop(app: Rc<RefCell<App>>) {
    // this repeats same as ScheduleRunnerPlugin does, but we yield to js event loop
    // while waiting for plugins to finish in case if any would depend on web IO
    // , which would not advance if we block the thread
    let plugins_state = app.borrow_mut().plugins_state();
    if plugins_state != PluginsState::Cleaned {
        if plugins_state == PluginsState::Adding {
            // ask js to start us again after short period
            let scope: WorkerGlobalScope = js_sys::global().unchecked_into();
            let cb = Closure::once_into_js(move || worker_loop(app));
            let _ =
                scope.set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 10);
            return;
        }

        let mut borrowed_app = app.borrow_mut();
        borrowed_app.finish();
        borrowed_app.cleanup();
    }

    // reading incoming worker messages

    info!("Worker starting reading messages");
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let app_for_messages = app.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(message_string) = e.data().as_string() {
            // info!("Received message: {message_string}");
            let req: serde_json::Result<LayaRequest> = serde_json::from_str(&message_string);
            match req {
                Ok(req) => {
                    let mut borrowed_app = app_for_messages.borrow_mut();
                    let agent_resource =
                        borrowed_app.world_mut().resource_mut::<LayaAgentResource>();
                    match &agent_resource.agent {
                        Some(agent) => {
                            let id = req.id;
                            let mut resp = LayaResponse {
                                request: req,
                                id: id,
                                data: vec![],
                            };
                            let questions: Vec<(String, _)> = resp
                                .request
                                .questions
                                .clone()
                                .into_iter()
                                .map(|(name, q)| (name, q.into()))
                                .collect();
                            let reply = match agent.system_one(&resp.request.state, &questions) {
                                Ok(answer) => {
                                    let reply: Vec<(String, AnswerDto)> =
                                        answer.into_iter().map(|(s, a)| (s, a.into())).collect();
                                    Some(reply)
                                }
                                Err(err) => {
                                    error!("Cannot get answer: {err}");
                                    None
                                }
                            };

                            if let Some(reply) = reply {
                                resp.data = reply;
                                let resp_to_send =
                                    serde_json::to_string(&resp).expect("json encoding failed");

                                let scope: DedicatedWorkerGlobalScope =
                                    js_sys::global().unchecked_into();
                                if let Err(err) =
                                    scope.post_message(&JsValue::from_str(&resp_to_send))
                                {
                                    error!("Failed to post message: {err:?}");
                                }
                            }
                        }
                        None => {
                            error!("Cannot process message, agent is not ready");
                        }
                    };
                    borrowed_app.update();
                }
                Err(err) => {
                    error!("Received unparseable message: {err}")
                }
            }
        }
    });

    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // run periodic ticks

    // this is mostly so that if we plan any async task on the pool
    // , then it would eventually be finished, including the task
    // that asynchroniously reads the model

    let tick = move |app: &mut App, _wait: Option<Duration>| -> Result<Option<Duration>, AppExit> {
        let start_time = Instant::now();

        app.update();

        if let Some(exit) = app.should_exit() {
            return Err(exit);
        };

        let end_time = Instant::now();

        if let Some(wait) = _wait {
            let exe_time = end_time - start_time;
            if exe_time < wait {
                return Ok(Some(wait - exe_time));
            }
        }

        Ok(None)
    };

    let ms = Duration::from_millis(1);
    let wait = Duration::from_millis(10);
    fn set_timeout(callback: &Closure<dyn FnMut()>, dur: Duration) {
        let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
        scope
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                dur.as_millis() as i32,
            )
            .expect("Should register `setTimeout`.");
    }

    let exit = Rc::new(RefCell::new(AppExit::Success));
    let closure_exit = exit.clone();

    let mut app = Rc::new(app);
    let moved_tick_closure = Rc::new(RefCell::new(None));
    let base_tick_closure = moved_tick_closure.clone();

    let tick_app = move || {
        let app = Rc::get_mut(&mut app).unwrap();
        let mut borrowed_app = app.borrow_mut();
        let delay = tick(&mut borrowed_app, Some(wait));
        match delay {
            Ok(delay) => set_timeout(
                moved_tick_closure.borrow().as_ref().unwrap(),
                delay.unwrap_or(ms),
            ),
            Err(code) => {
                closure_exit.replace(code);
            }
        }
    };
    *base_tick_closure.borrow_mut() = Some(Closure::wrap(Box::new(tick_app) as Box<dyn FnMut()>));
    set_timeout(base_tick_closure.borrow().as_ref().unwrap(), ms);
}
