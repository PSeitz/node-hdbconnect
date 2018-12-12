#[macro_use]
extern crate neon;
extern crate neon_serde;
#[macro_use]
extern crate serde_derive;
extern crate hdbconnect;
extern crate chashmap;

use neon::prelude::*;
use chashmap::CHashMap;

#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate serde_db;
extern crate serde_bytes;

use serde_db::ser::to_params;
// use serde_db::ser::SerializationError;

use hdbconnect::{Connection, HdbResult, HdbError, HdbValue, HdbResponse, PreparedStatement, HdbReturnValue, ParameterDescriptor};
use hdbconnect::ResultSet;
use hdbconnect::ServerCerts;
use hdbconnect::ConnectParams;
// use parking_lot::RwLock;
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
            Ok(_res) => $success,
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
    tls: Option<String>
}

fn create_client(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let arg0 = cx.argument::<JsValue>(0)?;
    let params:ConnectionParams = neon_serde::from_value(&mut cx, arg0)?;

    let mut builder = ConnectParams::builder();
    builder.hostname(params.host)
        .port(params.port as u16)
        .dbuser(params.user)
        .password(params.password);
    if let Some(cert) = &params.tls {
        builder.tls_with(ServerCerts::Direct(cert.to_string()));
    }
    let connect_params  = builder.build().unwrap();

    let f = cx.argument::<JsFunction>(1)?;
    ConnectTask(connect_params).schedule(f);
    Ok(cx.undefined())

}

fn drop_client(mut cx: FunctionContext) -> JsResult<JsString> {
    let arg0 = cx.argument::<JsString>(0)?.value();
    (*CONNECTIONS).remove(&arg0);
    Ok(cx.string("connection closed"))
}
fn drop_statement(mut cx: FunctionContext) -> JsResult<JsString> {
    let arg0 = cx.argument::<JsString>(0)?.value();
    (*PREPARED_STATEMENTS).remove(&arg0);
    Ok(cx.string("prepared closed"))
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

// connection.set_application_user("K2209657")?;
fn set_application_version(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let version = cx.argument::<JsString>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_application_version(&version);
    check_res_undefined!(res, cx);
}

// Sets client information into a session variable on the server.
// connection.set_application_source("5.3.23","update_customer.rs")?;
fn set_application_source(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let source = cx.argument::<JsString>(1)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().set_application_source(&source);
    check_res_undefined!(res, cx);
}

macro_rules! try_cast {
    ($cx:ident, $val:ident, $cast_into:ty, $on_success:expr) => {
        if let Some(val) = $val.into_typed::<$cast_into>().unwrap() {
            return Ok($on_success(val))
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
    if val.is_null() {
        return Ok(cx.null().upcast());
    }
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
         HdbValue::BLOB(_) => {
            let val = val.try_into::<serde_bytes::ByteBuf>().unwrap();
            let val: &[u8] = val.as_ref();
            buffer!(cx, val);
        }
        HdbValue::CLOB(_)  => {
            let val = val.into_typed::<Vec<u8>>().unwrap();
            buffer!(cx, val);
        }
        HdbValue::BOOLEAN(el) => {
            return Ok(cx.boolean(el).upcast())
        }
        HdbValue::STRING(_) | HdbValue::NSTRING(_) | HdbValue::TEXT(_) | HdbValue::SHORTTEXT(_) => {
            return Ok(cx.string(val.into_typed::<String>().unwrap()).upcast())
        }
        // HdbValue::SMALLDECIMAL(BigDecimal) {
        HdbValue::LONGDATE(_) | HdbValue::SECONDDATE(_) | HdbValue::DAYDATE(_) | HdbValue::SECONDTIME(_) => {
            return Ok(cx.string(val.into_typed::<String>().unwrap()).upcast())
        }
        HdbValue::N_TINYINT(_)
        | HdbValue::N_SMALLINT(_)
        | HdbValue::N_INT(_)
        | HdbValue::N_BIGINT(_) => {
            try_cast!(cx, val, Option<isize>, (|val| cx.number(val as f64).upcast()))
        }
        | HdbValue::N_DECIMAL(_)
        | HdbValue::N_REAL(_)
        | HdbValue::N_DOUBLE(_) => {
            try_cast!(cx, val, Option<f64>, (|val| cx.number(val as f64).upcast()))
        }
        HdbValue::N_CHAR(_)
        | HdbValue::N_VARCHAR(_)
        | HdbValue::N_NVARCHAR(_)
        | HdbValue::N_NCHAR(_) => {
            try_cast_string!(cx, val, Option<String>);
        }
        HdbValue::N_BINARY(el) | HdbValue::N_VARBINARY(el) | HdbValue::N_BSTRING(el) => {
            if let Some(el) = el {
                buffer!(cx, el);
            }
            return Ok(cx.null().upcast());
        }
        HdbValue::N_CLOB(_)  => {
            try_cast_buffer!(cx, val, Option<Vec<u8>>);
        }
        HdbValue::N_NCLOB(_) => {
            try_cast_string!(cx, val, Option<String>);
        }
        HdbValue::NCLOB(_) => {
            let val = val.into_typed::<String>().unwrap();
            return Ok(cx.string(val).upcast());
        }
        // HdbValue::N_BLOB(Some(mut blob)) => {
        HdbValue::N_BLOB(_) => {
            if let Some(val) = val.try_into::<Option<serde_bytes::ByteBuf>>().unwrap() {
                let val: &[u8] = val.as_ref();
                buffer!(cx, val);
            }else{
                return Ok(cx.null().upcast());
            }
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
        // HdbValue::N_SMALLDECIMAL(Option<BigDecimal>) |
        HdbValue::N_LONGDATE(_) | HdbValue::N_SECONDDATE(_) | HdbValue::N_DAYDATE(_) | HdbValue::N_SECONDTIME(_) => {
            try_cast_string!(cx, val, Option<String>);
        }

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
        row.reverse_values();
        while let Some(col_val) = row.pop() {
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


struct StatementTask{
    statement:String,
    conn_id: String,
}

impl Task for StatementTask {
    type Output = HdbResponse; // the result
    type Error = HdbError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let res:HdbResult<HdbResponse> = (*CONNECTIONS).get_mut(&self.conn_id).unwrap().statement(&self.statement);
        Ok(res?)
    }

    fn complete(self, mut cx: TaskContext, res: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match res {
            Ok(res) => convert_hdbresponse(&mut cx, res),
            Err(res) => cx.throw_error(&format!("{:?}", res)),
        }
    }
}

// // This generic method can handle all kinds of calls, and thus has the most complex return type. In many cases it will be more appropriate to use one of the methods query(), dml(), exec(), which have the adequate simple result type you usually want.
fn statement(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<ResultSet>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let statement = cx.argument::<JsString>(1)?.value();
    let f = cx.argument::<JsFunction>(2)?;
    StatementTask{conn_id, statement}.schedule(f);
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

// Prepares a statement and returns a handle to it.
fn prepare(mut cx:FunctionContext) -> JsResult<JsUndefined>{ //HdbResult<PreparedStatement>
    let conn_id = cx.argument::<JsString>(0)?.value();
    let stmt = cx.argument::<JsString>(1)?.value();
    let f = cx.argument::<JsFunction>(2)?;
    PrepareStatementTask{
        conn_id, stmt
    }.schedule(f);
    Ok(cx.undefined())
}


/// Commits the current transaction.
fn commit(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().commit();
    check_res_undefined!(res, cx);
}

/// Rolls back the current transaction.
fn rollback(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let client_id = cx.argument::<JsString>(0)?.value();
    let res = (*CONNECTIONS).get_mut(&client_id).unwrap().rollback();
    check_res_undefined!(res, cx);
}

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



// Commits the current transaction.
fn add_row(mut cx:FunctionContext) -> JsResult<JsValue>{ //HdbResult<()>
    let prepared_statement_id = cx.argument::<JsString>(0)?.value();

    let data = cx.argument::<JsArray>(1)?;

    let vec: Vec<Handle<JsValue>> = data.to_vec(&mut cx)?;

    let mut prepo = (*PREPARED_STATEMENTS).get_mut(&prepared_statement_id);
    let mut prep = prepo.as_mut().unwrap().lock();


    if vec.len() > prep.input_parameter_descriptors().map(|el|el.len()).unwrap_or(0){
        return cx.throw_error("too many parameters");
    }

    if prep.input_parameter_descriptors().is_some() {
        let data = {
            let params_desc = prep.input_parameter_descriptors().unwrap();
            let data:Vec<HdbValue> = vec.into_iter().enumerate().map(|(i, val)| {
                js_to_hdb_value(&mut cx, val, params_desc[i].clone())
            }).collect();
            data
        };
        let res = prep.add_row_to_batch(data);
        check_res_undefined!(res, cx);
    }

    return Ok(cx.undefined().upcast())
}


fn convert_hdbreturn_value<'a>(cx: &mut TaskContext<'a>, el: HdbReturnValue) -> JsResult<'a, JsValue> {
    match el {
        HdbReturnValue::ResultSet(rs) => {
            return Ok(convert_rs(cx, rs).unwrap().upcast());
        },
        HdbReturnValue::AffectedRows(data) => {
            let js_array = JsArray::new(cx, data.len() as u32);
            for (i, num) in data.iter().enumerate() {
                let num = cx.number(*num as f64);
                let _  = js_array.set(cx, i as u32, num);
            }
            return Ok(js_array.upcast());
        },
        HdbReturnValue::OutputParameters(_out) => {
            return cx.throw_error("OutputParameters not implemented");
        },
        HdbReturnValue::Success => {
            return Ok(cx.string("success").upcast());
        },
        HdbReturnValue::XaTransactionIds(_trans_id) => {
            return cx.throw_error("XaTransactionIds not implemented");
        },
    }
}
fn convert_hdbresponse<'a>(cx: &mut TaskContext<'a>, res: HdbResponse) -> JsResult<'a, JsValue> {
    if res.count() == 1{
        let mut data = res.return_values;
        let el = data.remove(0);
        convert_hdbreturn_value(cx, el)
    }else{
        let data = res.return_values;
        let js_array = JsArray::new(cx, data.len() as u32);
        let mut i = 0;
        for el in data {

            let res = convert_hdbreturn_value(cx, el)?;
            let _  = js_array.set(cx, i as u32, res);
            i+=1;
        }
        Ok(js_array.upcast())
    }
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
                convert_hdbresponse(&mut cx, res)
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

use serde_bytes::{Bytes};
fn js_to_hdb_value<'a>(cx: &mut FunctionContext<'a>, v: Handle<JsValue>, desc: ParameterDescriptor) -> HdbValue {
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
    if v.is_a::<JsArrayBuffer>() {
        let v = v.downcast::<JsArrayBuffer>().unwrap();
        let slice: &[u8] = cx.borrow(&v, |data| {
            data.as_slice::<u8>()
        });
        return to_params(&Bytes::new(&*slice), params).unwrap().pop().unwrap();
    }
    if v.is_a::<JsBuffer>() {
        let v = v.downcast::<JsBuffer>().unwrap();
        let slice: &[u8] = cx.borrow(&v, |data| {
            data.as_slice::<u8>()
        });
        return to_params(&Bytes::new(&*slice), params).unwrap().pop().unwrap();
    }
    if v.is_a::<JsBoolean>() {
        let v = v.downcast::<JsBoolean>().unwrap().value();
        return to_params(&v, params).unwrap().pop().unwrap();
    }
    if v.is_a::<JsUndefined>() {
        return HdbValue::N_TEXT(None);
    }
    if v.is_a::<JsNull>() {
        return HdbValue::N_TEXT(None);
    }
    if v.is_a::<JsObject>() {
        let dat:String = format!("object as parameter not supported {:?}", v.to_string(cx).map(|val|val.value()).unwrap_or_else(|_|"".to_string()));
        panic!(dat);
        // cx.throw_error("&dat");
    }
    if v.is_a::<JsFunction>() {
        let dat:String = format!("function as parameter not supported {:?}", v.to_string(cx).map(|val|val.value()).unwrap_or_else(|_|"".to_string()));
        panic!(dat);
        // cx.throw_error("&dat");
    }
    if v.is_a::<JsError>() {
        let dat:String = format!("error as parameter not supported {:?}", v.to_string(cx).map(|val|val.value()).unwrap_or_else(|_|"".to_string()));
        panic!(dat);
        // cx.throw_error("&dat");
    }
    panic!("not implemented");

}

// get_server_resource_consumption_info
// spawn
// multiple_statements
// pop_warnings
// get_resource_manager
// execute_with_debuginfo

register_module!(mut cx, {
    cx.export_function("createClient", create_client)?;
    cx.export_function("dropClient", drop_client)?;
    cx.export_function("dropStatement", drop_statement)?;
    cx.export_function("query", query)?;
    cx.export_function("statement", statement)?;
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

