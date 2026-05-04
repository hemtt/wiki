use arma3_wiki_model::{EventHandler, EventHandlerNamespace, ParsedEventHandler, Version};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    passed_commands: Vec<String>,
    failed_commands: IndexMap<String, Vec<String>>,
    outdated_commands: Vec<String>,

    unknown_types_commands: Vec<(String, String)>,

    passed_event_handlers: IndexMap<EventHandlerNamespace, Vec<ParsedEventHandler>>,
    failed_event_handlers: IndexMap<EventHandlerNamespace, Vec<EventHandler>>,
    outdated_event_handlers: IndexMap<EventHandlerNamespace, Vec<ParsedEventHandler>>,

    updated_version: Option<Version>,
}

impl Report {
    #[must_use]
    pub fn new(updated_version: Option<Version>) -> Self {
        Self {
            passed_commands: Vec::new(),
            failed_commands: IndexMap::new(),
            outdated_commands: Vec::new(),

            unknown_types_commands: Vec::new(),

            passed_event_handlers: IndexMap::new(),
            failed_event_handlers: IndexMap::new(),
            outdated_event_handlers: IndexMap::new(),

            updated_version,
        }
    }

    pub fn sort(&mut self) {
        self.passed_commands.sort();
        self.failed_commands.sort_keys();
        self.outdated_commands.sort();

        self.passed_event_handlers.sort_keys();
        for handlers in self.passed_event_handlers.values_mut() {
            handlers.sort();
        }
        self.failed_event_handlers.sort_keys();
        for handlers in self.failed_event_handlers.values_mut() {
            handlers.sort();
        }
        self.outdated_event_handlers.sort_keys();
        for handlers in self.outdated_event_handlers.values_mut() {
            handlers.sort();
        }
    }

    pub fn add_passed_command(&mut self, command: String) {
        self.passed_commands.push(command);
    }

    pub fn add_failed_command(&mut self, command: String, error: String) {
        self.failed_commands.entry(command).or_default().push(error);
    }

    pub fn add_outdated_command(&mut self, command: String) {
        self.outdated_commands.push(command);
    }

    pub fn add_unknown_type_command(&mut self, command: String, error: String) {
        self.unknown_types_commands.push((command, error));
    }

    #[must_use]
    pub const fn updated_version(&self) -> Option<&Version> {
        self.updated_version.as_ref()
    }

    #[must_use]
    pub fn passed_commands(&self) -> &[String] {
        &self.passed_commands
    }

    #[must_use]
    pub const fn failed_commands(&self) -> &IndexMap<String, Vec<String>> {
        &self.failed_commands
    }

    #[must_use]
    pub fn outdated_commands(&self) -> &[String] {
        &self.outdated_commands
    }

    #[must_use]
    pub fn unknown_types_commands(&self) -> &[(String, String)] {
        &self.unknown_types_commands
    }

    #[must_use]
    pub const fn passed_event_handlers(
        &self,
    ) -> &IndexMap<EventHandlerNamespace, Vec<ParsedEventHandler>> {
        &self.passed_event_handlers
    }

    #[must_use]
    pub const fn failed_event_handlers(
        &self,
    ) -> &IndexMap<EventHandlerNamespace, Vec<EventHandler>> {
        &self.failed_event_handlers
    }

    #[must_use]
    pub const fn outdated_event_handlers(
        &self,
    ) -> &IndexMap<EventHandlerNamespace, Vec<ParsedEventHandler>> {
        &self.outdated_event_handlers
    }

    pub fn add_passed_event_handler(
        &mut self,
        ns: EventHandlerNamespace,
        handler: ParsedEventHandler,
    ) {
        self.passed_event_handlers
            .entry(ns)
            .or_default()
            .push(handler);
    }

    pub fn add_failed_event_handler(&mut self, ns: EventHandlerNamespace, handler: EventHandler) {
        self.failed_event_handlers
            .entry(ns)
            .or_default()
            .push(handler);
    }

    pub fn add_outdated_event_handler(
        &mut self,
        ns: EventHandlerNamespace,
        handler: ParsedEventHandler,
    ) {
        self.outdated_event_handlers
            .entry(ns)
            .or_default()
            .push(handler);
    }
}
