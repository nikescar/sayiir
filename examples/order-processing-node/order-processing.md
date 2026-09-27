flowchart TD
    Start[order-processing]
    Task0[validate-order]
    Start --> Task0
    Task1[send-confirmation]
    Task0 --> Task1
    Task1 --> End[Done]
