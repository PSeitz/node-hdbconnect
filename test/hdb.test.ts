import {createClient, Connection, get_num_connections, get_num_prepared_statements} from '../src/index'
const fs = require("fs");

async function getConnection() {
    try {
        var fs = require("fs");
        var connection_param = JSON.parse(fs.readFileSync("connection.json"));
        let connection = await createClient(connection_param);
        return connection;
    } catch(e) {
        console.log(e);
    }
}

let connection:Connection;

beforeAll(async () => {
    connection = await getConnection();
});

afterAll(async () => {
    connection.close();
    expect(get_num_connections()).toEqual(0)
    expect(get_num_prepared_statements()).toEqual(0)
});

test('prepfail ', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab ( f1 INT primary key, f2 NVARCHAR(100))"]);

    let insert_stmt = await connection.prepare("INSERT INTO tab (f1, f2) values(?,?) ");
    insert_stmt.add_batch([11, "test text blubla1"]);
    insert_stmt.add_batch([12, "test text blubla2"]);
    insert_stmt.add_batch([13, "test text blubla3"]);
    insert_stmt.add_batch([14, "test text blubla4"]);
    insert_stmt.add_batch([15, "test text blubla5"]);
    await insert_stmt.execute_batch_and_drop();

    insert_stmt = await connection.prepare("INSERT INTO tab (f1, f2) values(?,?) ");
    insert_stmt.add_batch([11, "test text blubla1"]);
    insert_stmt.add_batch([12, "test text blubla2"]);
    insert_stmt.add_batch([13, "test text blubla3"]);
    insert_stmt.add_batch([14, "test text blubla4"]);
    insert_stmt.add_batch([15, "test text blubla5"]);

    await expect(insert_stmt.execute_batch_and_drop()).rejects.toEqual(new Error('unique constraint violated")'));

});

test('insert ', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (ID INT, NAME NVARCHAR (10))"]);

    let prep = await connection.prepare("INSERT INTO tab (ID,NAME) values(?,?) ");
    prep.add_batch([10, "nice"]);
    prep.add_batch([11, undefined]);
    prep.add_batch([undefined, undefined]);

    let batch_res = await prep.execute_batch_and_drop();

    var res = await connection.statement("SELECT COUNT(*) FROM tab");
    expect(res).toEqual([{"COUNT(*)": 3}]);
    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");

    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":10}]);
    var res = await connection.statement(`UPDATE TAB
        SET ID = '12'
        WHERE NAME = 'nice';`);

    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":12}]);

    await connection.statement("DROP TABLE tab");

    await expect(connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'")).rejects.toEqual(new Error('DbError(error [code: 259, sql state: HY000] at position 15: "invalid table name:  Could not find table/view TAB in schema SYSTEM: line 1 col 16 (at pos 15)")'));
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
    var res = await connection.statement(`UPDATE TAB
        SET ID = '12'
        WHERE NAME = 'nice';`);

    var res = await connection.statement("SELECT ID FROM tab WHERE NAME = 'nice'");
    expect(res).toEqual([{"ID":12}]);

    await connection.statement("DROP TABLE tab");

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

    let prep = await connection.prepare_execute("INSERT INTO lob_types (col_blob, col_clob, col_nclob, col_text) values(?,?,?,?) ", [buf, "test", "test", "nice"]);

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


test('update with prepared statement', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE TASK_LISTS"]);
    await connection.statement("CREATE COLUMN TABLE TASK_LISTS (ID BIGINT, JOB_ID NVARCHAR(100), STEP_STATUS NVARCHAR(100), TASK_NAME NVARCHAR(100), MARKED_HARVESTED INT)");

    await connection.statement("INSERT INTO TASK_LISTS (ID, JOB_ID, STEP_STATUS,TASK_NAME, MARKED_HARVESTED) values(4569145722365,'JOB_9000','success','nice1_task', 0) ");
    await connection.statement("INSERT INTO TASK_LISTS (ID, JOB_ID, STEP_STATUS,TASK_NAME, MARKED_HARVESTED) values(0000000000000,'JOB_8000','running','nice2_task', 0) ");
    await connection.statement("INSERT INTO TASK_LISTS (ID, JOB_ID, STEP_STATUS,TASK_NAME, MARKED_HARVESTED) values(1000000000000,'JOB_7000','stopped','nice3_task', 0) ");

    let query = `UPDATE TASK_LISTS
        SET MARKED_HARVESTED = 2
       WHERE ID IN (?)
       AND (STEP_STATUS = 'success'
       OR STEP_STATUS = 'failed')
       AND JOB_ID = ?`;

    let res = await connection.prepare_execute(query, [4569145722365, "JOB_9000"]);
    expect(res).toEqual([1]);

    res = await connection.prepare_execute(query, [4569145722365, "NO_HIT"]);
    expect(res).toEqual([0]);

});

test('test parameter binding corner cases', async () => {
    await connection.multiple_statements_ignore_err(["DROP TABLE TASK_LISTS"]);
    await connection.statement("CREATE COLUMN TABLE TASK_LISTS (JOB_ID BIGINT, STEP_STATUS NVARCHAR(100), TASK_NAME NVARCHAR(100))");

    await connection.statement("INSERT INTO TASK_LISTS (JOB_ID, STEP_STATUS,TASK_NAME) values(1,'initial','nice1_task') ");
    await connection.statement("INSERT INTO TASK_LISTS (JOB_ID, STEP_STATUS,TASK_NAME) values(2,'running','nice2_task') ");
    await connection.statement("INSERT INTO TASK_LISTS (JOB_ID, STEP_STATUS,TASK_NAME) values(3,'stopped','nice3_task') ");

    let query_no_bound_params     = "SELECT COUNT(JOB_ID) AS TASKCOUNT FROM TASK_LISTS WHERE \"STEP_STATUS\" = 'initial' OR \"STEP_STATUS\"='running'";
    let query_single_bound_params = "SELECT COUNT(JOB_ID) AS TASKCOUNT FROM TASK_LISTS WHERE \"STEP_STATUS\" = ? OR \"STEP_STATUS\"='running'";

    { //statement, no variables
        let res = await connection.statement(query_no_bound_params);
        expect(res).toEqual([{"TASKCOUNT": 2}]);
    }

    { //ERROR: invalid statement, unbound variables
        await expect(connection.statement(query_single_bound_params)).rejects.toThrow('not all variables bound');
    }

    { //prepare without parameters
        let prep = await connection.prepare(query_no_bound_params);
        let res = await prep.execute_batch();
        prep.drop()
        expect(res).toEqual([{"TASKCOUNT": 2}]);
    }

    { //prepare without parameters, empty add_batch
        let res = await connection.prepare_execute(query_no_bound_params, [])
        expect(res).toEqual([{"TASKCOUNT": 2}]);
    }

    { //ERROR: prepare without parameters, undefined add_batch
        await expect(connection.prepare_execute(query_no_bound_params, [undefined, undefined])).rejects.toThrow('too many parameters');
    }

    { //prepare with correctly bound parameter
        let res = await connection.prepare_execute(query_single_bound_params, ['initial'])
        expect(res).toEqual([{"TASKCOUNT": 2}]);
    }

    { //ERROR: prepare with too much bound parameter
        await expect(connection.prepare_execute(query_single_bound_params, ['initial','running'])).rejects.toThrow('too many parameters');
    }

    { //ERROR: prepare with not enough bound parameter
        await expect(connection.prepare_execute(query_single_bound_params, [])).rejects.toThrow('some paramters are missing');
    }


});
