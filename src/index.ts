const addon = require("../native");
import {promisify} from "util";

const createClientProm = promisify(addon.createClient);
const statement = promisify(addon.statement);
const multiple_statements_ignore_err = promisify(addon.multiple_statements_ignore_err);
const prepare = promisify(addon.prepare);
const execute_batch = promisify(addon.execute_batch);

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
export async function createClient(opt: IConnectionParameters): Promise<Connection> {
    const client_id = await createClientProm(opt);
    return new Connection(client_id);
}

/**
 * Returns the number of the internally hold connection. The connection does live in the native code and has to be closed by the caller.
 */
export function get_num_connections(): number {
    return addon.get_num_connections()
}

/**
 * Returns the number of the internally hold prepared statements. Prepared statements live in the native code and have to be removed by the caller, when finished using.
 */
export function get_num_prepared_statements(): number {
    return addon.get_num_prepared_statements()
}

export class Connection {
    private id: string;
    constructor(id: string) {
        this.id = id;
    }

    public close() {
        return addon.dropClient(this.id);
    }
    public statement(stmt: string): Promise<any[]> {
        return statement(this.id, stmt);
    }
    public multiple_statements_ignore_err(stmt: string[]) {
        return multiple_statements_ignore_err(this.id, stmt);
    }
    public set_auto_commit(bool: boolean) {
        return addon.set_auto_commit(this.id, bool);
    }
    public is_auto_commit() {
        return addon.is_auto_commit(this.id);
    }
    public set_fetch_size(val: number) {
        return addon.set_fetch_size(this.id, val);
    }
    public get_lob_read_length(): number {
        return addon.get_lob_read_length(this.id);
    }
    public set_lob_read_length(val: number) {
        return addon.set_lob_read_length(this.id, val);
    }
    public get_call_count(): number {
        return addon.get_call_count(this.id);
    }
    public set_application_user(val: string) {
        return addon.set_application_user(this.id, val);
    }
    public set_application_version(val: number) {
        return addon.set_application_version(this.id, val);
    }
    public set_application_source(val: string) {
        return addon.set_application_source(this.id, val);
    }

    /**
     * Creates a new prepared statement.
     *
     * @remarks
     * Don't forget to drop() the prepared statement.
     *
     */
    public async prepare(stmt: string): Promise<PreparedStatement> {
        const prepared_statement_id: string = await prepare(this.id, stmt);
        return new PreparedStatement(prepared_statement_id);
    }

    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    public async prepare_execute(stmt: string, data: any[]): Promise<any[]> {
        const prep = await this.prepare(stmt);
        try {
            prep.add_batch(data);
        } catch (e) {
            prep != null && prep.drop();
            throw(e);
        }
        return prep.execute_batch_and_drop()
    }
    /**
     * Creates a new prepared statemen, binds values and drops the prepared statement..
     */
    public async execute_prepare(stmt: string, data: any[]): Promise<any[]> {
        return this.prepare_execute(stmt, data);
    }
    public commit() {
        return addon.commit(this.id);
    }
    public rollback() {
        return addon.rollback(this.id);
    }
}

export class PreparedStatement {
    private id: string;
    constructor(id: string) {
        this.id = id;
    }
    public add_batch(data: any[]) {
        return addon.add_row(this.id, data);
    }
    public execute_batch(): Promise<any[]> {
        return execute_batch(this.id);
    }
    public async execute_batch_and_drop(): Promise<any[]> {
        try {
            let res = await this.execute_batch();
            this.drop();
            return res;
        } catch (e) {
            this.drop()
            throw(e);
        }
    }
    public drop() {
        return addon.dropStatement(this.id);
    }
}
