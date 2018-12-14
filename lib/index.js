var addon = require('../native');
const { promisify } = require('util');
const createClientProm = promisify(addon.createClient);
const statement = promisify(addon.statement);
const multiple_statements_ignore_err = promisify(addon.multiple_statements_ignore_err);
const prepare = promisify(addon.prepare);
const execute_batch = promisify(addon.execute_batch);
/**
 * Opens a new connection.
 *
 * @remarks
 * Don't forget to close() the connection.
 *
 * @param opt - The ConnectionParameters
 * @returns A Promise to a connection handle.
 *
 */
async function createClient(opt) {
    let client_id = await createClientProm(opt);
    return new Connection(client_id);
}
exports.createClient = createClient;
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
        let prepared_statement_id = await prepare(this.id, stmt);
        return new PreparedStatement(prepared_statement_id);
    }
    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    async prepare_execute(stmt, data) {
        let prep = await this.prepare(stmt);
        try {
            prep.add_batch(data);
            return prep.execute_batch();
        }
        catch (e) {
            prep != null && prep.drop();
            throw e;
        }
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
    drop() {
        return addon.dropStatement(this.id);
    }
}
//# sourceMappingURL=index.js.map