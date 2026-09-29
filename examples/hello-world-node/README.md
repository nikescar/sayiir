# Welcome Workflow

```mermaid
flowchart TD
    fetch-user[Fetch User]
    send-email[Send Email]
    fetch-user --> send-email
```

## Tasks

### fetch-user

**Language:** node  
**Entry:** fetchUser

```javascript
function fetchUser(id) {
  return { id: id, name: "Alice" };
}
```

### send-email

**Language:** node  
**Entry:** sendEmail

```javascript
function sendEmail(user) {
  return "Sent welcome to " + user.name;
}
```


