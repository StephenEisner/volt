mod displayable_command;

use fragment;
use helpers::SelectableSet;
use std::collections::HashMap;
use std::slice::Iter;
use models::application::modes::SearchSelectMode;
use commands::{self, Command};
pub use self::displayable_command::DisplayableCommand;

pub struct CommandMode {
    insert: bool,
    input: String,
    commands: HashMap<&'static str, Command>,
    results: SelectableSet<DisplayableCommand>,
}

impl CommandMode {
    pub const MAX_RESULTS: usize = 5;

    pub fn new() -> CommandMode {
        CommandMode {
            insert: true,
            input: String::new(),
            commands: commands::hash_map(),
            results: SelectableSet::new(Vec::new()),
        }
    }
}

impl SearchSelectMode<DisplayableCommand> for CommandMode {
    fn search(&mut self) {
        // Find the commands we're looking for using the query.
        let results = fragment::matching::find(
            &self.input,
            &self.commands.keys().collect(),
            CommandMode::MAX_RESULTS,
            false
        );

        // We don't care about the result objects; we just want
        // the underlying commands. Map the collection to get these.
        self.results = SelectableSet::new(
            results
            .into_iter()
            .filter_map(|result| {
                self.commands.get(*result).map(|command| {
                    DisplayableCommand{
                      description: *result,
                      command: *command
                    }
                })
            })
            .collect()
        );
    }

    fn query(&mut self) -> &mut String {
        &mut self.input
    }

    fn insert_mode(&self) -> bool {
        self.insert
    }

    fn set_insert_mode(&mut self, insert_mode: bool) {
        self.insert = insert_mode;
    }

    fn results(&self) -> Iter<DisplayableCommand> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&DisplayableCommand> {
        self.results.selection()
    }

    fn selected_index(&self) -> usize {
        self.results.selected_index()
    }

    fn select_previous(&mut self) {
        self.results.select_previous();
    }

    fn select_next(&mut self) {
        self.results.select_next();
    }
}