---
name: gameinput-docs-markdown
description: "GameInput/GDK docs as Markdown - docs.xbox.com serves .md pages, llms.txt and a 30 MB llms-full.txt; no public docs repo"
metadata:
  node_type: memory
  type: reference
  originSessionId: d9e76e64-856a-4ecb-816a-1c0c55e61b7f
  modified: 2026-11-13T22:30:08.307Z
---

Use the microsoft-docs plugin first (its `microsoft_docs_search` and `microsoft_docs_fetch` tools): checked 2026-11-13, it indexes the GameInput reference and concept pages on learn.microsoft.com/gaming/gdk/docs (`?view=gdk-2610`, plus v0-v2 versions). The user asked whether it was being used, after a research answer that didn't use it.

Microsoft's GDK docs, GameInput included, are also published at docs.xbox.com (devdocs.xbox.com is the same site), and are AI-friendly:

- Any page as Markdown: take the learn.microsoft.com path after `/gaming/gdk/docs/`, put it on `https://docs.xbox.com/` and add `.md`. Example: `https://docs.xbox.com/reference/input/gameinput/gameinput_members.md` (served as text/markdown). The members page links every reference page as .md.
- `https://docs.xbox.com/llms.txt` is an index, but truncated at 100,000 characters, so it has the GameInput overview pages and none of the API reference.
- `https://devdocs.xbox.com/llms-full.txt` is every page in one file, about 30 MB (checked 2026-11-09: 15,469 pages, 252 under reference/input/gameinput). Each page starts with `# Title` then `Source: <url>`. Too big to read whole; download and grep or split it.
- No public Git repo of the GDK docs: MicrosoftDocs has xbox-live-docs, playfab-docs, azure-gaming-docs, but not GDK. `microsoft/GDK` on GitHub holds GDK release zips, not docs.

Related: [[light-load-while-gaming]]
