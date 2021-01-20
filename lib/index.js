var addon = require('../native');

console.log("session created");
const session = addon.session_new('42', function (event_string) {
  console.log("js callback: " + event_string);
  const event = JSON.parse(event_string);
  if (event.Greeting === '42') {
    console.log("done");
    cb();
  } else {
    console.log("Error");
    new Error('id did not match');
  }
});

addon.session_assign(session);