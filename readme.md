| Prefix | Type          | Example                            |
| ------ | ------------- | ---------------------------------- |
| `+`    | Simple String | `+OK\r\n`                          |
| `-`    | Error         | `-ERR unknown command\r\n`         |
| `:`    | Integer       | `:1000\r\n`                        |
| `$`    | Bulk String   | `$5\r\nhello\r\n`                  |
| `*`    | Array         | `*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n` |
