use crate::document::Document;

pub trait Command {
    fn execute(&mut self, doc: &mut Document);
    fn undo(&mut self, doc: &mut Document);
    fn description(&self) -> &str;
}

pub struct HistoryStack {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl Default for HistoryStack {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryStack {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, doc: &mut Document) {
        cmd.execute(doc);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, doc: &mut Document) -> bool {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(doc);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, doc: &mut Document) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(doc);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Document command to add a feature to the feature tree with undo/redo support.
#[derive(Debug, Clone)]
pub struct AddFeatureCmd {
    pub feature: crate::feature::Feature,
    pub added_id: Option<uuid::Uuid>,
}

impl AddFeatureCmd {
    pub fn new(feature: crate::feature::Feature) -> Self {
        Self {
            feature,
            added_id: None,
        }
    }
}

impl Command for AddFeatureCmd {
    fn execute(&mut self, doc: &mut Document) {
        self.added_id = Some(self.feature.id);
        doc.add_feature(self.feature.clone());
    }

    fn undo(&mut self, doc: &mut Document) {
        if let Some(id) = self.added_id {
            doc.remove_feature(id);
        }
    }

    fn description(&self) -> &str {
        "Add Feature"
    }
}

/// Document command to set or modify a parameter with undo/redo support.
#[derive(Debug, Clone)]
pub struct SetParameterCmd {
    pub parameter: crate::param::Parameter,
    pub previous_parameter: Option<crate::param::Parameter>,
}

impl SetParameterCmd {
    pub fn new(parameter: crate::param::Parameter) -> Self {
        Self {
            parameter,
            previous_parameter: None,
        }
    }
}

impl Command for SetParameterCmd {
    fn execute(&mut self, doc: &mut Document) {
        self.previous_parameter = doc.parameters.get(&self.parameter.name).cloned();
        doc.parameters.set(self.parameter.clone());
    }

    fn undo(&mut self, doc: &mut Document) {
        if let Some(prev) = &self.previous_parameter {
            doc.parameters.set(prev.clone());
        } else {
            doc.parameters.remove(&self.parameter.name);
        }
    }

    fn description(&self) -> &str {
        "Set Parameter"
    }
}

/// Document command to toggle a feature's suppression state with undo/redo support.
#[derive(Debug, Clone)]
pub struct ToggleSuppressFeatureCmd {
    pub feature_id: uuid::Uuid,
    pub previous_state: Option<crate::feature::FeatureState>,
}

impl ToggleSuppressFeatureCmd {
    pub fn new(feature_id: uuid::Uuid) -> Self {
        Self {
            feature_id,
            previous_state: None,
        }
    }
}

impl Command for ToggleSuppressFeatureCmd {
    fn execute(&mut self, doc: &mut Document) {
        if let Some(f) = doc.find_feature_mut(self.feature_id) {
            self.previous_state = Some(f.state);
            f.state = match f.state {
                crate::feature::FeatureState::Suppressed => crate::feature::FeatureState::Active,
                _ => crate::feature::FeatureState::Suppressed,
            };
            if f.state == crate::feature::FeatureState::Active {
                f.error_message = None;
            }
            doc.is_dirty = true;
        }
    }

    fn undo(&mut self, doc: &mut Document) {
        if let Some(prev) = self.previous_state
            && let Some(f) = doc.find_feature_mut(self.feature_id)
        {
            f.state = prev;
            doc.is_dirty = true;
        }
    }

    fn description(&self) -> &str {
        "Toggle Feature Suppression"
    }
}

/// Document command to reorder a feature up or down with undo/redo support.
#[derive(Debug, Clone)]
pub struct ReorderFeatureCmd {
    pub feature_id: uuid::Uuid,
    pub move_up: bool,
    pub applied: bool,
}

impl ReorderFeatureCmd {
    pub fn new(feature_id: uuid::Uuid, move_up: bool) -> Self {
        Self {
            feature_id,
            move_up,
            applied: false,
        }
    }
}

impl Command for ReorderFeatureCmd {
    fn execute(&mut self, doc: &mut Document) {
        self.applied = if self.move_up {
            doc.move_feature_up(self.feature_id)
        } else {
            doc.move_feature_down(self.feature_id)
        };
    }

    fn undo(&mut self, doc: &mut Document) {
        if self.applied {
            if self.move_up {
                doc.move_feature_down(self.feature_id);
            } else {
                doc.move_feature_up(self.feature_id);
            }
        }
    }

    fn description(&self) -> &str {
        "Reorder Feature"
    }
}

/// Document command to modify a feature definition with undo/redo support.
#[derive(Debug, Clone)]
pub struct ModifyFeatureCmd {
    pub feature_id: uuid::Uuid,
    pub new_kind: crate::feature::FeatureKind,
    pub previous_kind: Option<crate::feature::FeatureKind>,
}

impl ModifyFeatureCmd {
    pub fn new(feature_id: uuid::Uuid, new_kind: crate::feature::FeatureKind) -> Self {
        Self {
            feature_id,
            new_kind,
            previous_kind: None,
        }
    }
}

impl Command for ModifyFeatureCmd {
    fn execute(&mut self, doc: &mut Document) {
        if let Some(f) = doc.find_feature_mut(self.feature_id) {
            self.previous_kind = Some(f.kind.clone());
            f.kind = self.new_kind.clone();
            doc.is_dirty = true;
        }
    }

    fn undo(&mut self, doc: &mut Document) {
        if let Some(prev) = &self.previous_kind
            && let Some(f) = doc.find_feature_mut(self.feature_id)
        {
            f.kind = prev.clone();
            doc.is_dirty = true;
        }
    }

    fn description(&self) -> &str {
        "Modify Feature"
    }
}
