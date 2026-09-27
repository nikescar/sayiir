flowchart TD
    validate-order[validate-order]
    send-confirmation[send-confirmation]
    validate-order --> send-confirmation

%%% validate-order (node)
%%% Entry: validateOrder
%%% Dependencies: {"sayiir":"latest"}
```node
import { createServer } from "node:http";

const PORT = 3000;

function validateOrder(order) {
  if (order.amount <= 0) throw new Error("Invalid amount");
  if (!order.customerEmail) throw new Error("Missing customer email");
  return { ...order, validated: true };
}
```

%%% send-confirmation (node)
%%% Entry: sendConfirmation
%%% Dependencies: {"sayiir":"latest"}
```node
import { createServer } from "node:http";

const PORT = 3000;

function sendConfirmation(shipment) {
    // In production: send email via SendGrid, Postmark, etc.
    return `Order shipped via ${shipment.carrier}, tracking: ${shipment.trackingNumber}`;
  }
```
