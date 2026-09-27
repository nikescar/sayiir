%%% Summary: welcome
flowchart TD
    fetch-user[fetch-user]
    send-email[send-email]
    fetch-user --> send-email

%%% fetch-user (node)
%%% Entry: fetchUser
%%% Dependencies: {}
```node
function fetchUser(id) {
  return { id: id, name: "Alice" };
}
```

%%% send-email (node)
%%% Entry: sendEmail
%%% Dependencies: {}
```node
function sendEmail(user) {
  return "Sent welcome to " + user.name;
}
```
