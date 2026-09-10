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

Every time you save, the diagram is rendered to `main.png` (opened automatically the first time). Your diagram is scored by **keywords**: each entry in the solution's `keywords` list is matched against your diagram (case-insensitively). A keyword selects how it is matched by its wrapper:

- `s/PATTERN/` — `PATTERN` is a regular expression.
- `w/TEXT/` — substring search for `TEXT` with all spaces and tabs ignored, so `login(user, pass)` also matches `login ( user,pass )`.
- anything else — plain substring search, matched verbatim.

Reach the pass threshold to complete the exercise.
