var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : new P(function (resolve) { resolve(result.value); }).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var addon = require('../native');
const { promisify } = require('util');
const createClient = promisify(addon.createClient);
const query = promisify(addon.query);
const statement = promisify(addon.statement);
const exec = promisify(addon.exec);
const dml = promisify(addon.dml);
const multiple_statements_ignore_err = promisify(addon.multiple_statements_ignore_err);
const prepare = promisify(addon.prepare);
const execute_batch = promisify(addon.execute_batch);
function createClientWrap(opt) {
    return __awaiter(this, void 0, void 0, function* () {
        let client_id = yield createClient(opt);
        return new Connection(client_id);
    });
}
exports.createClient = createClientWrap;
class Connection {
    constructor(id) {
        this.id = id;
    }
    close() {
        return addon.dropClient(this.id);
    }
    query(stmt) {
        return query(this.id, stmt);
    }
    statement(stmt) {
        return statement(this.id, stmt);
    }
    exec(stmt) {
        return exec(this.id, stmt);
    }
    dml(stmt) {
        return dml(this.id, stmt);
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
    prepare(stmt) {
        return __awaiter(this, void 0, void 0, function* () {
            let prepared_statement_id = yield prepare(this.id, stmt);
            return new PreparedStatement(prepared_statement_id);
        });
    }
    execute_prepare(stmt, data) {
        return __awaiter(this, void 0, void 0, function* () {
            let prep = yield this.prepare(stmt);
            try {
                prep.add_batch(data);
                return prep.execute_batch();
            }
            catch (e) {
                prep != null && prep.drop();
                throw e;
            }
        });
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