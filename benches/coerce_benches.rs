use coerce::actor::context::ActorContext;
use coerce::actor::message::{Handler, Message};
use coerce::actor::scheduler::ActorType::Anonymous;
use coerce::actor::system::ActorSystem;
use coerce::actor::{Actor, IntoActorId, LocalActorRef};
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct BenchmarkActor;

impl Actor for BenchmarkActor {}

struct Msg;

impl Message for Msg {
    type Result = ();
}

#[async_trait::async_trait]
impl Handler<Msg> for BenchmarkActor {
    async fn handle(&mut self, _message: Msg, _ctx: &mut ActorContext) {}
}

async fn actor_1000_send_and_wait(actor: &LocalActorRef<BenchmarkActor>) {
    for _ in 0..1000 {
        let _ = actor.send(Msg).await.unwrap();
    }
}

async fn actor_999_notify_1_send_and_wait(actor: &LocalActorRef<BenchmarkActor>) {
    for _ in 0..999 {
        let _ = actor.notify(Msg);
    }

    let _ = actor.send(Msg).await.unwrap();
}

fn actor_send_1000_benchmark(c: &mut Criterion) {
    let runtime = rt();
    let actor = runtime.block_on(async { actor().await });

    c.bench_function("actor_send_1000", |b| {
        b.iter(|| runtime.block_on(actor_1000_send_and_wait(&actor)))
    });
}

fn actor_notify_1000_benchmark(c: &mut Criterion) {
    let runtime = rt();
    let actor = runtime.block_on(async { actor().await });

    c.bench_function("actor_notify_1000", |b| {
        b.iter(|| runtime.block_on(actor_999_notify_1_send_and_wait(&actor)))
    });
}

fn rt() -> Runtime {
    tokio::runtime::Builder::new_multi_thread().build().unwrap()
}
fn create_1000_actors(c: &mut Criterion) {
    let runtime = rt();

    c.bench_function("create_1000_actors", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 0..1000 {
                    let _ = actor().await;
                }
            })
        })
    });
}

fn actor_send_receive_on_current_thread_1000_benchmark(c: &mut Criterion) {
    // let runtime = rt();

    c.bench_function("actor_send_receive_on_current_thread_1000", |b| {
        b.iter(|| async {
            let local = tokio::task::LocalSet::new();

            // let send_receive_1000 = async move {
            //     let actor = actor().await;

            //     for _ in 0..1000 {
            //         actor.send(Msg).await.unwrap();
            //     }
            // };

            // local.spawn_local(send_receive_1000);
            // local.await;
            local
                .run_until(async move {
                    let send_receive_1000 = async move {
                        let actor = actor().await;

                        for _ in 0..1000 {
                            actor.send(Msg).await.unwrap();
                        }
                    };

                    tokio::task::spawn_local(send_receive_1000);
                })
                .await;
        });
    });
}

async fn actor() -> LocalActorRef<BenchmarkActor> {
    let system = ActorSystem::new();
    system
        .new_actor("actor".into_actor_id(), BenchmarkActor, Anonymous)
        .await
        .expect("unable to create actor")
}

criterion_group!(
    benches,
    actor_send_1000_benchmark,
    actor_notify_1000_benchmark,
    create_1000_actors,
    actor_send_receive_on_current_thread_1000_benchmark
);
criterion_main!(benches);
