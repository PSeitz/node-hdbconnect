let hdb = require('.');

// 
hdb.createClient({"host": "","port": 39015 ,"user": "SYSTEM","password": ""}, (err, connection) => {
    if (err) {
    	console.log(err)
    }else{
    	console.log(connection)
    	connection.query("select * from dummy", (err, res) => {
			console.log(res)
    	})
    	// console.log(connection.query("select * from dummy"))
    }
});