use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

use coerce::actor::scheduler::ActorType::Anonymous;
use coerce::actor::system::ActorSystem;
use coerce::actor::{Actor, IntoActorId, LocalActorRef};

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

struct BenchmarkActor;

impl Actor for BenchmarkActor {}

async fn actor() -> LocalActorRef<BenchmarkActor> {
    let system = ActorSystem::new();
    system
        .new_actor("actor".into_actor_id(), BenchmarkActor, Anonymous)
        .await
        .expect("unable to create actor")
}

criterion_group!(benches, create_1000_actors);
criterion_main!(benches);
