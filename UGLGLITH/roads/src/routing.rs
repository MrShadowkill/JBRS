//! A bounded road graph. Nearby samples join only at the same elevation.
use crate::Road;
use engine_core::Vec3;
use std::collections::{BinaryHeap, HashMap};
pub struct Network {
    pub nodes: Vec<Vec3>,
    edges: Vec<Vec<usize>>,
}
impl Network {
    pub fn new(roads: &[Road]) -> Self {
        let mut nodes = Vec::new();
        let mut edges: Vec<Vec<usize>> = Vec::new();
        for road in roads {
            let start = nodes.len();
            for p in &road.points {
                nodes.push(*p);
                edges.push(Vec::new());
            }
            for i in start + 1..nodes.len() {
                edges[i - 1].push(i);
                edges[i].push(i - 1);
            }
        }
        let mut buckets: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (i, p) in nodes.iter().enumerate() {
            let key = ((p.x / 8.).floor() as i32, (p.z / 8.).floor() as i32);
            for dx in -1..=1 {
                for dz in -1..=1 {
                    if let Some(ids) = buckets.get(&(key.0 + dx, key.1 + dz)) {
                        for &j in ids {
                            if p.distance(nodes[j]) < 6.5 && (p.y - nodes[j].y).abs() < 0.6 {
                                edges[i].push(j);
                                edges[j].push(i);
                            }
                        }
                    }
                }
            }
            buckets.entry(key).or_default().push(i);
        }
        Self { nodes, edges }
    }
    pub fn nearest(&self, p: Vec3) -> usize {
        self.nodes
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.distance(p) + (a.y - p.y).abs() * 4.)
                    .total_cmp(&(b.distance(p) + (b.y - p.y).abs() * 4.))
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    pub fn route(&self, from: Vec3, to: Vec3) -> Option<(f32, Vec<Vec3>)> {
        if self.nodes.is_empty() {
            return None;
        }
        let start = self.nearest(from);
        let end = self.nearest(to);
        let mut cost = vec![u32::MAX; self.nodes.len()];
        let mut parent = vec![usize::MAX; self.nodes.len()];
        let mut queue = BinaryHeap::new();
        cost[start] = 0;
        queue.push((std::cmp::Reverse(0u32), start));
        while let Some((std::cmp::Reverse(d), i)) = queue.pop() {
            if d != cost[i] {
                continue;
            }
            if i == end {
                break;
            }
            for &j in &self.edges[i] {
                let n = d + (self.nodes[i].distance(self.nodes[j]) * 100.).max(1.) as u32;
                if n < cost[j] {
                    cost[j] = n;
                    parent[j] = i;
                    queue.push((std::cmp::Reverse(n), j));
                }
            }
        }
        if cost[end] == u32::MAX {
            return None;
        }
        let mut ids = vec![end];
        let mut i = end;
        while i != start {
            i = parent[i];
            ids.push(i);
        }
        ids.reverse();
        let mut route = Vec::with_capacity(ids.len());
        for (k, &id) in ids.iter().enumerate() {
            let a = self.nodes[id];
            let b = self.nodes[ids[(k + 1).min(ids.len() - 1)]];
            let prev = if k > 0 { self.nodes[ids[k - 1]] } else { a };
            let dx = if a.distance(b) > 0.1 {
                b.x - a.x
            } else {
                a.x - prev.x
            };
            let dz = if a.distance(b) > 0.1 {
                b.z - a.z
            } else {
                a.z - prev.z
            };
            let len = dx.hypot(dz).max(0.001);
            route.push(Vec3::new(a.x + dz / len * 1.6, a.y, a.z - dx / len * 1.6));
        }
        Some((
            cost[end] as f32 / 100. + from.distance(self.nodes[start]),
            route,
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn road(p: &[(f32, f32, f32)]) -> Road {
        Road {
            name: "test",
            points: p.iter().map(|&(x, y, z)| Vec3::new(x, y, z)).collect(),
            width: 8.,
            highway: false,
        }
    }
    #[test]
    fn intersection_connects_but_overpass_does_not() {
        let roads = [
            road(&[(-8., 0., 0.), (0., 0., 0.), (8., 0., 0.)]),
            road(&[(0., 0., -8.), (0., 0., 0.), (0., 0., 8.)]),
            road(&[(-8., 8., 0.), (0., 8., 0.), (8., 8., 0.)]),
        ];
        let n = Network::new(&roads);
        assert!(n
            .route(Vec3::new(-8., 0., 0.), Vec3::new(0., 0., 8.))
            .is_some());
        // Graph construction never joins a bridge to the street underneath.
        assert!(n.edges[6].iter().all(|&i| i >= 6));
    }
}
