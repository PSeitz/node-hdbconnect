interface IConnectionParameters {
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
 * @param opt - The IConnectionParameters
 * @returns A Promise to a connection handle.
 *
 */
export declare function createClient(opt: IConnectionParameters): Promise<Connection>;
/**
 * Returns the number of the internally hold connection. The connection does live in the native code and has to be closed by the caller.
 */
export declare function get_num_connections(): number;
/**
 * Returns the number of the internally hold prepared statements. Prepared statements live in the native code and have to be removed by the caller, when finished using.
 */
export declare function get_num_prepared_statements(): number;
export declare class Connection {
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
export declare class PreparedStatement {
    private id;
    constructor(id: string);
    add_batch(data: any[]): any;
    execute_batch(): Promise<any[]>;
    execute_batch_and_drop(): Promise<any[]>;
    drop(): any;
}
export {};
