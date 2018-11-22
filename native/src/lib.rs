#[macro_use]
extern crate neon;
extern crate hdbconnect;
use neon::prelude::*;


#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;
use hdbconnect::{Connection, HdbResult, IntoConnectParams};

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<String, Connection>> = {
        Mutex::new(HashMap::new())
    };    
}


fn create_client(mut cx: FunctionContext) -> JsResult<JsString> {

    let js_object = cx.argument::<JsObject>(0)?;
    
    let host = js_object.get(&mut cx, "host")?.downcast::<JsString>().or_throw(&mut cx)?.value();
    let port = js_object.get(&mut cx, "port")?.downcast::<JsNumber>().or_throw(&mut cx)?.value();
    let user = js_object.get(&mut cx, "user")?.downcast::<JsString>().or_throw(&mut cx)?.value();
    let password = js_object.get(&mut cx, "password")?.downcast::<JsString>().or_throw(&mut cx)?.value();


    use hdbconnect::ConnectParams;
    let connect_params = ConnectParams::builder()
        .hostname(host)
        .port(port as u16)
        .dbuser(user)
        .password(password)
        .build()
        .unwrap();

    let mut connection = Connection::new(connect_params).unwrap();
    let mut map = HASHMAP.lock().unwrap();
    map.insert("connection".to_string(), connection);
    Ok(cx.string("hello node"))
}

register_module!(mut cx, {
    cx.export_function("createClient", create_client)
    // cx.export_function("createClient", createClient)
});

