#[macro_use]
extern crate neon;
#[macro_use]
extern crate neon_serde;
#[macro_use]
extern crate serde_derive;
extern crate hdbconnect;
extern crate chashmap;
use hdbconnect::ResultSet;
use neon::prelude::*;
use chashmap::CHashMap;

#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate serde_db;

use std::collections::HashMap;
// use std::sync::Mutex;
use hdbconnect::{Connection, HdbResult, HdbError, HdbValue, HdbResponse, PreparedStatement, HdbReturnValue, ParameterRow, Parameters, ParameterDescriptor, IntoConnectParams};
use hdbconnect::ConnectParams;
use parking_lot::RwLock;
use parking_lot::Mutex;


lazy_static! {
    static ref CONNECTIONS: CHashMap<String, Connection> = {
        CHashMap::with_capacity(50)
    };
    static ref PREPARED_STATEMENTS: CHashMap<String, Mutex<PreparedStatement>> = {
        CHashMap::with_capacity(50)
    };
}

macro_rules! check_res {
    ($res:ident,$cx:ident, $success:block) => (
        match $res {
            Ok(res) => $success,
            Err(err) => {
                let js_object = JsObject::new(&mut $cx);
                let js_string = $cx.string(format!("{:?}", err));
                js_object.set(&mut $cx, "error", js_string).unwrap();
                return Ok(js_object.upcast())
            },
        }
    )
}
macro_rules! check_res_undefined {
    ($res:ident,$cx:ident) => (
        check_res!($res, $cx,  {
            return Ok($cx.undefined().upcast())
        });
    )
}


#[derive(Serialize, Debug, Deserialize)]
struct ConnectionParams {
    host: String,
    user: String,
    password: String,
    port: u32,
}

fn create_client(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let arg0 = cx.argument::<JsValue>(0)?;
    let params:ConnectionParams = neon_serde::from_value(&mut cx, arg0)?;

    // let js_object = cx.argument::<JsObject>(0)?;
    // let host = js_object.get(&mut cx, "host")?.downcast::<JsString>().or_throw(&mut cx)?.value();
    // let port = js_object.get(&mut cx, "port")?.downcast::<JsNumber>().or_throw(&mut cx)?.value();
    // let user = js_object.get(&mut cx, "user")?.downcast::<JsString>().or_throw(&mut cx)?.value();
    // let password = js_object.get(&mut cx, "password")?.downcast::<JsString>().or_throw(&mut cx)?.value();

    let connect_params = ConnectParams::builder()
        .hostname(params.host)
        .port(params.port as u16)
        .dbuser(params.user)
        .password(params.password)
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
    (*CONNECTIONS).remove(&arg0);
    Ok(cx.string("connection closed"))
}


struct ConnectTask(ConnectParams);

impl Task for ConnectTask {
    type Output = String; // the connection id
    type Error = HdbError;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let connection = Connection::new(self.0.clone())?;
        let id = nanoid::simple();
        (*CONNECTIONS).insert_new(id.to_string(), connection);
        Ok(id)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(res) => Ok(cx.string(res)),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
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
//     let mut map = (*CONNECTIONS).lock().unwrap();
//     let connection = map.get(&client_id).unwrap();
// }


fn set_auto_commit(mut cx:FunctionContext) -> JsResult<JsValue>{
    let client_id = cx.argument::<JsString>(0)?.value();
    let val = cx.argument::<JsBoolean>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_auto_commit(val);

    check_res_undefined!(res, cx);
}

fn is_auto_commit(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<bool>
    let client_id = cx.argument::<JsString>(0)?.value();
    let connection = (*CONNECTIONS).get(&client_id).unwrap();
    let res = connection.is_auto_commit();
    check_res!(res, cx,  {
        return Ok(cx.boolean(res.unwrap()).upcast())
    });
}

fn set_fetch_size(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let val = cx.argument::<JsNumber>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_fetch_size(val as u32);
    check_res_undefined!(res, cx);
}

fn get_lob_read_length(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<i32>
    let client_id = cx.argument::<JsString>(0)?.value();
    let connection = (*CONNECTIONS).get(&client_id).unwrap();
    let res = connection.get_lob_read_length();
    check_res!(res, cx,  {
        return Ok(cx.number(res.unwrap() as f64).upcast())
    });
}

fn set_lob_read_length(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let val = cx.argument::<JsNumber>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_lob_read_length(val as i32);
    check_res_undefined!(res, cx);
}

// fn get_server_resource_consumption_info(mut cx:FunctionContext ) -> JsResult<JsValue>{ //HdbResult<ServerResourceConsumptionInfo>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let connection = (*CONNECTIONS).get(&client_id).unwrap();
//     let res = connection.get_server_resource_consumption_info();
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }
fn get_call_count(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<i32>
    let client_id = cx.argument::<JsString>(0)?.value();
    let res = (*CONNECTIONS).get(&client_id).unwrap().get_call_count();
    check_res!(res, cx,  {
        return Ok(cx.number(res.unwrap() as f64).upcast())
    });
}

fn set_application_user(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let appl_user = cx.argument::<JsString>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_application_user(&appl_user);
    check_res_undefined!(res, cx);
}

// // connection.set_application_user("K2209657")?;

fn set_application_version(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let version = cx.argument::<JsString>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_application_version(&version);
    check_res_undefined!(res, cx);
}

fn set_application_source(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let source = cx.argument::<JsString>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_application_source(&source);
    check_res_undefined!(res, cx);
}

// // Sets client information into a session variable on the server.
// // connection.set_application_source("5.3.23","update_customer.rs")?;

// fn statement(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<HdbResponse>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let stmt = cx.argument::<JsString>(1)?.value();
//     let res (*CONNECTIONS).get_mut(&client_id).unwrap() connection.statement(&stmt);
//     let js_object = JsObject::new(&mut cx);
//     if res.is_err() {
//         let js_string = cx.string(format!("{:?}", res));
//         js_object.set(&mut cx, "error", js_string).unwrap();
//     }
//     Ok(js_object.upcast())
// }


// fn convert_vec_to_array(mut cx: FunctionContext, data: Vec<Vec<String>>, header: Vec<String>) -> JsResult<JsArray> {

//     // Create the JS array
//     let js_array = JsArray::new(&mut cx, data.len() as u32);

//     // Iterate over the rust Vec and map each value in the Vec to the JS array
//     for (i, row) in data.iter().enumerate() {
//         let js_object = JsObject::new(&mut cx);
//         for (j, col) in row.iter().enumerate() {
//             let col_name = cx.string(&header[j]);
//             let col_val = cx.string(col);
//             js_object.set(&mut cx, col_name, col_val).unwrap();
//         }
//         // let js_string = cx.string(obj);
//         let _  = js_array.set(&mut cx, i as u32, js_object);
//     }

//     Ok(js_array)
// }


macro_rules! try_cast_number {
    ($cx:ident, $val:ident, $cast_into:ty) => {
        if let Some(val) = $val.into_typed::<$cast_into>().unwrap() {
            return Ok($cx.number(val as f64).upcast())
        }else{
            return Ok($cx.null().upcast());
        }
    }
}

macro_rules! try_cast_string {
    ($cx:ident, $val:ident, $cast_into:ty) => {
        if let Some(val) = $val.into_typed::<$cast_into>().unwrap() {
            return Ok($cx.string(val).upcast())
        }else{
            return Ok($cx.null().upcast());
        }
    }
}
macro_rules! try_cast_buffer {
    ($cx:ident, $val:ident, $cast_into:ty) => {
        if let Some(val) = $val.into_typed::<$cast_into>().unwrap() {
            buffer!($cx, val);
        }else{
            return Ok($cx.null().upcast());
        }
    }
}

macro_rules! buffer {
    ($cx:ident, $val:ident) => {
        let mut dat = $cx.buffer($val.len() as u32).unwrap();
        $cx.borrow_mut(&mut dat, |data| {
            let slice = data.as_mut_slice::<u8>();
            slice.clone_from_slice(&$val);
        });
        return Ok(dat.upcast());
    }
}

fn hdb_value_to_js<'a>(cx: &mut TaskContext<'a>, val: HdbValue) -> JsResult<'a, JsValue> {
    use serde_db::de::DbValue;
    match val {
        HdbValue::NOTHING => {
            return Ok(cx.undefined().upcast());
        }
        HdbValue::TINYINT(_) 
        | HdbValue::SMALLINT(_)
        | HdbValue::INT(_)
        | HdbValue::BIGINT(_) 
        => 
        {
            return Ok(cx.number(val.into_typed::<isize>().unwrap() as f64).upcast())
        }
        | HdbValue::REAL(_)
        | HdbValue::DOUBLE(_)
        | HdbValue::DECIMAL(_) => {
            return Ok(cx.number(val.into_typed::<f64>().unwrap()).upcast())
        }
        HdbValue::CHAR(el) | HdbValue::VARCHAR(el) | HdbValue::NCHAR(el) | HdbValue::NVARCHAR(el) => {
            return Ok(cx.string(el).upcast())
        }
        HdbValue::BINARY(el) | HdbValue::VARBINARY(el) | HdbValue::BSTRING(el) => {
            let mut dat = cx.buffer(el.len() as u32).unwrap();
            cx.borrow_mut(&mut dat, |data| {
                let slice = data.as_mut_slice::<u8>();
                slice.clone_from_slice(&el);
            });
            return Ok(dat.upcast())
        }
        HdbValue::CLOB(_) | HdbValue::NCLOB(_) | HdbValue::BLOB(_) => {
            let val = val.into_typed::<Vec<u8>>().unwrap();
            buffer!(cx, val);
        }
        HdbValue::BOOLEAN(el) => {
            return Ok(cx.boolean(el).upcast())
        }
        HdbValue::STRING(_) | HdbValue::NSTRING(_) | HdbValue::TEXT(_) | HdbValue::SHORTTEXT(_) => {
            return Ok(cx.string(val.into_typed::<String>().unwrap()).upcast())
        }
        // HdbValue::SMALLDECIMAL(BigDecimal) | HdbValue::LONGDATE(LongDate) | HdbValue::SECONDDATE(SecondDate) | HdbValue::DAYDATE(DayDate) | HdbValue::SECONDTIME(SecondTime) => {
            
        // }
        HdbValue::N_TINYINT(_) 
        | HdbValue::N_SMALLINT(_) 
        | HdbValue::N_INT(_) 
        | HdbValue::N_BIGINT(_) => {
            try_cast_number!(cx, val, Option<isize>);
        }
        | HdbValue::N_DECIMAL(_) 
        | HdbValue::N_REAL(_) 
        | HdbValue::N_DOUBLE(_) => {
            try_cast_number!(cx, val, Option<f64>);
        }
        HdbValue::N_CHAR(_) 
        | HdbValue::N_VARCHAR(_)
        | HdbValue::N_NVARCHAR(_)
        | HdbValue::N_NCHAR(_) => {
            try_cast_string!(cx, val, Option<String>);
        }
        HdbValue::N_BINARY(el) | HdbValue::N_VARBINARY(el) | HdbValue::N_BSTRING(el) => {
            if let Some(el) = el {
                let mut dat = cx.buffer(el.len() as u32).unwrap();
                cx.borrow_mut(&mut dat, |data| {
                    let slice = data.as_mut_slice::<u8>();
                    slice.clone_from_slice(&el);
                });
                return Ok(dat.upcast());
            }
            return Ok(cx.null().upcast());
        }
        HdbValue::N_CLOB(_) | HdbValue::N_NCLOB(_) | HdbValue::N_BLOB(_) => {
            try_cast_buffer!(cx, val, Option<Vec<u8>>);
        }
        HdbValue::N_BOOLEAN(el) => {
            if let Some(el) = el {
                return Ok(cx.boolean(el).upcast())
            }
            return Ok(cx.null().upcast());
            
        }
        HdbValue::N_STRING(_) | HdbValue::N_NSTRING(_) | HdbValue::N_TEXT(_) | HdbValue::N_SHORTTEXT(_) => {
            if let Some(val) = val.into_typed::<Option<String>>().unwrap() {
                return Ok(cx.string(val).upcast())
            }else{
                return Ok(cx.null().upcast());
            }
        }
        // HdbValue::N_SMALLDECIMAL(Option<BigDecimal>) | HdbValue::N_SECONDDATE(Option<SecondDate>) | HdbValue::N_DAYDATE(Option<DayDate>) | HdbValue::N_SECONDTIME(Option<SecondTime>) => {
            
        // }

        _ => {}
    }

    Ok(cx.undefined().upcast())


}

fn convert_rs<'a>(cx: &mut TaskContext<'a>, rs: ResultSet) -> JsResult<'a, JsArray> {

    let js_array = JsArray::new(cx, 0);

    let mut i = 0;
    for row in rs {
        let mut row = row.unwrap();
        let js_object = JsObject::new(cx);
        let mut j = 0;
        let len = row.len();
        row.reverse_values();
        while let Some(col_val) = row.pop() {
            // let col_name = cx.string(row.get_fieldname(len - j -1).unwrap());
            let col_name = cx.string(row.get_fieldname(j).unwrap());
            let mut col_val = hdb_value_to_js(cx,col_val).unwrap();
            js_object.set(cx, col_name, col_val).unwrap();
            j+=1;
        }
        let _  = js_array.set(cx, i as u32, js_object);
        i += 1;
    }
    Ok(js_array)
}


struct QueryTask{
    query:String,
    conn_id: String,
}

impl Task for QueryTask {
    type Output = ResultSet; // the result
    type Error = HdbError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let res:HdbResult<ResultSet> = (*CONNECTIONS).get_mut(&self.conn_id).unwrap().query(&self.query);
        Ok(res?)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(res) => Ok(convert_rs(&mut cx, res).unwrap().upcast()),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}

// // This generic method can handle all kinds of calls, and thus has the most complex return type. In many cases it will be more appropriate to use one of the methods query(), dml(), exec(), which have the adequate simple result type you usually want.
fn query(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<ResultSet>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let query = cx.argument::<JsString>(1)?.value();
    let f = cx.argument::<JsFunction>(2)?;
    QueryTask{conn_id, query}.schedule(f);
    Ok(cx.undefined())
}

struct ExecTask{
    query:String,
    conn_id: String,
}

impl Task for ExecTask {
    type Output = ();
    type Error = HdbError;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let res:HdbResult<()> = (*CONNECTIONS).get_mut(&self.conn_id).unwrap().exec(&self.query);
        Ok(res?)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(_) => Ok(cx.undefined()),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}

fn exec(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<ResultSet>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let query = cx.argument::<JsString>(1)?.value();
    let f = cx.argument::<JsFunction>(2)?;
    ExecTask{conn_id, query}.schedule(f);
    Ok(cx.undefined())
}


struct DmlTask{
    query:String,
    conn_id: String,
}

impl Task for DmlTask {
    type Output = usize;
    type Error = HdbError;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let res:HdbResult<usize> = (*CONNECTIONS).get_mut(&self.conn_id).unwrap().dml(&self.query);
        Ok(res?)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(res) => Ok(cx.number(res as f64)),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}

fn dml(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<ResultSet>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let query = cx.argument::<JsString>(1)?.value();
    let f = cx.argument::<JsFunction>(2)?;
    DmlTask{conn_id, query}.schedule(f);
    Ok(cx.undefined())
}


struct PrepareStatementTask{
    stmt:String,
    conn_id: String,
}
impl Task for PrepareStatementTask {
    type Output = String; // the prepared statement id
    type Error = HdbError;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let prepared_statement = (*CONNECTIONS).get_mut(&self.conn_id).unwrap().prepare(&self.stmt)?;
        let id = nanoid::simple();
        (*PREPARED_STATEMENTS).insert_new(id.to_string(), Mutex::new(prepared_statement));
        Ok(id)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(res) => Ok(cx.string(res)),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}

// Executes a statement and expects a plain success.
fn prepare(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<PreparedStatement>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let stmt = cx.argument::<JsString>(1)?.value();
    let f = cx.argument::<JsFunction>(2)?;
    PrepareStatementTask{
        conn_id, stmt
    }.schedule(f);
    Ok(cx.undefined())
}

// // Prepares a statement and returns a handle to it.


fn commit(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().commit();
    check_res_undefined!(res, cx);
}


// Commits the current transaction.
fn rollback(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().rollback();
    check_res_undefined!(res, cx);
}

// // Rolls back the current transaction.
// fn spawn(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<Connection>
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let connection = (*CONNECTIONS).get(&client_id).unwrap();
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


struct MultipleStatementsIgnoreErr{
    queries:Vec<String>,
    conn_id: String,
}

impl Task for MultipleStatementsIgnoreErr {
    type Output = (); // the result
    type Error = HdbError;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let res = (*CONNECTIONS).get_mut(&self.conn_id).unwrap().multiple_statements_ignore_err(self.queries.clone());
        Ok(res)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(_) => Ok(cx.undefined()),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}


// // This generic method can handle all kinds of calls, and thus has the most complex return type. In many cases it will be more appropriate to use one of the methods query(), dml(), exec(), which have the adequate simple result type you usually want.
fn multiple_statements_ignore_err(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<ResultSet>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let queries:Vec<_> = cx.argument::<JsArray>(1)?.to_vec(&mut cx)?;
    let queries:Vec<_> = queries.iter().map(|v|v.downcast::<JsString>().unwrap().value()).collect();
    let f = cx.argument::<JsFunction>(2)?;
    MultipleStatementsIgnoreErr{conn_id, queries}.schedule(f);
    Ok(cx.undefined())
}


// // Creates a new connection object with the same settings and authentication.
// fn multiple_statements_ignore_err<mut cx:FunctionContext, S: AsRef<str>>(&mut self, stmts: Vec<S>){
//     let client_id = cx.argument::<JsString>(0)?.value();
//     let res (*CONNECTIONS).get_mut(&client_id).unwrap() connection.multiple_statements_ignore_err();
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
//     let connection = (*CONNECTIONS).get(&client_id).unwrap();
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
//     let connection = (*CONNECTIONS).get(&client_id).unwrap();
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
//     let connection = (*HASHMAP).get(&client_id).unwrap();
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





use serde_db::ser::to_params;
use serde_db::ser::SerializationError;

// Commits the current transaction.
fn add_row(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let prepared_statement_id = cx.argument::<JsString>(0)?.value();

    let data = cx.argument::<JsArray>(1)?;

    let vec: Vec<Handle<JsValue>> = data.to_vec(&mut cx)?;

    // let meta_slice = &metadata[vec.len().. vec.len() + 1];
    // let val = to_params(val, meta_slice)?.pop().unwrap();
    // vec.last_mut().map(|row|&mut row.values).unwrap().push(val);

    let mut prepo = (*PREPARED_STATEMENTS).get_mut(&prepared_statement_id);
    let mut prep = prepo.as_mut().unwrap().lock();

    if prep.input_parameter_descriptors().is_some() {
        let data = {
            let params_desc = prep.input_parameter_descriptors().unwrap();
            let data:Vec<HdbValue> = vec.into_iter().enumerate().map(|(i, val)| {
                js_to_hdb_value(&mut cx, val, params_desc[i].clone())
            }).collect();
            ParameterRow::new(data)
        };
        let res = prep.add_row(data);
        check_res_undefined!(res, cx);
    }

    // if let Some(params_desc) = prep.input_parameter_descriptors() {

    //     let data:Vec<HdbValue> = vec.into_iter().enumerate().map(|(i, val)| {
    //         js_to_hdb_value(&mut cx, val, params_desc[i].clone())
    //     }).collect();
    //     let data = ParameterRow::new(data);
    //     let res = prep.add_row(data);
    //     // for (i, val) in vec.iter().enumerate() {

    //     // }


    // }
    
    return Ok(cx.undefined().upcast())
    // let data:Vec<HdbValue> = vec.into_iter().map(|el|js_to_hdb_value(&mut cx, el)).collect();
    // println!("{:?}", data);
    // let data = ParameterRow::new(data);
    // // let v: Handle<JsValue> = cx.number(17).upcast();
    // // v.is_a::<JsString>(); // false
    // // v.is_a::<JsNumber>(); // true
    // // v.is_a::<JsValue>();  // true
    
    // // let res = prep.add_batch(&[1]); // TODO
    // let res = prep.add_row(data); // TODO
    // check_res_undefined!(res, cx);
}


struct ExecPreparedStatementTask{
    prepared_statement_id: String,
}

impl Task for ExecPreparedStatementTask {
    type Output = HdbResponse; // the result
    type Error = HdbError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let prep = (*PREPARED_STATEMENTS).get_mut(&self.prepared_statement_id).unwrap();
        let res:HdbResult<HdbResponse> = prep.lock().execute_batch();
        Ok(res?)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(res) => {
                if res.count() == 1{
                    let mut data = res.data;
                    let mut el = data.remove(0);
                    match el {
                        HdbReturnValue::ResultSet(rs) => {
                            return Ok(convert_rs(&mut cx, rs).unwrap().upcast());
                        },
                        HdbReturnValue::AffectedRows(data) => {

                            let js_array = JsArray::new(&mut cx, data.len() as u32);
                            for (i, num) in data.iter().enumerate() {
                                let num = cx.number(*num as f64);
                                let _  = js_array.set(&mut cx, i as u32, num);
                            }

                            return Ok(js_array.upcast());
                        },
                        HdbReturnValue::OutputParameters(out) => {
                            // unimplemented!()
                            return cx.throw_error("OutputParameters not implemented");
                            // return Ok(cx.string("outputs").upcast());
                        },
                        HdbReturnValue::Success => {
                            return Ok(cx.string("success").upcast());
                        },
                        HdbReturnValue::XaTransactionIds(trans_id) => {
                            // unimplemented!()
                            // return Ok(cx.string("someids").upcast());
                            return cx.throw_error("XaTransactionIds not implemented");
                        },
                        
                    }
                    
                }else{
                    unimplemented!()
                    // let data = res.get_data();
                    // for el in data {
                    // }
                }
            },
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}

// Commits the current transaction.
fn execute_batch(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<()>
    let prepared_statement_id = cx.argument::<JsString>(0)?.value();
    let f = cx.argument::<JsFunction>(1)?;
    ExecPreparedStatementTask{prepared_statement_id}.schedule(f);
    Ok(cx.undefined())
}


fn js_to_hdb_value<'a>(_cx: &mut FunctionContext<'a>, v: Handle<JsValue>, desc: ParameterDescriptor) -> HdbValue {
    let params = &[desc];
    if v.is_a::<JsString>() {
        let v = v.downcast::<JsString>().unwrap().value();
        return to_params(&v, params).unwrap().pop().unwrap();
    }
    if v.is_a::<JsNumber>() {
        let v = v.downcast::<JsNumber>().unwrap().value();
        if let Ok(mut val) = to_params(&v, params ) {
            return val.pop().unwrap()
        }
        if let Ok(mut val) = to_params(&(v as u64), params) {
            return val.pop().unwrap()
        }
    }
    if v.is_a::<JsUndefined>() {
        return HdbValue::N_TEXT(None);
    }
    panic!("not implemented");

}




// get_server_resource_consumption_info
// statement
// spawn
// multiple_statements
// pop_warnings
// get_resource_manager


register_module!(mut cx, {
    cx.export_function("createClient", create_client)?;
    cx.export_function("dropClient", drop_client)?;
    cx.export_function("query", query)?;
    cx.export_function("exec", exec)?;
    cx.export_function("dml", dml)?;
    cx.export_function("multiple_statements_ignore_err", multiple_statements_ignore_err)?;
    cx.export_function("set_auto_commit", set_auto_commit)?;
    cx.export_function("is_auto_commit", is_auto_commit)?;
    cx.export_function("set_fetch_size", set_fetch_size)?;
    cx.export_function("get_lob_read_length", get_lob_read_length)?;
    cx.export_function("set_lob_read_length", set_lob_read_length)?;
    cx.export_function("get_call_count", get_call_count)?;
    cx.export_function("set_application_user", set_application_user)?;
    cx.export_function("set_application_version", set_application_version)?;
    cx.export_function("set_application_source", set_application_source)?;
    cx.export_function("prepare", prepare)?;
    cx.export_function("commit", commit)?;
    cx.export_function("rollback", rollback)?;

    cx.export_function("add_row", add_row)?;
    cx.export_function("execute_batch", execute_batch)?;

    Ok(())
});

