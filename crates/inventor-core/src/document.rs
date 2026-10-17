use crate::feature::{Feature, FeatureKind};
use crate::param::ParameterTable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumPlane {
    pub id: Uuid,
    pub name: String,
    pub normal: [f64; 3],
    pub origin: [f64; 3],
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumAxis {
    pub id: Uuid,
    pub name: String,
    pub direction: [f64; 3],
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub xy_plane_id: Uuid,
    pub xz_plane_id: Uuid,
    pub yz_plane_id: Uuid,
    pub x_axis_id: Uuid,
    pub y_axis_id: Uuid,
    pub z_axis_id: Uuid,
    pub center_point_id: Uuid,
    pub features: Vec<Feature>,
    pub parameters: ParameterTable,
    pub is_dirty: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self::new("Part1")
    }
}

impl Document {
    pub fn new(title: impl Into<String>) -> Self {
        let xy_plane = Feature::new(
            "XY Plane",
            FeatureKind::OriginPlane {
                name: "XY Plane".into(),
            },
        );
        let xz_plane = Feature::new(
            "XZ Plane",
            FeatureKind::OriginPlane {
                name: "XZ Plane".into(),
            },
        );
        let yz_plane = Feature::new(
            "YZ Plane",
            FeatureKind::OriginPlane {
                name: "YZ Plane".into(),
            },
        );
        let x_axis = Feature::new(
            "X Axis",
            FeatureKind::OriginAxis {
                name: "X Axis".into(),
            },
        );
        let y_axis = Feature::new(
            "Y Axis",
            FeatureKind::OriginAxis {
                name: "Y Axis".into(),
            },
        );
        let z_axis = Feature::new(
            "Z Axis",
            FeatureKind::OriginAxis {
                name: "Z Axis".into(),
            },
        );
        let center = Feature::new("Center Point", FeatureKind::OriginPoint);

        let xy_id = xy_plane.id;
        let xz_id = xz_plane.id;
        let yz_id = yz_plane.id;
        let x_id = x_axis.id;
        let y_id = y_axis.id;
        let z_id = z_axis.id;
        let c_id = center.id;

        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            xy_plane_id: xy_id,
            xz_plane_id: xz_id,
            yz_plane_id: yz_id,
            x_axis_id: x_id,
            y_axis_id: y_id,
            z_axis_id: z_id,
            center_point_id: c_id,
            features: vec![xy_plane, xz_plane, yz_plane, x_axis, y_axis, z_axis, center],
            parameters: ParameterTable::new(),
            is_dirty: false,
        }
    }

    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
        self.is_dirty = true;
    }

    pub fn find_feature(&self, id: Uuid) -> Option<&Feature> {
        self.features.iter().find(|f| f.id == id)
    }

    pub fn find_feature_mut(&mut self, id: Uuid) -> Option<&mut Feature> {
        self.features.iter_mut().find(|f| f.id == id)
    }

    pub fn toggle_visibility(&mut self, id: Uuid) {
        if let Some(f) = self.find_feature_mut(id) {
            f.visible = !f.visible;
        }
    }

    pub fn remove_feature(&mut self, id: Uuid) -> Option<Feature> {
        if let Some(pos) = self.features.iter().position(|f| f.id == id) {
            self.is_dirty = true;
            Some(self.features.remove(pos))
        } else {
            None
        }
    }

    /// Suppresses a feature by setting its state to Suppressed.
    pub fn suppress_feature(&mut self, id: Uuid) -> bool {
        if let Some(f) = self.find_feature_mut(id) {
            f.state = crate::feature::FeatureState::Suppressed;
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Unsuppresses a feature by setting its state to Active.
    pub fn unsuppress_feature(&mut self, id: Uuid) -> bool {
        if let Some(f) = self.find_feature_mut(id) {
            f.state = crate::feature::FeatureState::Active;
            f.error_message = None;
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Moves a modeling feature one position up in the feature tree.
    /// Preserves the fixed first 7 origin features.
    pub fn move_feature_up(&mut self, id: Uuid) -> bool {
        if let Some(pos) = self.features.iter().position(|f| f.id == id)
            && pos > 7
        {
            self.features.swap(pos, pos - 1);
            self.is_dirty = true;
            return true;
        }
        false
    }

    /// Moves a modeling feature one position down in the feature tree.
    /// Preserves the fixed first 7 origin features.
    pub fn move_feature_down(&mut self, id: Uuid) -> bool {
        if let Some(pos) = self.features.iter().position(|f| f.id == id)
            && pos >= 7
            && pos + 1 < self.features.len()
        {
            self.features.swap(pos, pos + 1);
            self.is_dirty = true;
            return true;
        }
        false
    }
}
