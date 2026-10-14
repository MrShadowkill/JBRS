pub fn project(x: f32, z: f32, rect: [f32; 4]) -> [f32; 2] {
    [
        rect[0] + (x + 1024.) / 2048. * rect[2],
        rect[1] + (z + 1024.) / 2048. * rect[3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_matches_world_extent() {
        assert_eq!(project(-1024., -1024., [10., 20., 200., 100.]), [10., 20.]);
        assert_eq!(project(1024., 1024., [10., 20., 200., 100.]), [210., 120.]);
        assert_eq!(project(0., 0., [10., 20., 200., 100.]), [110., 70.]);
    }
}
