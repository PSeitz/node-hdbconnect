declare var addon: any;
declare const promisify: any;
declare const createClient: any;
declare const query: any;
declare const statement: any;
declare const exec: any;
declare const dml: any;
declare const multiple_statements_ignore_err: any;
declare const prepare: any;
declare const execute_batch: any;
interface ConnectionParameters {
    host: string;
    port: number;
    user: string;
    password: string;
    tls?: string;
}
declare function createClientWrap(opt: ConnectionParameters): Promise<Connection>;
declare class Connection {
    private id;
    constructor(id: string);
    close(): any;
    query(stmt: string): any[];
    statement(stmt: string): any[];
    exec(stmt: string): any[];
    dml(stmt: string): any[];
    multiple_statements_ignore_err(stmt: string[]): any;
    set_auto_commit(bool: boolean): any;
    is_auto_commit(): any;
    set_fetch_size(val: number): any;
    get_lob_read_length(): number;
    set_lob_read_length(val: number): any;
    get_call_count(): number;
    set_application_user(val: string): any;
    set_application_version(val: number): any;
    set_application_source(val: string): any;
    prepare(stmt: string): Promise<PreparedStatement>;
    execute_prepare(stmt: string, data: any[]): Promise<any[]>;
    commit(): any;
    rollback(): any;
}
declare class PreparedStatement {
    private id;
    constructor(id: string);
    add_batch(data: any[]): any;
    execute_batch(): any[];
    drop(): any;
}
