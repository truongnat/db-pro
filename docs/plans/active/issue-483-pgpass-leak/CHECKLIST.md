# #483 checklist

- [x] `write_pgpass` removes the file when `write_all` fails
- [x] sink dropped before unlink (Windows remove-while-open rule)
- [x] `create` wires the helper before `Ok(Self { path })`
- [x] failing-writer test proves file absent after error
- [x] success-path companion test
- [x] `cargo check -p db-pro-infrastructure` clean
- [x] `cargo test -p db-pro-infrastructure --lib` green: 167 passed / 0 failed
