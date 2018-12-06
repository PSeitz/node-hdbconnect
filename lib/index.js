var addon = require('../native');

const {promisify} = require('util');

const createClient = promisify(addon.createClient); 
const query = promisify(addon.query); 
const statement = promisify(addon.statement); 
const exec = promisify(addon.exec); 
const dml = promisify(addon.dml); 
const multiple_statements_ignore_err = promisify(addon.multiple_statements_ignore_err); 
const prepare = promisify(addon.prepare); 
const execute_batch = promisify(addon.execute_batch); 

exports.createClient = opt => createClient(opt).then(conn_id => new Connection(conn_id))

function Connection(id) {
    this.drop = ()=>{addon.dropClient(id)}
    this.close = ()=>{addon.dropClient(id)}

    this.query = (stmt) => query(id, stmt);
    this.statement = (stmt) => statement(id, stmt);
    this.exec = (stmt) => exec(id, stmt);
    this.dml = (stmt) => dml(id, stmt);
    this.multiple_statements_ignore_err = (stmt) => multiple_statements_ignore_err(id, stmt);

    this.set_auto_commit = (bool) => addon.set_auto_commit(id, bool)
    this.is_auto_commit = () => addon.is_auto_commit(id)
    this.set_fetch_size = (val) => addon.set_fetch_size(id, val)
    this.get_lob_read_length = () => addon.get_lob_read_length(id)
    this.set_lob_read_length = (val) => addon.set_lob_read_length(id, val)
    this.get_call_count = () => addon.get_call_count(id)
    this.set_application_user = (val) => addon.set_application_user(id, val)
    this.set_application_version = (val) => addon.set_application_version(id, val)
    this.set_application_source = (val) => addon.set_application_source(id, val)

    this.prepare = stmt => prepare(id, stmt).then(prepared_statement_id => new PreparedStatement(prepared_statement_id))

    this.commit = () => addon.commit(id)
    this.rollback = () => addon.rollback(id)
}


function PreparedStatement(id) {
    this.add_batch = (data) => addon.add_row(id, data)
    this.execute_batch = (cb) => execute_batch(id)
}
