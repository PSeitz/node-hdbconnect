let hdb = require('.');


hdb.createClient({"host": "","port": 0,"user": "","password": ""}, (err, connection) => {
    console.log(connection)

    console.log(connection.query("select * from dummy"))
});