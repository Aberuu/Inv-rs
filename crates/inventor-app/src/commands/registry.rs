//! Central command registry managing application actions and shortcuts.

use super::error::CommandError;
use crate::state::AppState;
use std::collections::HashMap;

/// High-level logical category of a command.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    File,
    Edit,
    Model,
    Sketch,
    View,
    Inspect,
    Tools,
    Help,
}

pub type CommandHandler = fn(&mut AppState) -> Result<(), CommandError>;
pub type EnabledPredicate = fn(&AppState) -> bool;

/// Registration entry for a CAD application command.
#[allow(dead_code)]
#[derive(Clone)]
pub struct CommandEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub shortcut: Option<String>,
    pub category: CommandCategory,
    pub handler: CommandHandler,
    pub is_enabled: Option<EnabledPredicate>,
}

impl CommandEntry {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        category: CommandCategory,
        handler: CommandHandler,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            shortcut: None,
            category,
            handler,
            is_enabled: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_predicate(mut self, predicate: EnabledPredicate) -> Self {
        self.is_enabled = Some(predicate);
        self
    }
}

/// Central registry mapping command IDs to execution handlers and enablement predicates.
pub struct CommandRegistry {
    commands: HashMap<String, CommandEntry>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        super::handlers::build_default_registry()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: CommandEntry) {
        self.commands.insert(entry.id.clone(), entry);
    }

    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<&CommandEntry> {
        self.commands.get(id)
    }

    #[allow(dead_code)]
    pub fn is_registered(&self, id: &str) -> bool {
        self.commands.contains_key(id)
    }

    pub fn is_enabled(&self, id: &str, state: &AppState) -> bool {
        if let Some(entry) = self.commands.get(id) {
            if let Some(check_fn) = entry.is_enabled {
                check_fn(state)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn execute(&self, id: &str, state: &mut AppState) -> Result<(), CommandError> {
        let entry = self
            .commands
            .get(id)
            .ok_or_else(|| CommandError::NotFound(id.to_string()))?;

        if entry.is_enabled.is_some_and(|check_fn| !check_fn(state)) {
            return Err(CommandError::Disabled(
                id.to_string(),
                "Command prerequisites not satisfied in current state.".into(),
            ));
        }

        (entry.handler)(state)
    }

    #[allow(dead_code)]
    pub fn list_commands(&self) -> Vec<&CommandEntry> {
        let mut list: Vec<&CommandEntry> = self.commands.values().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }
}
