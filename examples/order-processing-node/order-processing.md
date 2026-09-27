%%% Summary: order-processing
flowchart TD
    validate-order[validate-order]
    send-confirmation[send-confirmation]
    validate-order --> send-confirmation

%%% validate-order (node)
%%% Entry: validateOrder
%%% Dependencies: {}
```node
function validateOrder(order) {
  if (order.amount <= 0) throw new Error("Invalid amount");
  if (!order.customerEmail) throw new Error("Missing customer email");
  return { amount: order.amount, customerEmail: order.customerEmail, validated: true };
}
```

%%% send-confirmation (node)
%%% Entry: sendConfirmation
%%% Dependencies: {}
```node
function sendConfirmation(order) {
  return "Order confirmed for " + order.customerEmail + ", amount: $" + order.amount;
}
```
