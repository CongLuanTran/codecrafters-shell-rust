use std::collections::HashSet;

use rustyline::{
    completion::{Completer, Pair},
    Completer, Context, Helper, Highlighter, Hinter, Validator,
};
use trie_rs::{Trie, TrieBuilder};

#[derive(Helper, Completer, Hinter, Highlighter, Validator)]
pub struct MyHelper {
    #[rustyline(Completer)]
    pub completer: ShellCompleter,
}
pub struct ShellCompleter {
    trie: Trie<u8>,
}

impl ShellCompleter {
    pub fn new(commands: HashSet<String>) -> Self {
        let mut builder = TrieBuilder::new();
        for cmd in commands {
            builder.push(cmd.as_bytes());
        }
        let trie = builder.build();
        ShellCompleter { trie }
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix = &line[..pos];
        let completions = self
            .trie
            .predictive_search(prefix)
            .map(|bytes: Vec<u8>| {
                let s = String::from_utf8(bytes.to_vec()).unwrap_or_default();
                Pair {
                    display: s.clone(),
                    replacement: s + " ",
                }
            })
            .collect();
        Ok((0, completions))
    }
}
