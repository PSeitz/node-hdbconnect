#[macro_use]
extern crate neon;
extern crate hdbconnect;
use hdbconnect::ResultSet;
use neon::prelude::*;


#[macro_use]
extern crate lazy_static;
extern crate parking_lot;

use std::collections::HashMap;
use std::sync::Mutex;
use hdbconnect::{Connection, HdbResult, HdbError, IntoConnectParams};
use hdbconnect::ConnectParams;
use parking_lot::RwLock;
lazy_static! {
    static ref HASHMAP: RwLock<HashMap<String, Connection>> = {
        RwLock::new(HashMap::new())
    };
}


fn create_client(mut cx: FunctionContext) -> JsResult<JsUndefined> {

    let js_object = cx.argument::<JsObject>(0)?;
    let host = js_object.get(&mut cx, "host")?.downcast::<JsString>().or_throw(&mut cx)?.value();
    let port = js_object.get(&mut cx, "port")?.downcast::<JsNumber>().or_throw(&mut cx)?.value();
    let user = js_object.get(&mut cx, "user")?.downcast::<JsString>().or_throw(&mut cx)?.value();
    let password = js_object.get(&mut cx, "password")?.downcast::<JsString>().or_throw(&mut cx)?.value();

    let connect_params = ConnectParams::builder()
        .hostname(host)
        .port(port as u16)
        .dbuser(user)
        .password(password)
        .build()
        .unwrap();

    println!("{:?}", connect_params);
    let f = cx.argument::<JsFunction>(1)?;
    ConnectTask(connect_params).schedule(f);
    Ok(cx.undefined())

}

fn drop_client(mut cx: FunctionContext) -> JsResult<JsString> {
//     cx.check_argument::<JsString>(0)?;
    let arg0 = cx.argument::<JsString>(0)?.value();
    let mut map = HASHMAP.write();
    map.remove(&arg0);
    Ok(cx.string("connection closed"))
}


struct ConnectTask(ConnectParams);

impl Task for ConnectTask {
    type Output = String;
    type Error = HdbError;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let connection = Connection::new(self.0.clone())?;
        let id = nanoid::simple();
        let mut map = HASHMAP.write();
        map.insert(id.to_string(), connection);
        Ok(id)
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        Ok(cx.string(result.unwrap()))
    }
}

// fn hdb_result_to_jsvalue<T:std::fmt::Debug>(mut cx:FunctionContext, res:HdbResult<T>) -> Option<JsObject> {
//     if res.is_err() {
//         let js_object = JsObject::new(&mut cx);//             // let js_string = cx.string("an error");
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//         Some(js_object)
//     }else{
//         None
//     }
// }

// Connection

// fn new(mut cx:FunctionContext, params: ConnectParams) -> HdbResult<Connection>{
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHMAP.lock().unwrap();
//     let connection = map.get(&client_id).unwrap();
// }



fn set_auto_commit(mut cx:FunctionContext) -> JsResult<JsValue>{
    let client_id = cx.argument::<JsString>(0)?.value();
    let val = cx.argument::<JsBoolean>(1)?.value();
    let mut map = HASHMAP.write();
    let connection = map.get_mut(&client_id).unwrap();
    let res = connection.set_auto_commit(val);
    let js_object = JsObject::new(&mut cx);
    if res.is_err() {
        let js_string = cx.string(format!("{:?}", res));
        js_object.set(&mut cx, "error", js_string).unwrap();
    }
    Ok(js_object.upcast())
}

fn is_auto_commit(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<bool>
    let client_id = cx.argument::<JsString>(0)?.value();
    let map = HASHMAP.read();
    let connection = map.get(&client_id).unwrap();
    let res = connection.is_auto_commit();
    let js_object = JsObject::new(&mut cx);
    if res.is_err() {
        let js_string = cx.string(format!("{:?}", res));
        js_object.set(&mut cx, "error", js_string).unwrap();
    }else{
        let val = cx.boolean(res.unwrap());
        js_object.set(&mut cx, "val", val).unwrap();
    }
    Ok(js_object.upcast())
}

fn set_fetch_size(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let val = cx.argument::<JsNumber>(1)?.value();
    let mut map = HASHMAP.write();
    let connection = map.get_mut(&client_id).unwrap();
    let res = connection.set_fetch_size(val as u32);
    let js_object = JsObject::new(&mut cx);
    if res.is_err() {
        let js_string = cx.string(format!("{:?}", res));
        js_object.set(&mut cx, "error", js_string).unwrap();
    }
    Ok(js_object.upcast())
}

// fn get_lob_read_length(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<i32>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let map = HASHMAP.read();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.get_lob_read_length();
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }

// fn set_lob_read_length(mut cx:FunctionContext, lob_read_length: i32) -> JsResult<JsValue>{ //HdbResult<()>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.set_lob_read_length();
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }

// fn get_server_resource_consumption_info(mut cx:FunctionContext ) -> JsResult<JsValue>{ //HdbResult<ServerResourceConsumptionInfo>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let map = HASHMAP.read();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.get_server_resource_consumption_info();
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }
// fn get_call_count(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<i32>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let map = HASHMAP.read();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.get_call_count();
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }

fn set_application_user(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let appl_user = cx.argument::<JsString>(1)?.value();
    let map = HASHMAP.read();
    let connection = map.get(&client_id).unwrap();
    let res = connection.set_application_user(&appl_user);
    let js_object = JsObject::new(&mut cx);
    if res.is_err() {
        let js_string = cx.string(format!("{:?}", res));
        js_object.set(&mut cx, "error", js_string).unwrap();
    }
    Ok(js_object.upcast())
}

// // connection.set_application_user("K2209657")?;

// fn set_application_version(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let version = cx.argument::<JsString>(1)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.set_application_version(&version);
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }

// // connection.set_application_version("5.3.23")?;

// fn set_application_source(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let source = cx.argument::<JsString>(1)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.set_application_source(&source);
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }

// // Sets client information into a session variable on the server.


// // connection.set_application_source("5.3.23","update_customer.rs")?;

// fn statement(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<HdbResponse>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let stmt = cx.argument::<JsString>(1)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.statement(&stmt);
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }

// // Executes a statement on the database.

fn convert_vec_to_array(mut cx: FunctionContext, data: Vec<Vec<String>>, header: Vec<String>) -> JsResult<JsArray> {

    // Create the JS array
    let js_array = JsArray::new(&mut cx, data.len() as u32);

    // Iterate over the rust Vec and map each value in the Vec to the JS array
    for (i, row) in data.iter().enumerate() {
        let js_object = JsObject::new(&mut cx);
        for (j, col) in row.iter().enumerate() {
            let col_name = cx.string(&header[j]);
            let col_val = cx.string(col);
            js_object.set(&mut cx, col_name, col_val).unwrap();
        }
        // let js_string = cx.string(obj);
        let _  = js_array.set(&mut cx, i as u32, js_object);
    }

    Ok(js_array)
}




// // This generic method can handle all kinds of calls, and thus has the most complex return type. In many cases it will be more appropriate to use one of the methods query(), dml(), exec(), which have the adequate simple result type you usually want.
fn query(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<ResultSet>
    let client_id = cx.argument::<JsString>(0)?.value();
    let stmt = cx.argument::<JsString>(1)?.value();
    let mut map = HASHMAP.write();
    let connection = map.get_mut(&client_id).unwrap();
    let res:HdbResult<ResultSet> = connection.query(&stmt);
    match res {
        Ok(res) => {
            let mut column_names = vec![];
            {
                let meta = res.metadata();
                for i in 0..meta.number_of_fields() {
                    column_names.push(meta.columnname(i).unwrap().to_string());
                }
            }
            let res: Vec<Vec<String>> = res.try_into().unwrap();
            return Ok(convert_vec_to_array(cx, res, column_names).unwrap().upcast())
            // let js_object = JsObject::new(&mut cx);
            // return Ok(js_object.upcast())
        },
        Err(err) => {
            let js_object = JsObject::new(&mut cx);
            let js_string = cx.string(format!("{:?}", err));
            js_object.set(&mut cx, "error", js_string).unwrap();
            return Ok(js_object.upcast())
        },
    }
}

// // Executes a statement and expects a single ResultSet.
// fn dml(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<usize>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let stmt = cx.argument::<JsString>(1)?.value();
//     let mut map = write()();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.dml(&stmt);
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Executes a statement and expects a single number of affected rows.
// fn exec(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let stmt = cx.argument::<JsString>(1)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.exec(&stmt);
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Executes a statement and expects a plain success.
// fn prepare(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<PreparedStatement>
//     let client_id = cx.argument::<JsString>(0)?.value();
//.     let stmt = cx.argument::<JsString>(1)?.value();
//     let mut map = HASHMAread.unwrap();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.prepare(&stmt);
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Prepares a statement and returns a handle to it.

// // Note that the handle keeps using the same connection.
// fn commit(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.commit();
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Commits the current transaction.
// fn rollback(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.rollback();
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Rolls back the current transaction.
// fn spawn(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<Connection>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHread.unwrap();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.spawn();
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Creates a new connection object with the same settings and authentication.
// fn multiple_statements_ignore_err<mut cx:FunctionContext, S: AsRef<str>>(&mut self, stmts: Vec<S>){
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHMAP.write();
//     let connection = map.get_mut(&client_id).unwrap();
//     let res = connection.multiple_statements_ignore_err();
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Utility method to fire a couple of statements, ignoring errors and return values
// fn multiple_statements<mut cx:FunctionContext, S: AsRef<str>>({
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let mut map = HASHMAP.lock().unwrap();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.multiple_statements();
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }
// //     &mut self,
// //     stmts: Vec<S>
// // ) -> HdbResult<()>

// // Utility method to fire a couple of statements, ignoring their return values; the method returns with the first error, or with ()
// fn pop_warnings(mut cx:FunctionContext) -> HdbResult<Option<Vec<ServerError>>>{
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let map = HASHMAP.read();
//     let connection = map.get(&client_id).unwrap();
//     let res = connection.pop_warnings();
//     let js_object = JsObject::new(&mut cx);
//     match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Returns warnings that were returned from the server since the last call to this method.
// fn get_resource_manager(mut cx:FunctionContext) -> Box<dyn ResourceManager>{
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let map = HASHMAP.read();
//     let connection = map.get(&client_id).unwrap();
//.     let res = connection.get_resource_manager();
//     let js_object = JsObject::new(&mut cx);
//.    match res {
//         Ok(res) => {
//             let _res: Vec<Vec<String>> = res.try_into().unwrap();
//             let js_object = JsObject::new(&mut cx);
//             return Ok(js_object.upcast())
//         },
//         Err(err) => {
//             let js_object = JsObject::new(&mut cx);
//             let js_string = cx.string(format!("{:?}", err));
//             js_object.set(&mut cx, "error", js_string).unwrap();
//             return Ok(js_object.upcast())
//         },
//     }
// }

// // Returns an implementation of dist_tx::rm::ResourceManager that is based on this connection.
// fn execute_with_debuginfo(
























register_module!(mut cx, {
    cx.export_function("createClient", create_client)?;
    cx.export_function("dropClient", drop_client)?;
    cx.export_function("query", query)?;
//     cx.export_function("dropClient", drop_client)?;
//.     cx.export_function("createClient", createClient)

    Ok(())
});




// ister_module!(mut m, {
//     m.export_function("performAsyncTask", perform_async_task)
// });

