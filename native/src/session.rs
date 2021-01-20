use std::cell::RefCell;
use std::sync::Arc;

use neon::prelude::*;
use serde::Serialize;
use std::sync::mpsc::{channel, Sender};
use std::thread;

#[derive(Debug, Serialize)]
enum Event {
    Update(usize),
    Greeting(String),
    Done,
}

type BoxedSession = JsBox<RefCell<RustSession>>;

pub struct RustSession {
    id: String,
    callback: Root<JsFunction>,
    assigned_file: Option<String>,
    calculated_result: Option<u64>,
    shutdown: Option<Root<JsFunction>>,
    queue: Arc<EventQueue>,
}

impl RustSession {
    fn send_js_event_queue<T: Serialize>(
        queue: Arc<EventQueue>,
        callback: Root<JsFunction>,
        event: T,
    ) {
        let js_string = serde_json::to_string(&event).expect("Serialization failed");
        queue.send(|mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = vec![cx.string(js_string)];

            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    }

    fn assign<'a, C: Context<'a>>(
        &mut self,
        mut cx: C,
        assigned_file: &str,
    ) -> JsResult<'a, JsUndefined> {
        self.assigned_file = Some(assigned_file.into());
        let id = self.id.clone();
        let callback: Root<JsFunction> = self.callback.clone(&mut cx);
        let queue = Arc::clone(&self.queue);

        std::thread::spawn(move || {
            use std::time;
            thread::sleep(time::Duration::from_millis(1000));
            RustSession::send_js_event_queue(queue.clone(), callback, Event::Update(100));
            thread::sleep(time::Duration::from_millis(1000));
            RustSession::send_js_event_queue(queue.clone(), callback, Event::Greeting(id))
        });

        Ok(cx.undefined())
    }
}

impl Finalize for RustSession {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        let Self {
            callback, shutdown, ..
        } = self;

        if let Some(shutdown) = shutdown {
            let shutdown = shutdown.into_inner(cx);
            let this = cx.undefined();
            let args = Vec::<Handle<JsValue>>::new();
            let _ = shutdown.call(cx, this, args);
        }

        callback.drop(cx);
    }
}

pub fn session_new(mut cx: FunctionContext) -> JsResult<BoxedSession> {
    let id = cx.argument::<JsString>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let shutdown = cx.argument_opt(2);

    let queue = cx.queue();
    let shutdown = shutdown
        .map(|v| v.downcast_or_throw::<JsFunction, _>(&mut cx))
        .transpose()?
        .map(|v| v.root(&mut cx));

    let session = cx.boxed(RefCell::new(RustSession {
        id,
        callback,
        assigned_file: None,
        calculated_result: None,
        shutdown,
        queue: Arc::new(queue),
    }));

    Ok(session)
}

pub fn session_assign(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let session = cx.argument::<BoxedSession>(0)?;
    let file = cx.argument::<JsString>(1)?.value(&mut cx);
    let mut session = session.borrow_mut();

    session.assign(cx, &file)
}
