<div align='center'><h1> raf </h1></div>
<div align='center'><i> redact all files!!!<p>(actually only one type of file, lol) </i></div>

<p>

A command line interface tool that helps you to redact text that matches a certain regex in a document.

It is inspired by the following:

1. [PyRedactKit](https://github.com/brootware/PyRedactKit)
2. [py-redact](https://github.com/datumbrain/py-redact/blob/master/py_redact/docx_redactor.py)

# Usage
**Disclaimer**:
Sadly, for now, you can only redact `.txt` file. `lopdf` crate was used to redact `.pdf` files but it is a failure, the codes still remain, I may work on it some day in the future to see how it can be done.

1. Git clone this repo

Example:
```
git clone https://github.com/jymchng/raf.git
```

2. `cargo run` it

Example, to redact all sgNRIC and emails text for all files in a folder use:
```
cargo run -- folder ./tests/test_files -t sgNRIC emails
```
Note the `folder` subcommand, 
Or you can use:
```rust
cargo run -- folder ./tests/test_files -t sgNRIC email
```
This is because `email` and `emails` are categorized under the list of `types` in the `patterns.json` file, i.e.:

```json
{
    "pattern": "([a-z0-9_+]([a-z0-9_+.\\-]*[a-z0-9_+\\-])?)@([a-z0-9]+([\\-\\.]{1}[a-z0-9]+)*\\.[a-z]{2,6})",
    "type": [
        "email",
        "emails"
    ]
}
```


cargo run -- file ./tests/test_files/file1.txt -t sgNRIC email