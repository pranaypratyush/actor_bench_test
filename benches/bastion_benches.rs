use bastion::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fmt::Debug;
use std::thread::sleep;
use std::time::Duration;
use tracing::error;

async fn child_task(ctx: BastionContext) -> Result<(), ()> {
    loop {
        MessageHandler::new(ctx.recv().await?)
            .on_question(|n: i32, sender| {
                if n == 42 {
                    sender.reply(101).expect("Failed to reply to sender");
                } else {
                    error!("Expected number `42`, found `{}`", n);
                }
            })
            .on_fallback(|v, addr| panic!("Wrong message from {:?}: got {:?}", addr, v));
    }
}

fn actor_creation() -> ChildRef {
    let child = Bastion::children(|children| children.with_exec(child_task))
        .expect("Couldn't create the children group.");
    child.elems()[0].clone()
}

async fn request<T: 'static + Debug + Send + Sync>(
    child: &ChildRef,
    body: T,
) -> std::io::Result<()> {
    let answer = child
        .ask_anonymously(body)
        .expect("Couldn't perform request")
        .await
        .expect("Couldn't receive answer");

    MessageHandler::new(answer)
        .on_tell(|n: i32, _| assert_eq!(n, 101))
        .on_fallback(|_, _| panic!("Unknown message"));

    Ok(())
}

fn send_message(child: &ChildRef) {
    run!(request(child, black_box(42))).expect("send_command_to_child failed");
}

fn notify_message(child: &ChildRef) {
    child
        .tell_anonymously(black_box("A message containing data."))
        .expect("Couldn't send the message.");

    // Yield to the Bastion runtime by sleeping for a short duration
    sleep(Duration::from_millis(10));
}
fn bastion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bastion");

    group.bench_function("actor_creation", |b| b.iter(|| actor_creation()));
    let child_ref = actor_creation();
    group.bench_function("send_message", |b| b.iter(|| send_message(&child_ref)));
    // group.bench_function("notify_message", |b| b.iter(|| notify_message(&child_ref)));
    group.finish();
}

criterion_group!(benches, bastion_benchmark);
criterion_main!(benches);
