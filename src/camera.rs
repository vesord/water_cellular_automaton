use na;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct MVP {
    model: na::Matrix4<f32>,
    view_rotation: na::Matrix4<f32>,
    view_translation: na::Matrix4<f32>,
    projection: na::Matrix4<f32>,
}

impl MVP {
    pub fn new() -> MVP {

        let model = na::Isometry3::identity();

        let mut model: na::Matrix4<f32> = model.to_homogeneous() * 1.;
        model[15] = 1.;

        let view_rotation: na::Matrix4<f32> = na::Isometry3::rotation(na::Vector3::y() * 3.14 / 3.).to_homogeneous();
        let view_rotation = na::Isometry3::rotation(na::Vector3::x() * 3.14 / 3.).to_homogeneous() * view_rotation;
        let view_translation: na::Matrix4<f32> = na::Isometry3::translation(0., -1., -2.).to_homogeneous();

        let projection: na::Matrix4<f32> = na::Orthographic3::new(-1.41, 1.41, -2.5, 1., -30., 30.)
            .to_homogeneous();

        MVP {
            model,
            view_rotation,
            view_translation,
            projection,
        }
    }

    pub fn get_transform(&self) -> na::Matrix4<f32> {
        self.projection * self.view_translation * self.view_rotation * self.model
    }

    pub fn projection_recalc(&mut self, w: i32, h: i32) {
        let aspect: f32 = (w) as f32 / (h) as f32;
        println!("aspect: {}", aspect);
        self.projection = na::Orthographic3::new(-1.41  * aspect, 1.41 * aspect, -2.5, 1., -30., 30.)
            .to_homogeneous();
    }

    pub fn view_rotate_naviball(&mut self, naviball: na::Vector2<f32>) {
        let rot_y = na::Isometry3::rotation(na::Vector3::y() * 3.14 * naviball.x);
        let rot_x = na::Isometry3::rotation(na::Vector3::x() * 3.14 * naviball.y);
        let rot_total: na::Matrix4<f32> = (rot_x * rot_y).to_homogeneous();
        self.view_rotation = rot_total * self.view_rotation;
    }
}
