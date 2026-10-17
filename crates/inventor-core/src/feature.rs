use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureState {
    Active,
    Suppressed,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    Union,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureKind {
    OriginPlane {
        name: String,
    },
    OriginAxis {
        name: String,
    },
    OriginPoint,
    Sketch {
        name: String,
        plane_id: Uuid,
    },
    Extrude {
        name: String,
        sketch_id: Uuid,
        distance: f64,
        is_cut: bool,
    },
    Revolve {
        name: String,
        sketch_id: Uuid,
        angle_rad: f64,
        is_cut: bool,
    },
    Boolean {
        name: String,
        op: BooleanOp,
        target_id: Uuid,
        tool_id: Uuid,
    },
    Fillet {
        name: String,
        radius: f64,
    },
    Chamfer {
        name: String,
        distance: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: Uuid,
    pub name: String,
    pub kind: FeatureKind,
    pub state: FeatureState,
    pub visible: bool,
    pub error_message: Option<String>,
}

impl Feature {
    pub fn new(name: impl Into<String>, kind: FeatureKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            state: FeatureState::Active,
            visible: true,
            error_message: None,
        }
    }
}
