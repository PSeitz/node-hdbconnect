"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const addon = require("../native");
const util_1 = require("util");
const createClientProm = util_1.promisify(addon.createClient);
const statement = util_1.promisify(addon.statement);
const multiple_statements_ignore_err = util_1.promisify(addon.multiple_statements_ignore_err);
const prepare = util_1.promisify(addon.prepare);
const execute_batch = util_1.promisify(addon.execute_batch);
/**
 * Opens a new connection.
 *
 * @remarks
 * Don't forget to close() the connection.
 *
 * @param opt - The IConnectionParameters
 * @returns A Promise to a connection handle.
 *
 */
async function createClient(opt) {
    const client_id = await createClientProm(opt);
    return new Connection(client_id);
}
exports.createClient = createClient;
/**
 * Returns the number of the internally hold connection. The connection does live in the native code and has to be closed by the caller.
 */
function get_num_connections() {
    return addon.get_num_connections();
}
exports.get_num_connections = get_num_connections;
/**
 * Returns the number of the internally hold prepared statements. Prepared statements live in the native code and have to be removed by the caller, when finished using.
 */
function get_num_prepared_statements() {
    return addon.get_num_prepared_statements();
}
exports.get_num_prepared_statements = get_num_prepared_statements;
class Connection {
    constructor(id) {
        this.id = id;
    }
    close() {
        return addon.dropClient(this.id);
    }
    statement(stmt) {
        return statement(this.id, stmt);
    }
    multiple_statements_ignore_err(stmt) {
        return multiple_statements_ignore_err(this.id, stmt);
    }
    set_auto_commit(bool) {
        return addon.set_auto_commit(this.id, bool);
    }
    is_auto_commit() {
        return addon.is_auto_commit(this.id);
    }
    set_fetch_size(val) {
        return addon.set_fetch_size(this.id, val);
    }
    get_lob_read_length() {
        return addon.get_lob_read_length(this.id);
    }
    set_lob_read_length(val) {
        return addon.set_lob_read_length(this.id, val);
    }
    get_call_count() {
        return addon.get_call_count(this.id);
    }
    set_application_user(val) {
        return addon.set_application_user(this.id, val);
    }
    set_application_version(val) {
        return addon.set_application_version(this.id, val);
    }
    set_application_source(val) {
        return addon.set_application_source(this.id, val);
    }
    /**
     * Creates a new prepared statement.
     *
     * @remarks
     * Don't forget to drop() the prepared statement.
     *
     */
    async prepare(stmt) {
        const prepared_statement_id = await prepare(this.id, stmt);
        return new PreparedStatement(prepared_statement_id);
    }
    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    async prepare_execute(stmt, data) {
        const prep = await this.prepare(stmt);
        try {
            prep.add_batch(data);
        }
        catch (e) {
            prep != null && prep.drop();
            throw (e);
        }
        return prep.execute_batch_and_drop();
    }
    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    async execute_prepare(stmt, data) {
        return this.prepare_execute(stmt, data);
    }
    commit() {
        return addon.commit(this.id);
    }
    rollback() {
        return addon.rollback(this.id);
    }
}
exports.Connection = Connection;
class PreparedStatement {
    constructor(id) {
        this.id = id;
    }
    add_batch(data) {
        return addon.add_row(this.id, data);
    }
    execute_batch() {
        return execute_batch(this.id);
    }
    async execute_batch_and_drop() {
        // let self = this;
        // return this.execute_batch().finally(()=>self.drop())
        try {
            let res = await this.execute_batch();
            this.drop();
            return res;
        }
        catch (e) {
            this.drop();
            throw (e);
        }
    }
    drop() {
        return addon.dropStatement(this.id);
    }
}
exports.PreparedStatement = PreparedStatement;
//# sourceMappingURL=index.js.map