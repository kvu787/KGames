# Json

A JSON parser and writer. The parser follows [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259),
and says where in the text a mistake is, by line and column. KGames programs read their settings
files with it, and write them, and what scripts save, with the writer.
It's written for KGames, which depends on no crates besides Microsoft's `windows-core`.

`json::parse` returns a `Value`. Objects keep their members in the file's order, and
`Value::describe` names a value for messages like `must be a number, not "1280"`.

`Value::to_json` writes a value as text, with each object member on its own line, indented by two
spaces a level, and numbers that parse back exactly. `json::object` makes an object of members.
