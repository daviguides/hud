README examples (richrs 0.2.1), verbatim
panel, table, tree: compile and render
progress: compiles; stdout bytes:        0, stderr bytes:        0 (advance() only mutates state; nothing is drawn)
live: does not compile:
  error[E0277]: Text: From<&String> not satisfied (live.update(&format!(..)))
  error[E0277]: the ? operator cannot be applied to type () (Live::update returns unit)
