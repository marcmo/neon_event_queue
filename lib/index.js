var addon = require('../native');

console.log("session created");
const session = addon.session_new('42', function (greeting) {
  console.log("js callback: " + greeting);
  if (greeting === '42') {
    cb();
  } else {
    new Error('id did not match');
  }
});

addon.session_assign(session);