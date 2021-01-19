// use neon::prelude::*;
// use std::sync::Arc;

// pub struct RustSession {
//     callback: Arc<Root<JsFunction>>,
//     queue: Arc<EventQueue>,
// }

// declare_types! {

//     pub class JsRustSession for RustSession {
//         init(mut cx) {
//             let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
//             Ok(RustSession {
//                 queue: Arc::new(cx.queue()),
//                 callback: Arc::new(callback),
//             })
//         }

//         method async_operation(mut cx) {
//             let this = cx.this();
//             let (queue, mut callback) = {
//                 let guard = cx.lock();
//                 let session = this.borrow(&guard);
//                 (session.queue.clone(), session.callback.clone())
//             };
//             std::thread::spawn(move || queue.send(move |mut cx| {
//                 use std::{thread, time};
//                 thread::sleep(time::Duration::from_millis(1000));

//                 let cb: &mut Root<JsFunction> = Arc::get_mut(&mut callback).unwrap();
//                 // let callback = cb.into_inner(&mut cx);
//                 let this = cx.undefined();
//                 let args = vec![cx.string("hello")];

//                 cb.call(&mut cx, this, args)?;

//                 Ok(())
//             }));

//             Ok(cx.undefined().upcast())
//         }
//     }
// }
use std::cell::RefCell;
use std::sync::Arc;

use neon::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
enum Events {
    Update(usize),
    Done,
}

type BoxedSession = JsBox<RefCell<RustSession>>;

pub struct RustSession {
    id: String,
    callback: Root<JsFunction>,
    assigned_file: Option<String>,
    shutdown: Option<Root<JsFunction>>,
    queue: Arc<EventQueue>,
}

impl RustSession {
    fn send_js_event_queue<'a, C: Context<'a>, T: Serialize>(
        mut cx: C,
        queue: Arc<EventQueue>,
        callback: Root<JsFunction>,
        event: T,
    ) -> JsResult<'a, JsUndefined> {
        let js_string = serde_json::to_string(&event).expect("Serialization failed");
        let callback = callback.into_inner(&mut cx);
        let this = cx.undefined();
        let args = vec![cx.string(js_string)];

        callback.call(&mut cx, this, args)?;
        Ok(cx.undefined())
    }

    fn send_js_event<'a, C: Context<'a>, T: Serialize>(
        &self,
        mut cx: C,
        event: T,
    ) -> JsResult<'a, JsUndefined> {
        let callback = self.callback.clone(&mut cx);
        let queue = Arc::clone(&self.queue);
        RustSession::send_js_event_queue(cx, queue, callback, event)
        // let js_string = serde_json::to_string(&event).expect("Serialization failed");
        // let callback = callback.into_inner(&mut cx);
        // let this = cx.undefined();
        // let args = vec![cx.string(js_string)];

        // callback.call(&mut cx, this, args)?;
        // Ok(cx.undefined())
    }

    fn assign<'a, C: Context<'a>>(&self, mut cx: C) -> JsResult<'a, JsUndefined> {
        let id = self.id.clone();
        let callback = self.callback.clone(&mut cx);
        let queue = Arc::clone(&self.queue);

        std::thread::spawn(move || {
            use std::{thread, time};
            thread::sleep(time::Duration::from_millis(1000));
            queue.send(|mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                let args = vec![cx.string(id)];

                callback.call(&mut cx, this, args)?;

                Ok(())
            })
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
        shutdown,
        queue: Arc::new(queue),
    }));

    Ok(session)
}

pub fn session_assign(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let session = cx.argument::<BoxedSession>(0)?;
    let session = session.borrow();

    session.assign(cx)
}
