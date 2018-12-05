let hdb = require('.');

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

let connection;

beforeAll(async () => {
    connection = await getConnection();
});

afterAll(async () => {
    connection.close();
});



test('adds 1 + 2 to equal 3', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (C1 INT, C2 NVARCHAR (10))"]);

    expect(1+2).toBe(3);
});



test('insert ', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (ID INT, NAME NVARCHAR (10))"]);

    let prep = await connection.prepare("INSERT INTO tab (ID,NAME) values(?,?) ");
    prep.add_batch([10, "nice"]);
    prep.add_batch([11, undefined]);
    prep.add_batch([undefined, undefined]);

    let batch_res = await prep.execute_batch();


    var res = await connection.query("SELECT COUNT(*) FROM tab");
    expect(res).toEqual([{"COUNT(*)": 3}]);
    var res = await connection.query("SELECT ID FROM tab WHERE NAME = 'nice'");

    var res = await connection.query("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":10}]);
    var res = await connection.dml(`UPDATE TAB
        SET ID = '12'
        WHERE NAME = 'nice';`);

    var res = await connection.query("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":12}]);

    await connection.exec("DROP TABLE tab");

    await expect(connection.query("SELECT ID FROM tab WHERE NAME = 'nice'")).rejects.toEqual(new Error('DbError(error [code: 259, sql state: HY000] at position 15: "invalid table name:  Could not find table/view TAB in schema SYSTEM: line 1 col 16 (at pos 15)")'));
});


// let hdb = require('.');

// async function test() {
//     try {
//         var fs = require("fs");
//         var connection_param = JSON.parse(fs.readFileSync("connection.json"));
//         console.log(connection_param)
//         let connection = await hdb.createClient(connection_param);
//         await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (C1 INT, C2 NVARCHAR (10))"]);
//         let prep = await connection.prepare("INSERT INTO tab (C1,C2) values(?,?) ");

//         await connection.dml("INSERT INTO tab (C1,C2) values(NULL, NULL) ");

//         prep.add_batch([10, "nice"]);
//         prep.add_batch([11, undefined]);
//         prep.add_batch([undefined, undefined]);
//         let batch_res = await prep.execute_batch();
//         console.log(batch_res)
//         let res = await connection.query("select * FROM tab");
//         console.log(res)

//     } catch(e) {
//         console.log(e);
//     }
// }
// test();
