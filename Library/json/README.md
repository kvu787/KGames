# Json

A JSON parser that follows [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259), and says where
in the text a mistake is, by line and column. KGames programs read their settings files with it.
It's written for KGames, which depends on no crates besides Microsoft's `windows-core`.

`json::parse` returns a `Value`. Objects keep their members in the file's order, and
`Value::describe` names a value for messages like `must be a number, not "1280"`.
