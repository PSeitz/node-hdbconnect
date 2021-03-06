# node-hdbconnect

HANA database driver for node - uses [hdbconnect](https://github.com/emabee/rust-hdbconnect)

## Prerequisites

- Rust: `curl https://sh.rustup.rs -sSf | sh`

- Node Build Dependencies: https://guides.neon-bindings.com/getting-started/



## Example

`npm install --save  PSeitz/node-hdbconnect`

```javascript

const hdb = require('node-hdbconnect');
async function test(){

    let connection;
    try{
        connection = await hdb.createClient({
            "host": "ld2512",
            "port": 30515,
            "user": "SYSTEM",
            "password": "manager"
        }
        );

        console.log(await connection.statement("SELECT * FROM DUMMY"))

        await connection.multiple_statements_ignore_err(["DROP TABLE FOO_SQUARE"]);
        await connection.statement("create table FOO_SQUARE ( f1 INT primary key, f2 INT)");

        let insert_stmt = await connection.prepare("insert into FOO_SQUARE (f1, f2) values(?,?)");
        insert_stmt.add_batch([10, 10]);
        insert_stmt.add_batch([11, 20]);
        await insert_stmt.execute_batch();
        insert_stmt.drop();

        console.log(await connection.statement("SELECT * FROM FOO_SQUARE"))

    }catch(err){
        console.log("Error:"+err.message)
    }
    connection!=null && connection.close()
}

test();


```