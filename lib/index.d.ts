declare var addon: any;
declare const promisify: any;
declare const createClientProm: any;
declare const statement: any;
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
declare function createClient(opt: ConnectionParameters): Promise<Connection>;
declare class Connection {
    private id;
    constructor(id: string);
    close(): any;
    statement(stmt: string): Promise<any[]>;
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
    /**
     * Creates a new prepared statement.
     *
     * @remarks
     * Don't forget to drop() the prepared statement.
     *
     */
    prepare(stmt: string): Promise<PreparedStatement>;
    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    prepare_execute(stmt: string, data: any[]): Promise<any[]>;
    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    execute_prepare(stmt: string, data: any[]): Promise<any[]>;
    commit(): any;
    rollback(): any;
}
declare class PreparedStatement {
    private id;
    constructor(id: string);
    add_batch(data: any[]): any;
    execute_batch(): Promise<any[]>;
    drop(): any;
}
