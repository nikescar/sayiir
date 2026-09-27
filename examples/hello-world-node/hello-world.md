flowchart TD
    Start[welcome]
    Task0[fetch-user]
    Start --> Task0
    Task1[send-email]
    Task0 --> Task1
    Task1 --> End[Done]
