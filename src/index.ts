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

interface ConnectionParameters {
    host: string;
    port: number;
    user: string;
    password: string;
    tls?: string;
}

async function createClientWrap(opt:ConnectionParameters):Promise<Connection> {
    let client_id = await createClient(opt)
    return new Connection(client_id)
}

exports.createClient = createClientWrap;

class Connection {
    private id: string;
    constructor(id: string) {
        this.id = id;
    }

    close() {
        return addon.dropClient(this.id)
    }
    query(stmt:string):any[] {
        return query(this.id, stmt)
    }
    statement(stmt:string):any[] {
        return statement(this.id, stmt)
    }
    exec(stmt:string):any[] {
        return exec(this.id, stmt)
    }
    dml(stmt:string):any[] {
        return dml(this.id, stmt)
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
    async prepare(stmt:string):Promise<PreparedStatement> {
        let prepared_statement_id:string = await prepare(this.id, stmt)
        return new PreparedStatement(prepared_statement_id);
    }
    async execute_prepare(stmt:string, data: any[]) {
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
    execute_batch():any[] {
        return execute_batch(this.id)
    }
    drop () {
        return addon.dropStatement(this.id)
    }
}

