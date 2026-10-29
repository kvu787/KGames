# Style

For single and double quotes, only use the ASCII forms: ', "
Never use these: “, ”, ‘, ’, etc.

# Application compatibility

Do not attempt to maintain any sort of application compatibility between different commits of the repo. This creates unwanted complexity.

# Development platform compatibility

Support Windows 11 x64 as the only development platform.

# Target platform compatibility

Support Windows 11 x64 as the only target platform.

# Folder and file naming

This only applies to things that we have the freedom to name as wanted.
Use CamelCase.
Use complete proper words. Don't use typical shortenings. Good: Source, Documentation. Bad: src, docs.

Exceptions: README.md, LICENSE.md, TODO.md, README.txt, LICENSE.txt, TODO.txt, AGENTS.md

# Conversations

Record verbatim and commit all conversations between the user and AI (meaning all user prompts and AI responses) in a folder named `Conversations` located at the root of this Git repo.
Record and commit at the end of every turn.
Use one file per conversation.
Prefix these files with the PowerShell evaluation of `[DateTime]::UtcNow.ToString("yyyy-MM-dd_HH-mm") + "_UTC"`.
Separate commits that record conversations from other commits.
Prefix these commits with `[cnv]`.
The conversation file name must also include a human-readable subject line.
So, the format for the conversation file name is: `<timestamp> <subject>.md`.

If the conversation begins with `dnr`, then do not record the conversation.

# PowerShell

All PowerShell scripts must:

- Run on Powershell 7 or higher
- Use `Set-StrictMode -Version Latest`
- Use `$ErrorActionPreference = 'Stop'`

## Dependencies

You can add any dependencies that are scoped to the repo.
You must not add any dependencies that have a scope that is broader than repo-scope.
If you want to add and use any dependencies that have a scope that is broader than repo-scope, you must request this from the user.

# Git

When implementing stuff, avoid difficult-to-review "mega-commits".
When it makes sense, split work into multiple commits to make it easier to review.
Separate commits that record conversations from other commits.

# Markdown

Tables in Markdown must be padded and aligned in a way to make them easy to read in a plaintext editor, not only in a Markdown viewer.
Any mathematical notation in Markdown files (LaTeX, KaTeX, MathJax, etc) must display properly in VSCode's Markdown previewer and GitHub.com's Markdown displayer.
