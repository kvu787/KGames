---
name: bash-heredoc-python
description: Bash heredocs feeding Python fail intermittently on this machine; write the script to the scratchpad and run python -I <file>
metadata:
  node_type: memory
  type: project
  originSessionId: 078024e2-ccf2-4799-98dd-bb98ba890867
  modified: 2026-11-08T02:59:45.246Z
---

Piping Python into `python -I -` through a Bash heredoc (`<<'PYEOF'`) on this machine intermittently fails with "unexpected EOF while looking for matching `''" when the script is long or quote-heavy, even with a quoted delimiter.

**Why:** Git Bash mangles some long heredoc bodies; the failure appears mid-script, so partial edits are a risk only avoided because the script writes files at the end.

**How to apply:** For anything beyond a short snippet, Write the .py file into the session scratchpad and run `python -I <absolute path>`. Keep edit scripts transactional: assert all replacements before writing any file. See also [[light-load-while-gaming]].
