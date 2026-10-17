use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConstraintKind {
    /// Two points coincide
    Coincident { point1: Uuid, point2: Uuid },
    /// Line is horizontal
    Horizontal { line: Uuid },
    /// Line is vertical
    Vertical { line: Uuid },
    /// Two lines are parallel
    Parallel { line1: Uuid, line2: Uuid },
    /// Two lines are perpendicular
    Perpendicular { line1: Uuid, line2: Uuid },
    /// Distance between two points or length of line
    Distance {
        point1: Uuid,
        point2: Uuid,
        distance: f64,
    },
    /// Fix a point's coordinates
    FixPoint { point: Uuid, x: f64, y: f64 },
    /// Circle radius
    Radius { circle: Uuid, radius: f64 },
    /// Circle diameter
    Diameter { circle: Uuid, diameter: f64 },
    /// Angle between two lines in degrees
    Angle {
        line1: Uuid,
        line2: Uuid,
        angle_deg: f64,
    },
    /// Equal length (two lines) or equal radius (two circles)
    Equal { entity1: Uuid, entity2: Uuid },
    /// Tangent between line and circle, or between two circles
    Tangent { entity1: Uuid, entity2: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Uuid,
    pub name: String,
    pub kind: ConstraintKind,
    pub parameter_name: Option<String>,
}

impl Constraint {
    pub fn new(name: impl Into<String>, kind: ConstraintKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            parameter_name: None,
        }
    }

    /// Attaches an associated document parameter name to this constraint.
    pub fn with_parameter(mut self, param_name: impl Into<String>) -> Self {
        self.parameter_name = Some(param_name.into());
        self
    }

    /// Updates the scalar target dimension of this constraint from a parameter value.
    pub fn update_from_parameter_value(&mut self, val: f64) {
        match &mut self.kind {
            ConstraintKind::Distance { distance, .. } => *distance = val,
            ConstraintKind::Radius { radius, .. } => *radius = val,
            ConstraintKind::Diameter { diameter, .. } => *diameter = val,
            ConstraintKind::Angle { angle_deg, .. } => *angle_deg = val,
            _ => {}
        }
    }
}
