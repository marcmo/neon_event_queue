var addon = require('../native');

console.log(addon.hello());

let session = new addon.RustSession();
console.log("session created");
session.foo();
session.async_operation();