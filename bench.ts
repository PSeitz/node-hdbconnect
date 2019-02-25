import { Connection } from "./lib";

const hdb = require('.');
const {
  performance
} = require('perf_hooks');

async function getConnection() {
    try {
        var fs = require("fs");
        var connection_param = JSON.parse(fs.readFileSync("connection.json"));
        let connection = await hdb.createClient(connection_param);
        return connection;
    } catch(e) {
        console.log(e);
    }
}

class Packet{
    public connection: Connection;
    private num: number;
    constructor(connection: Connection, num:number) {
        this.connection = connection;
        this.num = num;
    }

    public prepare() {
        let query = `create table FOO_SQUARE${this.num} ( f1 INT primary key, f2 NVARCHAR(100))`;
        return this.connection.statement(query)
    }

    public async query() {
        console.log("prepare");
        
        let insert_stmt = await this.connection.prepare(`insert into FOO_SQUARE${this.num} (f1, f2) values(?,?)`);
        console.log("add_batch");
        insert_stmt.add_batch([11, "test text blubla1"]);
        insert_stmt.add_batch([12, "test text blubla2"]);
        insert_stmt.add_batch([13, "test text blubla3"]);
        insert_stmt.add_batch([14, "test text blubla4"]);
        insert_stmt.add_batch([15, "test text blubla5"]);
        console.log("execute_batch");
        let res = await insert_stmt.execute_batch_and_drop()
        console.log("yop" + res);
        return res;
        // return insert_stmt.execute_batch_and_drop()
    }

    // public query() {
    //     return this.connection.statement(`insert into FOO_SQUARE${this.num} (f1, f2) values(13, 'test text blubla')`)
    // }

    public cleanup() {
        return this.connection.multiple_statements_ignore_err([`drop table FOO_SQUARE${this.num}`]);
    }
}

async function startBench() {
    let nums = [...Array(1)].map((_,i) => i);
    let query_packets:Packet[] = await Promise.all(nums.map(async (i)=>{
        let connection = await getConnection();
        return new Packet(connection, i)
    }));

    var t0 = performance.now();
    await Promise.all(query_packets.map(el => el.cleanup()));
    await Promise.all(query_packets.map(el => el.prepare()));

    // for (let el in [...Array(2)])
    // {
    //     console.log(el);
    //     let res = await Promise.all(query_packets.map(el => el.query()));
    //     console.log(res);
        
    // }

    await Promise.all(query_packets.map(el => el.query()));
    await Promise.all(query_packets.map(el => el.query()));

    console.log("el");
    await Promise.all(query_packets.map(el => el.query()));

    await Promise.all(query_packets.map(el => el.cleanup()));
    await Promise.all(query_packets.map(el => el.connection.close()));

    var t1 = performance.now();
    console.log("Call to doSomething took " + (t1 - t0) + " milliseconds.")
}

startBench();

// connection = await hdb.createClient({
//             "host": "ld2512",
//             "port": 30515,
//             "user": "SYSTEM",
//             "password": "manager"
//         }
//         );

//         console.log(await connection.statement("SELECT * FROM DUMMY"))

//         await connection.multiple_statements_ignore_err(["DROP TABLE FOO_SQUARE"]);
//         await connection.statement("create table FOO_SQUARE ( f1 INT primary key, f2 INT)");

//         let insert_stmt = await connection.prepare("insert into FOO_SQUARE (f1, f2) values(?,?)");
//         insert_stmt.add_batch([10, 10]);
//         insert_stmt.add_batch([11, 20]);
//         await insert_stmt.execute_batch();
//         insert_stmt.drop();

//         console.log(await connection.statement("SELECT * FROM FOO_SQUARE"))
