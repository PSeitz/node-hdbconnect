var addon = require('../native');



exports.createClient = function(opt, cb) {
    addon.createClient(opt, (err, conn_id) => {
        cb(err, new Connection(conn_id))
    })
}

function Connection(id) {
    this.drop = ()=>{addon.dropClient(id)}

    this.query = (stmt, cb)=>{return addon.query(id, stmt, cb)}
}

// module.exports = addon;