let hdb = require('.');


async function test() {
    try {
        let connection = await hdb.createClient({"host": "172.31.217.8","port": 39015 ,"user": "SYSTEM","password": ""});
        await connection.multiple_statements_ignore_err(["DROP TABLE tab","CREATE COLUMN TABLE tab (C1 INT, C2 NVARCHAR (10))"]);
        let prep = await connection.prepare("INSERT INTO tab (C1,C2) values(?,?) ");

        await connection.dml("INSERT INTO tab (C1,C2) values(NULL, NULL) ");

        prep.add_batch([10, "nice"]);
        prep.add_batch([11, undefined]);
        prep.add_batch([undefined, undefined]);
        let batch_res = await prep.execute_batch();
        console.log(batch_res)
        let res = await connection.query("select * FROM tab");
        console.log(res)

    } catch(e) {
        console.log(e);
    }
}
test();
