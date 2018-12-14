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
    private connection: Connection;
    private num: number;
    constructor(connection: Connection) {
        this.connection = connection;
    }

    public prepare() {
        return this.connection.statement(`create table FOO_SQUARE${this.num} ( f1 INT primary key, f2 NVARCHAR(100)`)
    }


    public async query() {
        let insert_stmt = await this.connection.prepare(`insert into FOO_SQUARE${this.num} (f1, f2) values(?,?)`);
        insert_stmt.add_batch([11, "test text blubla"]);
        insert_stmt.add_batch([12, "test text blubla"]);
        insert_stmt.add_batch([13, "test text blubla"]);
        insert_stmt.add_batch([14, "test text blubla"]);
        insert_stmt.add_batch([15, "test text blubla"]);
        return insert_stmt.execute_batch_and_drop()
    }


    public cleanup() {
        return this.connection.statement(`drop table FOO_SQUARE${this.num}`);
    }
}

async function startBench(argument) {

    let query_packets:[Packet] = await Promise.all(Array(2).fill().map(async (i)=>{
        let connection = await getConnection();
        return new Packet(connection)
        // return {
        //     connection:connection,
        //     prepare: connection=>{
        //         return connection.statement(`create table FOO_SQUARE${i} ( f1 INT primary key, f2 NVARCHAR(100)`)
        //     },
        //     query: async (connection)=>{
        //         let insert_stmt = await connection.prepare(`insert into FOO_SQUARE${i} (f1, f2) values(?,?)`);
        //         insert_stmt.add_batch([11, "test text blubla"]);
        //         insert_stmt.add_batch([12, "test text blubla"]);
        //         insert_stmt.add_batch([13, "test text blubla"]);
        //         insert_stmt.add_batch([14, "test text blubla"]);
        //         insert_stmt.add_batch([15, "test text blubla"]);
        //         return insert_stmt.execute_batch_and_drop()
        //     },
        //     cleanup: (connection)=>{
        //         return connection.statement(`drop table FOO_SQUARE${i}`);
        //     }
        // }
    }));

    var t0 = performance.now();

    await Promise.all(query_packets.map(el => el.prepare(el.connection)));

    for (let el of Array(10).fill())
    {
        await Promise.all(query_packets.map(el => el.connection.close()));
    }

    await Promise.all(query_packets.map(el => el.cleanup(el.connection)));


    var t1 = performance.now();
    console.log("Call to doSomething took " + (t1 - t0) + " milliseconds.")
    // query_packets.map(el => el.query(el.connection))

    // for (let connection of connections)
    // {
    //     Promise.all([p1, p2, p3])
    //     .then(values => {
    //         console.log(values); // [3, 1337, "foo"]
    //     });
    // }
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
