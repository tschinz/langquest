---
id          = "sequence_login"
name        = "Login Sequence Diagram"
language    = "plantuml"
difficulty  = 2
description = "Draw a PlantUML sequence diagram of a simple login flow."
topics      = ["plantuml", "sequence-diagram", "uml"]
---

# Login Sequence Diagram

Complete `main.puml` so it describes the following login flow as a **sequence
diagram**:

1. The **User** sends `login(user, pass)` to the **Browser**.
2. The **Browser** sends `POST /login` to the **Server**.
3. The **Server** replies to the **Browser** with `200 OK`.
4. The **Browser** shows `welcome` to the **User**.

## How it is graded

Every time you save, the diagram is rendered to `main.png` (opened automatically
the first time). Your diagram is scored by **fuzzy similarity** to the reference
solution — the participants and messages must match, but ordering and minor
formatting differences are tolerated.
