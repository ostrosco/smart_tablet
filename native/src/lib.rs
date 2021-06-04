use neon::prelude::*;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("Hello, node."))
}

register_module!(mut cx, {
    cx.export_function("hello", hello)
});
