const hdb = require('.');
const fs = require("fs");
async function getConnection() {
    try {
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



// TINYINT, SMALLINT, INTEGER, BIGINT, SMALLDECIMAL, DECIMAL, REAL, DOUBLE
// BOOLEAN
// VARCHAR, NVARCHAR, ALPHANUM, SHORTTEXT
// VARBINARY
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
});

// DATE, TIME, SECONDDATE, TIMESTAMP
test('test date data types', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE date_types","CREATE COLUMN TABLE date_types (COL_DATE DATE, COL_TIME TIME, COL_SECONDDATE SECONDDATE, COL_TIMESTAMP TIMESTAMP, COL_DATE_NN DATE, COL_TIME_NN TIME, COL_SECONDDATE_NN SECONDDATE, COL_TIMESTAMP_NN TIMESTAMP)"]);

    const arr = new Uint16Array(2);
    arr[0] = 5000;
    arr[1] = 4000;
    const buf = Buffer.from(arr.buffer);

    await connection.statement("INSERT INTO date_types (COL_DATE, COL_TIME, COL_SECONDDATE, COL_TIMESTAMP, COL_DATE_NN, COL_TIME_NN, COL_SECONDDATE_NN, COL_TIMESTAMP_NN) values('2018-08-12','23:59:59','2018-08-12 10:00:00','2018/01/02 10:00:00', '2018-08-12','23:59:59','2018-08-12 10:00:00','2018/01/02 10:00:00') ");
    var res = await connection.statement("SELECT COUNT(*) FROM date_types");
    expect(res).toEqual([{"COUNT(*)": 1}])

    var res = await connection.statement("SELECT * FROM date_types");
    expect(res).toEqual([{
        "COL_DATE": "2018-08-12",
        "COL_DATE_NN": "2018-08-12",
        "COL_SECONDDATE": "2018-08-12T10:00:00",
        "COL_SECONDDATE_NN": "2018-08-12T10:00:00",
        "COL_TIME": "23:59:59",
        "COL_TIMESTAMP": "2018-01-02T10:00:00.0000000",
        "COL_TIMESTAMP_NN": "2018-01-02T10:00:00.0000000",
        "COL_TIME_NN": "23:59:59",
    }]);
    // let prep = await connection.prepare("INSERT INTO date_types (col_blob, col_clob, col_nclob, col_text) values(?,?,?,?) ");
    // prep.add_batch([buf, "test", "test", "nice"]);

    // let batch_res = await prep.execute_batch();
    // prep.drop();

    // expect(await connection.statement("SELECT col_blob FROM date_types")).toEqual([ { COL_BLOB: null }, { COL_BLOB: buf } ]);
    // expect(await connection.statement("SELECT col_nclob FROM date_types")).toEqual([ { COL_NCLOB: 'oge' }, { COL_NCLOB: 'test' } ]);
    // expect(await connection.statement("SELECT col_text FROM date_types")).toEqual( [ { COL_TEXT: null }, { COL_TEXT: 'nice' } ]);
});

test('test call procedure', async () => {

    let prep = await connection.prepare("CALL ESH_SEARCH (?,?) ");
    prep.add_batch(["[\"/$all/?$apply=filter((Search.search(query='*')))&$count=true&estimate=true&whyfound=true&$top=10&facets=all\"]"]);

    let batch_res = await prep.execute_batch();
    prep.drop();

});

test('test error', async () => {

    await expect(connection.prepare("INSERT INTO not_there (col_blob,t) values(?) ")).rejects.toThrow('Could not find');

    await connection.multiple_statements_ignore_err(["DROP TABLE test_err","CREATE COLUMN TABLE test_err (col_int INT)"]);

    let prep = await connection.prepare("INSERT INTO test_err (col_int) values(?) ");
    console.log(prep)
    expect(()=>{prep.add_batch(['NOT_INT'])}).toThrow('cannot be parsed into SQL type some integer');

    prep.drop()

});

test('bytes to nclob', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE lob_types","CREATE COLUMN TABLE lob_types (col_nclob NCLOB, col_nclob_nn NCLOB not null)"]);

    const arr = new Uint16Array(2);
    arr[0] = 5000;
    arr[1] = 4000;
    const buf = Buffer.from(arr.buffer);

    let prep = await connection.prepare("INSERT INTO lob_types (col_nclob, col_nclob_nn) values(?,?) ");
    expect(()=>{prep.add_batch([buf, buf])}).toThrow('invalid utf-8');

    //test buffer
    fs.writeFileSync("test_text", "test");
    var data = fs.readFileSync('test_text');
    prep.add_batch([data, data]);

    let batch_res = await prep.execute_batch();
    prep.drop();

    var res = await connection.statement("SELECT * FROM lob_types");
    expect(res).toEqual([{"COL_NCLOB": "test","COL_NCLOB_NN": "test"}]);
});

