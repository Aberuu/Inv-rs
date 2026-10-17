//! Data-driven ribbon schema for the CAD user interface.
//!
//! Provides the data structures and default definitions for the Ribbon UI:
//! Tab -> Panel -> Item (Large/Small Button, Dropdown, Separator).

/// Visual size of a ribbon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonButtonSize {
    Large,
    Small,
}

/// An individual option within a ribbon dropdown menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RibbonDropdownOption {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub tooltip: String,
    pub command_id: Option<String>,
    pub enabled: bool,
}

impl RibbonDropdownOption {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: Option<&str>,
        command_id: Option<&str>,
        tooltip: impl Into<String>,
    ) -> Self {
        let cmd = command_id.map(|s| s.to_string());
        let has_cmd = cmd.is_some();
        let tooltip_str = tooltip.into();
        let effective_tooltip = if !has_cmd && tooltip_str.is_empty() {
            "Belum tersedia".to_string()
        } else {
            tooltip_str
        };

        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.map(|s| s.to_string()),
            tooltip: effective_tooltip,
            command_id: cmd,
            enabled: has_cmd,
        }
    }
}

/// An item inside a ribbon panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RibbonItem {
    Button {
        id: String,
        label: String,
        icon: Option<String>,
        tooltip: String,
        size: RibbonButtonSize,
        command_id: Option<String>,
        enabled: bool,
    },
    Dropdown {
        id: String,
        label: String,
        icon: Option<String>,
        tooltip: String,
        size: RibbonButtonSize,
        options: Vec<RibbonDropdownOption>,
        enabled: bool,
    },
    Separator,
}

impl RibbonItem {
    /// Creates a large ribbon button.
    pub fn large_button(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: Option<&str>,
        command_id: Option<&str>,
        tooltip: impl Into<String>,
    ) -> Self {
        let cmd = command_id.map(|s| s.to_string());
        let has_cmd = cmd.is_some();
        let tooltip_str = tooltip.into();
        let effective_tooltip = if !has_cmd && tooltip_str.is_empty() {
            "Belum tersedia".to_string()
        } else {
            tooltip_str
        };

        Self::Button {
            id: id.into(),
            label: label.into(),
            icon: icon.map(|s| s.to_string()),
            tooltip: effective_tooltip,
            size: RibbonButtonSize::Large,
            command_id: cmd,
            enabled: has_cmd,
        }
    }

    /// Creates a small ribbon button.
    pub fn small_button(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: Option<&str>,
        command_id: Option<&str>,
        tooltip: impl Into<String>,
    ) -> Self {
        let cmd = command_id.map(|s| s.to_string());
        let has_cmd = cmd.is_some();
        let tooltip_str = tooltip.into();
        let effective_tooltip = if !has_cmd && tooltip_str.is_empty() {
            "Belum tersedia".to_string()
        } else {
            tooltip_str
        };

        Self::Button {
            id: id.into(),
            label: label.into(),
            icon: icon.map(|s| s.to_string()),
            tooltip: effective_tooltip,
            size: RibbonButtonSize::Small,
            command_id: cmd,
            enabled: has_cmd,
        }
    }

    /// Creates a ribbon dropdown menu.
    pub fn dropdown(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: Option<&str>,
        tooltip: impl Into<String>,
        size: RibbonButtonSize,
        options: Vec<RibbonDropdownOption>,
    ) -> Self {
        let has_any_enabled = options.iter().any(|o| o.enabled);
        let tooltip_str = tooltip.into();
        let effective_tooltip = if !has_any_enabled && tooltip_str.is_empty() {
            "Belum tersedia".to_string()
        } else {
            tooltip_str
        };

        Self::Dropdown {
            id: id.into(),
            label: label.into(),
            icon: icon.map(|s| s.to_string()),
            tooltip: effective_tooltip,
            size,
            options,
            enabled: has_any_enabled,
        }
    }

    /// Creates a visual separator line between item clusters.
    pub fn separator() -> Self {
        Self::Separator
    }

    #[allow(dead_code)]
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Button { id, .. } | Self::Dropdown { id, .. } => Some(id.as_str()),
            Self::Separator => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Button { enabled, .. } | Self::Dropdown { enabled, .. } => *enabled,
            Self::Separator => true,
        }
    }

    #[allow(dead_code)]
    pub fn tooltip(&self) -> &str {
        match self {
            Self::Button { tooltip, .. } | Self::Dropdown { tooltip, .. } => tooltip.as_str(),
            Self::Separator => "",
        }
    }
}

/// A logical panel grouping related ribbon items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RibbonPanel {
    pub id: String,
    pub title: String,
    pub items: Vec<RibbonItem>,
}

impl RibbonPanel {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            items: Vec::new(),
        }
    }

    pub fn with_items(mut self, items: Vec<RibbonItem>) -> Self {
        self.items = items;
        self
    }

    #[allow(dead_code)]
    pub fn add_item(&mut self, item: RibbonItem) -> &mut Self {
        self.items.push(item);
        self
    }
}

/// A top-level ribbon tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RibbonTab {
    pub id: String,
    pub label: String,
    pub panels: Vec<RibbonPanel>,
    pub enabled: bool,
    pub tooltip: Option<String>,
}

impl RibbonTab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            panels: Vec::new(),
            enabled: true,
            tooltip: None,
        }
    }

    pub fn disabled(
        id: impl Into<String>,
        label: impl Into<String>,
        tooltip: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            panels: Vec::new(),
            enabled: false,
            tooltip: Some(tooltip.into()),
        }
    }

    pub fn with_panels(mut self, panels: Vec<RibbonPanel>) -> Self {
        self.panels = panels;
        self
    }

    #[allow(dead_code)]
    pub fn add_panel(&mut self, panel: RibbonPanel) -> &mut Self {
        self.panels.push(panel);
        self
    }
}

/// Full ribbon schema hierarchy: Tabs -> Panels -> Items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RibbonSchema {
    pub tabs: Vec<RibbonTab>,
}

impl RibbonSchema {
    /// Builds the standard Part modeling environment ribbon schema according to PRD Section 14.
    pub fn default_part_schema() -> Self {
        // Tab 1: 3D Model
        let model_tab = RibbonTab::new("model_3d", "3D Model").with_panels(vec![
            RibbonPanel::new("sketch", "Sketch").with_items(vec![RibbonItem::large_button(
                "create_sketch",
                "2D Sketch",
                Some("✏"),
                Some("sketch.new"),
                "Create a 2D sketch on a plane or planar face",
            )]),
            RibbonPanel::new("create", "Create").with_items(vec![
                RibbonItem::large_button(
                    "box",
                    "Box",
                    Some("📦"),
                    Some("model.box"),
                    "Create a 3D box solid",
                ),
                RibbonItem::large_button(
                    "cylinder",
                    "Cylinder",
                    Some("🔘"),
                    Some("model.cylinder"),
                    "Create a 3D cylinder solid",
                ),
                RibbonItem::separator(),
                RibbonItem::large_button(
                    "extrude",
                    "Extrude",
                    Some("⬆"),
                    Some("model.extrude"),
                    "Extrude a closed sketch profile",
                ),
                RibbonItem::large_button(
                    "revolve",
                    "Revolve",
                    Some("🔄"),
                    Some("model.revolve"),
                    "Revolve a closed sketch profile around an axis",
                ),
            ]),
            RibbonPanel::new("modify", "Modify").with_items(vec![
                RibbonItem::small_button(
                    "combine",
                    "Combine",
                    Some("⧉"),
                    Some("model.combine"),
                    "Perform boolean Join, Cut, or Intersect between solid features",
                ),
                RibbonItem::small_button(
                    "fillet",
                    "Fillet",
                    Some("⌒"),
                    Some("model.fillet"),
                    "Create rounded blend on solid edges (evaluated in T8)",
                ),
                RibbonItem::small_button(
                    "chamfer",
                    "Chamfer",
                    Some("◺"),
                    Some("model.chamfer"),
                    "Create beveled chamfer on solid edges (evaluated in T8)",
                ),
                RibbonItem::small_button("hole", "Hole", Some("🕳"), None, "Belum tersedia"),
                RibbonItem::small_button("shell", "Shell", Some("⛶"), None, "Belum tersedia"),
            ]),
            RibbonPanel::new("pattern", "Pattern").with_items(vec![
                RibbonItem::small_button(
                    "rect_pattern",
                    "Rectangular",
                    Some("⊞"),
                    None,
                    "Belum tersedia",
                ),
                RibbonItem::small_button(
                    "circ_pattern",
                    "Circular",
                    Some("⨀"),
                    None,
                    "Belum tersedia",
                ),
                RibbonItem::small_button("mirror", "Mirror", Some("⇋"), None, "Belum tersedia"),
            ]),
            RibbonPanel::new("work_features", "Work Features").with_items(vec![
                RibbonItem::small_button("work_plane", "Plane", Some("▱"), None, "Belum tersedia"),
                RibbonItem::small_button("work_axis", "Axis", Some("⌁"), None, "Belum tersedia"),
                RibbonItem::small_button("work_point", "Point", Some("•"), None, "Belum tersedia"),
            ]),
            RibbonPanel::new("history", "History").with_items(vec![
                RibbonItem::small_button(
                    "undo",
                    "Undo",
                    Some("↶"),
                    Some("edit.undo"),
                    "Undo the last document change (Ctrl+Z)",
                ),
                RibbonItem::small_button(
                    "redo",
                    "Redo",
                    Some("↷"),
                    Some("edit.redo"),
                    "Redo the last undone change (Ctrl+Y)",
                ),
            ]),
        ]);

        // Tab 2: Sketch
        let sketch_tab = RibbonTab::new("sketch", "Sketch").with_panels(vec![
            RibbonPanel::new("create", "Create").with_items(vec![
                RibbonItem::large_button(
                    "line",
                    "Line",
                    Some("📏"),
                    Some("sketch.line"),
                    "Draw straight line segments",
                ),
                RibbonItem::large_button(
                    "circle",
                    "Circle",
                    Some("⭕"),
                    Some("sketch.circle"),
                    "Draw circle from center and radius",
                ),
                RibbonItem::large_button("arc", "Arc", Some("⌒"), None, "Belum tersedia"),
                RibbonItem::large_button(
                    "rectangle",
                    "Rectangle",
                    Some("▭"),
                    Some("sketch.rectangle"),
                    "Draw rectangle from corner points",
                ),
                RibbonItem::small_button(
                    "point",
                    "Point",
                    Some("•"),
                    Some("sketch.point"),
                    "Place standalone sketch points or reference vertices",
                ),
            ]),
            RibbonPanel::new("constrain", "Constrain").with_items(vec![
                RibbonItem::small_button(
                    "coincident",
                    "Coincident",
                    Some("⨀"),
                    Some("sketch.coincident"),
                    "Constrain two points to share exact coordinates",
                ),
                RibbonItem::small_button(
                    "horizontal",
                    "Horizontal",
                    Some("━"),
                    Some("sketch.horizontal"),
                    "Constrain line horizontal",
                ),
                RibbonItem::small_button(
                    "vertical",
                    "Vertical",
                    Some("↕"),
                    Some("sketch.vertical"),
                    "Constrain line vertical",
                ),
                RibbonItem::small_button(
                    "parallel",
                    "Parallel",
                    Some("∥"),
                    Some("sketch.parallel"),
                    "Constrain two lines parallel",
                ),
                RibbonItem::small_button(
                    "perpendicular",
                    "Perpendicular",
                    Some("⊥"),
                    Some("sketch.perpendicular"),
                    "Constrain two lines perpendicular",
                ),
                RibbonItem::small_button(
                    "tangent",
                    "Tangent",
                    Some("⌒"),
                    Some("sketch.tangent"),
                    "Constrain line tangent to curve or circles tangent",
                ),
                RibbonItem::small_button(
                    "equal",
                    "Equal",
                    Some("="),
                    Some("sketch.equal"),
                    "Constrain lines equal length or circles equal radius",
                ),
                RibbonItem::small_button(
                    "fix",
                    "Fix",
                    Some("🔒"),
                    Some("sketch.fix"),
                    "Lock point at current coordinates",
                ),
            ]),
            RibbonPanel::new("dimension", "Dimension").with_items(vec![RibbonItem::large_button(
                "dimension",
                "Dimension",
                Some("📐"),
                Some("sketch.dimension"),
                "Apply distance, length, radius, or angle dimension (D)",
            )]),
            RibbonPanel::new("exit", "Exit").with_items(vec![RibbonItem::large_button(
                "finish_sketch",
                "Finish Sketch",
                Some("✔"),
                Some("sketch.finish"),
                "Finish 2D sketch editing and return to 3D",
            )]),
        ]);

        // Tab 3: Inspect
        let inspect_tab = RibbonTab::new("inspect", "Inspect").with_panels(vec![
            RibbonPanel::new("measure", "Measure").with_items(vec![
                RibbonItem::large_button(
                    "measure_box",
                    "Bounding Box",
                    Some("📐"),
                    Some("inspect.measure"),
                    "Measure 3D solid bounding box dimensions",
                ),
                RibbonItem::small_button(
                    "measure_dist",
                    "Distance",
                    Some("📏"),
                    None,
                    "Belum tersedia",
                ),
                RibbonItem::small_button(
                    "measure_angle",
                    "Angle",
                    Some("∠"),
                    None,
                    "Belum tersedia",
                ),
            ]),
        ]);

        // Tab 4: Tools
        let tools_tab = RibbonTab::new("tools", "Tools").with_panels(vec![
            RibbonPanel::new("parameters", "Parameters").with_items(vec![
                RibbonItem::large_button(
                    "parameters",
                    "Parameters",
                    Some("fx"),
                    Some("tools.parameters"),
                    "Edit global parametric dimension table",
                ),
            ]),
            RibbonPanel::new("export", "Export").with_items(vec![
                RibbonItem::large_button(
                    "export_step",
                    "Export STEP",
                    Some("📤"),
                    Some("file.export_step"),
                    "Export active 3D solid as ISO 10303 STEP file",
                ),
                RibbonItem::large_button(
                    "export_stl",
                    "Export STL",
                    Some("🖨"),
                    Some("file.export_stl"),
                    "Export active 3D solid as STL mesh",
                ),
            ]),
            RibbonPanel::new("options", "Options").with_items(vec![RibbonItem::small_button(
                "options",
                "Options",
                Some("⚙"),
                Some("tools.options"),
                "Configure application and kernel backend options",
            )]),
        ]);

        // Tab 5: View
        let view_tab = RibbonTab::new("view", "View").with_panels(vec![
            RibbonPanel::new("navigate", "Navigate").with_items(vec![
                RibbonItem::large_button(
                    "view_home",
                    "Home (F6)",
                    Some("🏠"),
                    Some("view.home"),
                    "Reset camera to standard Isometric Home view",
                ),
                RibbonItem::small_button(
                    "view_front",
                    "Front",
                    Some("⯈"),
                    Some("view.front"),
                    "Orient camera to Front view",
                ),
                RibbonItem::small_button(
                    "view_top",
                    "Top",
                    Some("⯅"),
                    Some("view.top"),
                    "Orient camera to Top view",
                ),
                RibbonItem::small_button(
                    "view_right",
                    "Right",
                    Some("⯈"),
                    Some("view.right"),
                    "Orient camera to Right view",
                ),
            ]),
            RibbonPanel::new("appearance", "Appearance").with_items(vec![
                RibbonItem::dropdown(
                    "projection",
                    "Projection",
                    Some("📽"),
                    "Camera projection style",
                    RibbonButtonSize::Small,
                    vec![
                        RibbonDropdownOption::new(
                            "proj_perspective",
                            "Perspective",
                            None,
                            Some("view.proj_perspective"),
                            "Realistic 3D perspective projection",
                        ),
                        RibbonDropdownOption::new(
                            "proj_ortho",
                            "Orthographic",
                            None,
                            Some("view.proj_ortho"),
                            "Parallel 3D orthographic projection",
                        ),
                    ],
                ),
                RibbonItem::separator(),
                RibbonItem::small_button(
                    "toggle_grid",
                    "Grid",
                    Some("▦"),
                    Some("view.toggle_grid"),
                    "Toggle ground plane grid",
                ),
                RibbonItem::small_button(
                    "toggle_axes",
                    "Axes",
                    Some("┼"),
                    Some("view.toggle_axes"),
                    "Toggle coordinate axes indicator",
                ),
            ]),
        ]);

        // Tab 6: Assemble (Contextual / disabled until Milestone 6)
        let assemble_tab =
            RibbonTab::disabled(
                "assemble",
                "Assemble",
                "Belum tersedia (Milestone 6 Assembly)",
            )
            .with_panels(vec![
                RibbonPanel::new("component", "Component").with_items(vec![
                    RibbonItem::large_button("place", "Place", Some("📥"), None, "Belum tersedia"),
                ]),
                RibbonPanel::new("relationships", "Relationships").with_items(vec![
                    RibbonItem::small_button("joint", "Joint", Some("🔗"), None, "Belum tersedia"),
                    RibbonItem::small_button(
                        "constrain",
                        "Constrain",
                        Some("⛓"),
                        None,
                        "Belum tersedia",
                    ),
                ]),
            ]);

        // Tab 7: Annotate (Contextual / disabled until Milestone 7)
        let annotate_tab = RibbonTab::disabled(
            "annotate",
            "Annotate",
            "Belum tersedia (Milestone 7 Drawing)",
        )
        .with_panels(vec![
            RibbonPanel::new("dimension", "Dimension").with_items(vec![RibbonItem::large_button(
                "annot_dim",
                "Dimension",
                Some("📏"),
                None,
                "Belum tersedia",
            )]),
            RibbonPanel::new("notes", "Notes").with_items(vec![RibbonItem::small_button(
                "annot_text",
                "Text",
                Some("🗛"),
                None,
                "Belum tersedia",
            )]),
        ]);

        Self {
            tabs: vec![
                model_tab,
                sketch_tab,
                inspect_tab,
                tools_tab,
                view_tab,
                assemble_tab,
                annotate_tab,
            ],
        }
    }

    /// Searches all tabs, panels, and items matching the query keyword.
    pub fn search_items<'a>(
        &'a self,
        query: &str,
    ) -> Vec<(&'a RibbonTab, &'a RibbonPanel, &'a RibbonItem)> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for tab in &self.tabs {
            for panel in &tab.panels {
                for item in &panel.items {
                    match item {
                        RibbonItem::Button { label, id, .. } => {
                            if label.to_lowercase().contains(&q) || id.to_lowercase().contains(&q) {
                                results.push((tab, panel, item));
                            }
                        }
                        RibbonItem::Dropdown {
                            label, id, options, ..
                        } => {
                            if label.to_lowercase().contains(&q)
                                || id.to_lowercase().contains(&q)
                                || options.iter().any(|o| o.label.to_lowercase().contains(&q))
                            {
                                results.push((tab, panel, item));
                            }
                        }
                        RibbonItem::Separator => {}
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_part_schema_structure() {
        let schema = RibbonSchema::default_part_schema();
        let tab_ids: Vec<&str> = schema.tabs.iter().map(|t| t.id.as_str()).collect();

        // Must include all tabs defined in PRD Section 14
        assert_eq!(
            tab_ids,
            vec![
                "model_3d", "sketch", "inspect", "tools", "view", "assemble", "annotate"
            ]
        );

        // Assemble and Annotate are nonactive before M6 / M7 per PRD Section 14
        let assemble_tab = schema.tabs.iter().find(|t| t.id == "assemble").unwrap();
        assert!(!assemble_tab.enabled);
        assert!(
            assemble_tab
                .tooltip
                .as_ref()
                .unwrap()
                .contains("Belum tersedia")
        );

        let annotate_tab = schema.tabs.iter().find(|t| t.id == "annotate").unwrap();
        assert!(!annotate_tab.enabled);
        assert!(
            annotate_tab
                .tooltip
                .as_ref()
                .unwrap()
                .contains("Belum tersedia")
        );

        // 3D Model and Sketch tabs must be enabled
        let model_tab = schema.tabs.iter().find(|t| t.id == "model_3d").unwrap();
        assert!(model_tab.enabled);
        assert!(!model_tab.panels.is_empty());
    }

    #[test]
    fn test_disabled_button_without_command_has_belum_tersedia_tooltip() {
        let item = RibbonItem::large_button(
            "revolve",
            "Revolve",
            Some("🔄"),
            None,
            "", // Empty custom tooltip
        );

        assert_eq!(item.id(), Some("revolve"));
        assert!(!item.is_enabled());
        assert_eq!(item.tooltip(), "Belum tersedia");
    }

    #[test]
    fn test_enabled_button_with_command() {
        let item = RibbonItem::large_button(
            "box",
            "Box",
            Some("📦"),
            Some("model.box"),
            "Create a 3D box solid",
        );

        assert_eq!(item.id(), Some("box"));
        assert!(item.is_enabled());
        assert_eq!(item.tooltip(), "Create a 3D box solid");
    }

    #[test]
    fn test_large_and_small_button_sizes() {
        let large = RibbonItem::large_button(
            "extrude",
            "Extrude",
            Some("⬆"),
            Some("model.extrude"),
            "Extrude",
        );
        let small = RibbonItem::small_button("fillet", "Fillet", Some("⌒"), None, "Belum tersedia");

        match large {
            RibbonItem::Button { size, .. } => assert_eq!(size, RibbonButtonSize::Large),
            _ => panic!("Expected Button"),
        }

        match small {
            RibbonItem::Button { size, .. } => assert_eq!(size, RibbonButtonSize::Small),
            _ => panic!("Expected Button"),
        }
    }

    #[test]
    fn test_dropdown_item() {
        let dropdown = RibbonItem::dropdown(
            "proj_mode",
            "Projection",
            Some("📽"),
            "Projection style",
            RibbonButtonSize::Small,
            vec![
                RibbonDropdownOption::new(
                    "p1",
                    "Perspective",
                    None,
                    Some("view.perspective"),
                    "Perspective",
                ),
                RibbonDropdownOption::new("p2", "Orthographic", None, None, ""),
            ],
        );

        assert_eq!(dropdown.id(), Some("proj_mode"));
        assert!(dropdown.is_enabled()); // Has at least one enabled option

        if let RibbonItem::Dropdown { options, .. } = dropdown {
            assert_eq!(options.len(), 2);
            assert!(options[0].enabled);
            assert!(!options[1].enabled);
            assert_eq!(options[1].tooltip, "Belum tersedia");
        } else {
            panic!("Expected Dropdown");
        }
    }

    #[test]
    fn test_add_button_single_entry() {
        // Acceptance criterion: adding a button requires adding only a single data entry
        let mut panel = RibbonPanel::new("custom_panel", "Custom");
        assert_eq!(panel.items.len(), 0);

        // Add a single entry:
        panel.add_item(RibbonItem::large_button(
            "my_new_tool",
            "My New Tool",
            Some("⚡"),
            Some("custom.tool"),
            "Execute custom action",
        ));

        assert_eq!(panel.items.len(), 1);
        assert_eq!(panel.items[0].id(), Some("my_new_tool"));
        assert!(panel.items[0].is_enabled());
    }

    #[test]
    fn test_search_items() {
        let schema = RibbonSchema::default_part_schema();

        // Search for "box"
        let box_results = schema.search_items("box");
        assert!(!box_results.is_empty());
        assert!(
            box_results
                .iter()
                .any(|(_, _, item)| item.id() == Some("box"))
        );

        // Search for "extrude"
        let extrude_results = schema.search_items("extrude");
        assert!(!extrude_results.is_empty());
        assert!(
            extrude_results
                .iter()
                .any(|(_, _, item)| item.id() == Some("extrude"))
        );

        // Search for "projection" (dropdown option or id)
        let proj_results = schema.search_items("projection");
        assert!(!proj_results.is_empty());

        // Empty query returns empty results
        assert!(schema.search_items("").is_empty());
    }

    #[test]
    fn test_panel_and_separator_construction() {
        let mut tab = RibbonTab::new("custom_tab", "Custom Tab");
        let mut panel = RibbonPanel::new("p1", "Panel 1");
        panel.add_item(RibbonItem::separator());
        assert_eq!(panel.items.len(), 1);
        assert_eq!(panel.items[0], RibbonItem::Separator);
        assert_eq!(panel.items[0].id(), None);
        assert!(panel.items[0].is_enabled());
        assert_eq!(panel.items[0].tooltip(), "");

        tab.add_panel(panel);
        assert_eq!(tab.panels.len(), 1);
    }
}
