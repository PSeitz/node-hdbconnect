var addon = require('../native');

const {promisify} = require('util');

const createClientProm = promisify(addon.createClient);
const statement = promisify(addon.statement);
const multiple_statements_ignore_err = promisify(addon.multiple_statements_ignore_err);
const prepare = promisify(addon.prepare);
const execute_batch = promisify(addon.execute_batch);

interface ConnectionParameters {
    host: string;
    port: number;
    user: string;
    password: string;
    tls?: string;
}

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
async function createClient(opt:ConnectionParameters):Promise<Connection> {
    let client_id = await createClientProm(opt)
    return new Connection(client_id)
}

exports.createClient = createClient;

class Connection {
    private id: string;
    constructor(id: string) {
        this.id = id;
    }

    close() {
        return addon.dropClient(this.id)
    }
    statement(stmt:string):Promise<any[]> {
        return statement(this.id, stmt)
    }
    multiple_statements_ignore_err(stmt:string[]) {
        return multiple_statements_ignore_err(this.id, stmt)
    }
    set_auto_commit(bool:boolean) {
        return addon.set_auto_commit(this.id, bool)
    }
    is_auto_commit() {
        return addon.is_auto_commit(this.id)
    }
    set_fetch_size(val:number) {
        return addon.set_fetch_size(this.id, val)
    }
    get_lob_read_length():number {
        return addon.get_lob_read_length(this.id)
    }
    set_lob_read_length(val:number) {
        return addon.set_lob_read_length(this.id, val)
    }
    get_call_count():number {
        return addon.get_call_count(this.id)
    }
    set_application_user(val:string) {
        return addon.set_application_user(this.id, val)
    }
    set_application_version(val:number) {
        return addon.set_application_version(this.id, val)
    }
    set_application_source(val:string) {
        return addon.set_application_source(this.id, val)
    }

    /**
     * Creates a new prepared statement.
     *
     * @remarks
     * Don't forget to drop() the prepared statement.
     *
     */
    async prepare(stmt:string):Promise<PreparedStatement> {
        let prepared_statement_id:string = await prepare(this.id, stmt)
        return new PreparedStatement(prepared_statement_id);
    }

    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    async execute_prepare(stmt:string, data: any[]):Promise<any[]>{
        let prep = await this.prepare(stmt);
        try{
            prep.add_batch(data);
            return prep.execute_batch();
        }catch(e){
            prep != null && prep.drop();
            throw e;
        }
    }
    commit(){
        return addon.commit(this.id)
    }
    rollback (){
        return addon.rollback(this.id)
    }
}


class PreparedStatement {
    private id: string;
    constructor(id: string) {
        this.id = id;
    }
    add_batch(data:any[]) {
        return addon.add_row(this.id, data)
    }
    execute_batch():Promise<any[]> {
        return execute_batch(this.id)
    }
    drop() {
        return addon.dropStatement(this.id)
    }
}

