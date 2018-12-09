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



test('insert ', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (ID INT, NAME NVARCHAR (10))"]);

    let prep = await connection.prepare("INSERT INTO tab (ID,NAME) values(?,?) ");
    prep.add_batch([10, "nice"]);
    prep.add_batch([11, undefined]);
    prep.add_batch([undefined, undefined]);

    let batch_res = await prep.execute_batch();
    prep.drop();

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

test('statement can handle everything', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (ID INT, NAME NVARCHAR (10))"]);

    await connection.statement("INSERT INTO tab (ID,NAME) values(10,'nice') ");
    await connection.statement("INSERT INTO tab (ID,NAME) values(11,NULL) ");
    await connection.statement("INSERT INTO tab (ID,NAME) values(NULL,NULL) ");

    var res = await connection.statement("SELECT COUNT(*) FROM tab");
    expect(res).toEqual([{"COUNT(*)": 3}]);
    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");

    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":10}]);
    var res = await connection.dml(`UPDATE TAB
        SET ID = '12'
        WHERE NAME = 'nice';`);

    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":12}]);

    await connection.exec("DROP TABLE tab");

    await expect(connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'")).rejects.toEqual(new Error('DbError(error [code: 259, sql state: HY000] at position 15: "invalid table name:  Could not find table/view TAB in schema SYSTEM: line 1 col 16 (at pos 15)")'));
});


// DATE, TIME, SECONDDATE, TIMESTAMP
// TINYINT, SMALLINT, INTEGER, BIGINT, SMALLDECIMAL, DECIMAL, REAL, DOUBLE
// BOOLEAN
// VARCHAR, NVARCHAR, ALPHANUM, SHORTTEXT
// VARBINARY
// BLOB, CLOB, NCLOB, TEXT
// ARRAY
// ST_GEOMETRY, ST_POINT


test('test lob data types', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE lob_types","CREATE COLUMN TABLE lob_types (col_blob BLOB, col_clob CLOB, col_nclob NCLOB, col_text TEXT)"]);

    const arr = new Uint16Array(2);
    arr[0] = 5000;
    arr[1] = 4000;
    const buf = Buffer.from(arr.buffer);

    await connection.statement("INSERT INTO lob_types (col_nclob) values('oge') ");
    let prep = await connection.prepare("INSERT INTO lob_types (col_blob, col_clob, col_nclob, col_text) values(?,?,?,?) ");
    prep.add_batch([buf, "test", "test", "nice"]);

    let batch_res = await prep.execute_batch();
    prep.drop();

    var res = await connection.statement("SELECT COUNT(*) FROM lob_types");
    expect(res).toEqual([{"COUNT(*)": 2}]);
    expect(await connection.statement("SELECT col_blob FROM lob_types")).toEqual([ { COL_BLOB: null }, { COL_BLOB: buf } ]);
    expect(await connection.statement("SELECT col_nclob FROM lob_types")).toEqual([ { COL_NCLOB: 'oge' }, { COL_NCLOB: 'test' } ]);
    expect(await connection.statement("SELECT col_text FROM lob_types")).toEqual( [ { COL_TEXT: null }, { COL_TEXT: 'nice' } ]);
    // var res = await connection.statement("SELECT * FROM lob_types");
    // console.log(res)
    // var res = await connection.statement("SELECT ID FROM lob_types WHERE NAME = 'nice'");
    // expect(res).toEqual([{"ID":10}]);
    // var res = await connection.dml(`UPDATE lob_types
    //     SET ID = '12'
    //     WHERE NAME = 'nice';`);

    // var res = await connection.statement("SELECT ID FROM lob_types WHERE NAME = 'nice'");
    // expect(res).toEqual([{"ID":12}]);

    // await connection.exec("DROP TABLE lob_types");

    // await expect(connection.statement("SELECT ID FROM lob_types WHERE NAME = 'nice'")).rejects.toEqual(new Error('DbError(error [code: 259, sql state: HY000] at position 15: "invalid table name:  Could not find table/view TAB in schema SYSTEM: line 1 col 16 (at pos 15)")'));
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
