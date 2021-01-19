use neon::prelude::*;
use std::sync::Arc;

pub struct RustSession {
    callback: Arc<Root<JsFunction>>,
    queue: Arc<EventQueue>,
}

declare_types! {

    pub class JsRustSession for RustSession {
        init(mut cx) {
            let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
            Ok(RustSession {
                queue: Arc::new(cx.queue()),
                callback: Arc::new(callback),
            })
        }

        method async_operation(mut cx) {
            let this = cx.this();
            let (queue, mut callback) = {
                let guard = cx.lock();
                let session = this.borrow(&guard);
                (session.queue.clone(), session.callback.clone())
            };
            std::thread::spawn(move || queue.send(move |mut cx| {
                use std::{thread, time};
                thread::sleep(time::Duration::from_millis(1000));

                let cb: &mut Root<JsFunction> = Arc::get_mut(&mut callback).unwrap();
                // let callback = cb.into_inner(&mut cx);
                let this = cx.undefined();
                let args = vec![cx.string("hello")];

                cb.call(&mut cx, this, args)?;

                Ok(())
            }));

            Ok(cx.undefined().upcast())
        }
    }
}
