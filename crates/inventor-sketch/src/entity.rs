use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub id: Uuid,
    pub x: f64,
    pub y: f64,
    pub is_fixed: bool,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            x,
            y,
            is_fixed: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line2D {
    pub id: Uuid,
    pub start: Uuid,
    pub end: Uuid,
    pub is_construction: bool,
}

impl Line2D {
    pub fn new(start: Uuid, end: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            start,
            end,
            is_construction: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle2D {
    pub id: Uuid,
    pub center: Uuid,
    pub radius: f64,
    pub is_construction: bool,
}

impl Circle2D {
    pub fn new(center: Uuid, radius: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            center,
            radius,
            is_construction: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc2D {
    pub id: Uuid,
    pub center: Uuid,
    pub start: Uuid,
    pub end: Uuid,
    pub radius: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sketch2D {
    pub id: Uuid,
    pub name: String,
    pub plane_id: Uuid,
    pub points: Vec<Point2D>,
    pub lines: Vec<Line2D>,
    pub circles: Vec<Circle2D>,
    pub arcs: Vec<Arc2D>,
}

impl Sketch2D {
    pub fn new(name: impl Into<String>, plane_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            plane_id,
            points: Vec::new(),
            lines: Vec::new(),
            circles: Vec::new(),
            arcs: Vec::new(),
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64) -> Uuid {
        let pt = Point2D::new(x, y);
        let id = pt.id;
        self.points.push(pt);
        id
    }

    pub fn add_line(&mut self, start: Uuid, end: Uuid) -> Uuid {
        let line = Line2D::new(start, end);
        let id = line.id;
        self.lines.push(line);
        id
    }

    pub fn add_circle(&mut self, center: Uuid, radius: f64) -> Uuid {
        let circle = Circle2D::new(center, radius);
        let id = circle.id;
        self.circles.push(circle);
        id
    }

    pub fn find_point(&self, id: Uuid) -> Option<&Point2D> {
        self.points.iter().find(|p| p.id == id)
    }

    pub fn find_point_mut(&mut self, id: Uuid) -> Option<&mut Point2D> {
        self.points.iter_mut().find(|p| p.id == id)
    }

    pub fn find_line(&self, id: Uuid) -> Option<&Line2D> {
        self.lines.iter().find(|l| l.id == id)
    }

    pub fn find_line_mut(&mut self, id: Uuid) -> Option<&mut Line2D> {
        self.lines.iter_mut().find(|l| l.id == id)
    }

    pub fn find_circle(&self, id: Uuid) -> Option<&Circle2D> {
        self.circles.iter().find(|c| c.id == id)
    }

    pub fn find_circle_mut(&mut self, id: Uuid) -> Option<&mut Circle2D> {
        self.circles.iter_mut().find(|c| c.id == id)
    }

    pub fn find_arc(&self, id: Uuid) -> Option<&Arc2D> {
        self.arcs.iter().find(|a| a.id == id)
    }

    pub fn find_arc_mut(&mut self, id: Uuid) -> Option<&mut Arc2D> {
        self.arcs.iter_mut().find(|a| a.id == id)
    }

    /// Extracts closed boundary profiles (ordered polygons in CCW orientation) from the sketch.
    ///
    /// Traces connected non-construction line loops. If no line loops exist but circles are
    /// present, tessellates each circle into a regular polygon profile.
    pub fn extract_closed_profiles(&self) -> Vec<Vec<[f64; 2]>> {
        use std::collections::{HashMap, HashSet};

        let mut profiles = Vec::new();
        let valid_lines: Vec<&Line2D> = self.lines.iter().filter(|l| !l.is_construction).collect();

        if !valid_lines.is_empty() {
            // Build undirected adjacency graph: point_id -> Vec<(neighbor_id, line_id)>
            let mut adj: HashMap<Uuid, Vec<(Uuid, Uuid)>> = HashMap::new();
            for line in &valid_lines {
                adj.entry(line.start).or_default().push((line.end, line.id));
                adj.entry(line.end).or_default().push((line.start, line.id));
            }

            let mut visited_lines: HashSet<Uuid> = HashSet::new();

            for line in &valid_lines {
                if visited_lines.contains(&line.id) {
                    continue;
                }

                let start_pt = line.start;
                let mut current_pt = line.end;
                let mut loop_pts = vec![start_pt, current_pt];
                visited_lines.insert(line.id);

                let mut closed = false;
                while !closed {
                    // Find an unvisited edge from current_pt
                    let next_edge = adj.get(&current_pt).and_then(|neighbors| {
                        neighbors
                            .iter()
                            .find(|(_, lid)| !visited_lines.contains(lid))
                    });

                    if let Some(&(next_pt, next_line_id)) = next_edge {
                        visited_lines.insert(next_line_id);
                        if next_pt == start_pt {
                            closed = true;
                        } else {
                            loop_pts.push(next_pt);
                            current_pt = next_pt;
                        }
                    } else {
                        // Open path, cannot close loop
                        break;
                    }
                }

                if closed && loop_pts.len() >= 3 {
                    let mut polygon: Vec<[f64; 2]> = Vec::with_capacity(loop_pts.len());
                    for pid in &loop_pts {
                        if let Some(pt) = self.find_point(*pid) {
                            polygon.push([pt.x, pt.y]);
                        }
                    }

                    if polygon.len() >= 3 {
                        // Calculate signed area to ensure Counter-Clockwise (CCW) winding
                        let mut area2 = 0.0;
                        for i in 0..polygon.len() {
                            let next = (i + 1) % polygon.len();
                            area2 +=
                                polygon[i][0] * polygon[next][1] - polygon[next][0] * polygon[i][1];
                        }
                        if area2 < 0.0 {
                            polygon.reverse();
                        }
                        profiles.push(polygon);
                    }
                }
            }
        }

        // Fallback: If no closed line loops were found, check if there are circles
        if profiles.is_empty() {
            for circle in &self.circles {
                if circle.is_construction {
                    continue;
                }
                if let Some(center) = self.find_point(circle.center) {
                    let segments = 48;
                    let mut polygon = Vec::with_capacity(segments);
                    let tau = std::f64::consts::TAU;
                    for i in 0..segments {
                        let theta = (i as f64) * tau / (segments as f64);
                        let x = center.x + circle.radius * theta.cos();
                        let y = center.y + circle.radius * theta.sin();
                        polygon.push([x, y]);
                    }
                    profiles.push(polygon);
                }
            }
        }

        profiles
    }
}
