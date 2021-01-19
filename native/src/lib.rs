use neon::prelude::*;

mod session;
use crate::session::*;

register_module!(mut cx, {
    cx.export_function("session_new", session_new)?;
    cx.export_function("session_assign", session_assign)?;
    Ok(())
});
