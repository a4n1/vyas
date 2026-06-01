use glam::{Mat4, Vec3, Vec4};

pub(crate) struct Frustum {
    planes: [Vec4; 5],
}

impl Frustum {
    pub(crate) fn from_view_projection(view_proj: &Mat4) -> Self {
        let cols = view_proj.to_cols_array_2d();
        let row = |index| {
            Vec4::new(
                cols[0][index],
                cols[1][index],
                cols[2][index],
                cols[3][index],
            )
        };

        let x = row(0);
        let y = row(1);
        let z = row(2);
        let w = row(3);

        Self {
            planes: [w + x, w - x, w - y, w + y, w - z],
        }
    }

    pub(crate) fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        self.planes.iter().all(|plane| {
            let normal = plane.truncate();
            let vertex = Vec3::new(
                if normal.x >= 0.0 { max.x } else { min.x },
                if normal.y >= 0.0 { max.y } else { min.y },
                if normal.z >= 0.0 { max.z } else { min.z },
            );

            normal.dot(vertex) + plane.w >= 0.0
        })
    }
}
