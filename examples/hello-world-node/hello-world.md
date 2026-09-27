flowchart TD
    fetch-user[fetch-user]
    send-email[send-email]
    fetch-user --> send-email

%%% fetch-user (node)
%%% Entry: fetchUser
%%% Dependencies: {"sayiir":"latest"}
```node
function fetchUser(id) {
  return { id, name: "Alice" };
}
```

%%% send-email (node)
%%% Entry: sendEmail
%%% Dependencies: {"sayiir":"latest"}
```node
function sendEmail(user) {
  return `Sent welcome to ${user.name}`;
}
```
