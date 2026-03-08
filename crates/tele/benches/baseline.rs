use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tokio::runtime::Builder as RuntimeBuilder;

use tele::Client;
use tele::bot::{BotContext, RequestState, RequestStateKey, Router};
use tele::types::{BotCommand, SendMessageRequest, SetMyCommandsRequest, Update};

const COMMAND_UPDATE_JSON: &[u8] = br#"{
  "update_id": 7001,
  "message": {
    "message_id": 7001,
    "date": 1700000000,
    "chat": {"id": 1, "type": "private"},
    "from": {"id": 42, "is_bot": false, "first_name": "bench"},
    "text": "/start baseline"
  }
}"#;

const CALLBACK_UPDATE_JSON: &[u8] = br#"{
  "update_id": 7002,
  "callback_query": {
    "id": "cb-7002",
    "from": {"id": 42, "is_bot": false, "first_name": "bench"},
    "message": {
      "message_id": 11,
      "date": 1700000000,
      "chat": {"id": 1, "type": "private"},
      "text": "press"
    },
    "data": "confirm:42"
  }
}"#;

const PLAIN_MESSAGE_UPDATE_JSON: &[u8] = br#"{
  "update_id": 7003,
  "message": {
    "message_id": 7003,
    "date": 1700000000,
    "chat": {"id": 1, "type": "private"},
    "from": {"id": 42, "is_bot": false, "first_name": "bench"},
    "text": "hello baseline"
  }
}"#;

#[derive(Clone, Copy)]
struct TraceId(u64);

const TRACE_ID: RequestStateKey<TraceId> = RequestStateKey::new("trace_id");
const SECOND_TRACE_ID: RequestStateKey<TraceId> = RequestStateKey::new("second_trace_id");

fn benchmark_client() -> Option<Client> {
    let builder = Client::builder("http://127.0.0.1:9").ok()?;
    let builder = builder.bot_token("123:abc").ok()?;
    builder.build().ok()
}

fn benchmark_runtime() -> Option<tokio::runtime::Runtime> {
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .ok()
}

fn benchmark_update_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_deserialize");

    group.throughput(Throughput::Bytes(COMMAND_UPDATE_JSON.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("message_command", COMMAND_UPDATE_JSON.len()),
        &COMMAND_UPDATE_JSON,
        |b, payload| {
            b.iter(|| {
                let parsed = serde_json::from_slice::<Update>(black_box(payload));
                let _ = black_box(parsed);
            });
        },
    );

    group.throughput(Throughput::Bytes(CALLBACK_UPDATE_JSON.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("callback_query", CALLBACK_UPDATE_JSON.len()),
        &CALLBACK_UPDATE_JSON,
        |b, payload| {
            b.iter(|| {
                let parsed = serde_json::from_slice::<Update>(black_box(payload));
                let _ = black_box(parsed);
            });
        },
    );

    group.finish();
}

fn benchmark_request_serialize(c: &mut Criterion) {
    let Ok(send_message) = SendMessageRequest::new(123_456_789_i64, "hello from benchmark") else {
        return;
    };
    let Ok(commands) = SetMyCommandsRequest::new(vec![
        match BotCommand::new("start", "start the bot") {
            Ok(command) => command,
            Err(_) => return,
        },
        match BotCommand::new("help", "show help") {
            Ok(command) => command,
            Err(_) => return,
        },
        match BotCommand::new("status", "show status") {
            Ok(command) => command,
            Err(_) => return,
        },
    ]) else {
        return;
    };

    let mut group = c.benchmark_group("request_serialize");

    group.bench_function("send_message", |b| {
        b.iter(|| {
            let payload = serde_json::to_vec(black_box(&send_message));
            let _ = black_box(payload);
        });
    });

    group.bench_function("set_my_commands", |b| {
        b.iter(|| {
            let payload = serde_json::to_vec(black_box(&commands));
            let _ = black_box(payload);
        });
    });

    group.finish();
}

fn benchmark_request_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_state");

    group.bench_function("slot_set_read_remove", |b| {
        b.iter(|| {
            let state = RequestState::default();
            let _ = state.slot(TRACE_ID).set(TraceId(42));
            let _ = state.slot(SECOND_TRACE_ID).set(TraceId(7));
            let first = state.slot(TRACE_ID).cloned().map(|value| value.0);
            let second = state.slot(SECOND_TRACE_ID).cloned().map(|value| value.0);
            let removed = state.slot(TRACE_ID).remove().map(|value| value.0);
            black_box((first, second, removed));
        });
    });

    let state = RequestState::default();
    let _ = state.slot(TRACE_ID).set(TraceId(42));
    group.bench_function("get_or_insert_with_hit", |b| {
        b.iter(|| {
            let value = state.get_or_insert_with(|| TraceId(99));
            black_box(value.0);
        });
    });

    group.finish();
}

fn benchmark_router_dispatch(c: &mut Criterion) {
    let Some(client) = benchmark_client() else {
        return;
    };
    let Some(runtime) = benchmark_runtime() else {
        return;
    };
    let Ok(command_update) = serde_json::from_slice::<Update>(COMMAND_UPDATE_JSON) else {
        return;
    };
    let Ok(message_update) = serde_json::from_slice::<Update>(PLAIN_MESSAGE_UPDATE_JSON) else {
        return;
    };

    let mut command_router = Router::new();
    command_router.command_route("start").handle(
        |_context: BotContext, update: Update| async move {
            let _ = black_box(update.update_id);
            Ok(())
        },
    );

    let mut middleware_router = Router::new();
    middleware_router.middleware(|context, update, next| async move {
        let _ = context.request_state().slot(TRACE_ID).set(TraceId(42));
        let _ = context
            .request_state()
            .slot(SECOND_TRACE_ID)
            .set(TraceId(7));
        next(context, update).await
    });
    middleware_router
        .message_route()
        .handle(|context: BotContext, update: Update| async move {
            let first = context
                .request_state()
                .slot(TRACE_ID)
                .cloned()
                .map(|value| value.0);
            let second = context
                .request_state()
                .slot(SECOND_TRACE_ID)
                .cloned()
                .map(|value| value.0);
            let _ = black_box((update.update_id, first, second));
            Ok(())
        });

    let mut group = c.benchmark_group("router_dispatch");

    group.bench_function("command_route", |b| {
        let client = client.clone();
        let router = command_router.clone();
        b.to_async(&runtime).iter_batched(
            || command_update.clone(),
            |update| {
                let client = client.clone();
                let router = router.clone();
                async move {
                    let dispatched = router
                        .dispatch_prepared(BotContext::new(client), update)
                        .await;
                    let _ = black_box(dispatched);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("message_route_with_middleware_state", |b| {
        let client = client.clone();
        let router = middleware_router.clone();
        b.to_async(&runtime).iter_batched(
            || message_update.clone(),
            |update| {
                let client = client.clone();
                let router = router.clone();
                async move {
                    let dispatched = router
                        .dispatch_prepared(BotContext::new(client), update)
                        .await;
                    let _ = black_box(dispatched);
                }
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group! {
    name = baseline;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .sample_size(30);
    targets =
        benchmark_update_deserialize,
        benchmark_request_serialize,
        benchmark_request_state,
        benchmark_router_dispatch
}
criterion_main!(baseline);
