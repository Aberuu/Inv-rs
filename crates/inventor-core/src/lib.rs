pub mod document;
pub mod feature;
pub mod history;
pub mod param;

pub use document::*;
pub use feature::*;
pub use history::*;
pub use param::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_initialization() {
        let doc = Document::new("TestPart");
        assert_eq!(doc.title, "TestPart");
        assert_eq!(doc.features.len(), 7); // Origin planes, axes, point
        assert!(!doc.is_dirty);
    }

    #[test]
    fn test_parameter_table() {
        let mut table = ParameterTable::new();
        table.set(Parameter::new("length", 120.0, Unit::Millimeter));
        table.set(Parameter::new("width", 60.0, Unit::Millimeter));

        assert_eq!(table.get_value("length"), Some(120.0));
        assert_eq!(table.get_value("width"), Some(60.0));
        assert_eq!(table.list().len(), 2);
    }

    struct DummyCmd {
        applied: bool,
    }
    impl Command for DummyCmd {
        fn execute(&mut self, doc: &mut Document) {
            doc.is_dirty = true;
            self.applied = true;
        }
        fn undo(&mut self, doc: &mut Document) {
            doc.is_dirty = false;
            self.applied = false;
        }
        fn description(&self) -> &str {
            "dummy"
        }
    }

    #[test]
    fn test_history_undo_redo() {
        let mut doc = Document::new("Part");
        let mut history = HistoryStack::new();
        assert!(!history.can_undo());

        history.execute(Box::new(DummyCmd { applied: false }), &mut doc);
        assert!(doc.is_dirty);
        assert!(history.can_undo());

        assert!(history.undo(&mut doc));
        assert!(!doc.is_dirty);
        assert!(history.can_redo());

        assert!(history.redo(&mut doc));
        assert!(doc.is_dirty);
    }

    #[test]
    fn test_add_feature_cmd_undo_redo() {
        let mut doc = Document::new("Part");
        let mut history = HistoryStack::new();
        let initial_count = doc.features.len();

        let feat = Feature::new(
            "TestExtrude",
            FeatureKind::Extrude {
                name: "TestExtrude".into(),
                sketch_id: doc.xy_plane_id,
                distance: 25.0,
                is_cut: false,
            },
        );
        let feat_id = feat.id;

        history.execute(Box::new(AddFeatureCmd::new(feat)), &mut doc);
        assert_eq!(doc.features.len(), initial_count + 1);
        assert!(doc.find_feature(feat_id).is_some());

        // Undo
        assert!(history.undo(&mut doc));
        assert_eq!(doc.features.len(), initial_count);
        assert!(doc.find_feature(feat_id).is_none());

        // Redo
        assert!(history.redo(&mut doc));
        assert_eq!(doc.features.len(), initial_count + 1);
        assert!(doc.find_feature(feat_id).is_some());
    }

    #[test]
    fn test_set_parameter_cmd_undo_redo() {
        let mut doc = Document::new("Part");
        let mut history = HistoryStack::new();

        // Initial set
        let param = Parameter::new("length", 100.0, Unit::Millimeter);
        history.execute(Box::new(SetParameterCmd::new(param)), &mut doc);
        assert_eq!(doc.parameters.get_value("length"), Some(100.0));

        // Modify parameter
        let param_mod = Parameter::new("length", 200.0, Unit::Millimeter);
        history.execute(Box::new(SetParameterCmd::new(param_mod)), &mut doc);
        assert_eq!(doc.parameters.get_value("length"), Some(200.0));

        // Undo modification
        assert!(history.undo(&mut doc));
        assert_eq!(doc.parameters.get_value("length"), Some(100.0));

        // Undo initial set
        assert!(history.undo(&mut doc));
        assert_eq!(doc.parameters.get_value("length"), None);

        // Redo initial set
        assert!(history.redo(&mut doc));
        assert_eq!(doc.parameters.get_value("length"), Some(100.0));
    }

    #[test]
    fn test_toggle_suppress_cmd_undo_redo() {
        let mut doc = Document::new("Part");
        let mut history = HistoryStack::new();

        let feat = Feature::new("Extrude1", FeatureKind::OriginPoint);
        let feat_id = feat.id;
        doc.add_feature(feat);

        assert_eq!(
            doc.find_feature(feat_id).unwrap().state,
            FeatureState::Active
        );

        // Suppress
        history.execute(Box::new(ToggleSuppressFeatureCmd::new(feat_id)), &mut doc);
        assert_eq!(
            doc.find_feature(feat_id).unwrap().state,
            FeatureState::Suppressed
        );

        // Undo
        assert!(history.undo(&mut doc));
        assert_eq!(
            doc.find_feature(feat_id).unwrap().state,
            FeatureState::Active
        );

        // Redo
        assert!(history.redo(&mut doc));
        assert_eq!(
            doc.find_feature(feat_id).unwrap().state,
            FeatureState::Suppressed
        );
    }

    #[test]
    fn test_reorder_feature_cmd_undo_redo() {
        let mut doc = Document::new("Part");
        let mut history = HistoryStack::new();

        let f1 = Feature::new("Feature1", FeatureKind::OriginPoint);
        let f2 = Feature::new("Feature2", FeatureKind::OriginPoint);
        let f1_id = f1.id;
        let f2_id = f2.id;
        doc.add_feature(f1);
        doc.add_feature(f2);

        let initial_pos1 = doc.features.iter().position(|f| f.id == f1_id).unwrap();
        let initial_pos2 = doc.features.iter().position(|f| f.id == f2_id).unwrap();
        assert_eq!(initial_pos1, 7);
        assert_eq!(initial_pos2, 8);

        // Move f2 up
        history.execute(Box::new(ReorderFeatureCmd::new(f2_id, true)), &mut doc);
        assert_eq!(doc.features[7].id, f2_id);
        assert_eq!(doc.features[8].id, f1_id);

        // Undo
        assert!(history.undo(&mut doc));
        assert_eq!(doc.features[7].id, f1_id);
        assert_eq!(doc.features[8].id, f2_id);

        // Redo
        assert!(history.redo(&mut doc));
        assert_eq!(doc.features[7].id, f2_id);
        assert_eq!(doc.features[8].id, f1_id);
    }

    #[test]
    fn test_modify_feature_cmd_undo_redo() {
        let mut doc = Document::new("Part");
        let mut history = HistoryStack::new();

        let feat = Feature::new(
            "Extrude1",
            FeatureKind::Extrude {
                name: "Extrude1".into(),
                sketch_id: doc.xy_plane_id,
                distance: 20.0,
                is_cut: false,
            },
        );
        let feat_id = feat.id;
        doc.add_feature(feat);

        let new_kind = FeatureKind::Extrude {
            name: "Extrude1".into(),
            sketch_id: doc.xy_plane_id,
            distance: 50.0,
            is_cut: true,
        };

        // Modify
        history.execute(Box::new(ModifyFeatureCmd::new(feat_id, new_kind)), &mut doc);
        match &doc.find_feature(feat_id).unwrap().kind {
            FeatureKind::Extrude {
                distance, is_cut, ..
            } => {
                assert_eq!(*distance, 50.0);
                assert!(*is_cut);
            }
            _ => panic!("Expected Extrude feature"),
        }

        // Undo
        assert!(history.undo(&mut doc));
        match &doc.find_feature(feat_id).unwrap().kind {
            FeatureKind::Extrude {
                distance, is_cut, ..
            } => {
                assert_eq!(*distance, 20.0);
                assert!(!*is_cut);
            }
            _ => panic!("Expected Extrude feature"),
        }
    }
}
