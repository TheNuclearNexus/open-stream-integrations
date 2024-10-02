use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use once_cell::sync::Lazy;
use std::{convert::Infallible, time::Duration};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    Mutex,
};
use tokio_stream::StreamExt as _;

#[derive(Clone)]
#[derive(Debug)]
pub struct SseEvent {
    pub data: String,
    pub event: String
}

pub static BROADCAST: Lazy<Mutex<(Sender<SseEvent>, Receiver<SseEvent>)>> =
    Lazy::new(|| Mutex::new(broadcast::channel::<SseEvent>(16)));

pub async fn handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A `Stream` that repeats an event every second

    let rx = BROADCAST.lock().await.1.resubscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx);

    Sse::new(
        stream
            .map(|msg| {
                let msg = msg.unwrap();
                Event::default().data(msg.data).event(msg.event)
            })
            .map(Ok),
    )
    .keep_alive(KeepAlive::new())
}
