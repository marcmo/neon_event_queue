use neon::prelude::*;

mod session;
use crate::session::JsRustSession;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_class::<JsRustSession>("RustSession")?;
    Ok(())
});
