//! 2D and 3D camera.

use crate::{
    get_context,
    math::Rect,
    texture::RenderTarget,
    window::{screen_height, screen_width},
};
use glam::{vec2, vec3, Mat4, Vec2};

pub trait Camera {
    fn matrix(&self) -> Mat4;
    fn depth_enabled(&self) -> bool;
    fn render_pass(&self) -> Option<miniquad::RenderPass>;
}

#[derive(Clone, Copy)]
pub struct Camera2D {
    /// Rotation in degrees
    pub rotation: f32,
    /// Scaling, should be (1.0, 1.0) by default
    pub zoom: Vec2,
    /// Rotation and zoom origin
    pub target: Vec2,
    /// Displacement from target
    pub offset: Vec2,

    /// If "render_target" is set - camera will render to texture
    /// otherwise to the screen
    pub render_target: Option<RenderTarget>,
}

impl Camera2D {
    /// Will make camera space equals given rect
    pub fn from_display_rect(rect: Rect) -> Camera2D {
        let target = vec2(rect.x + rect.w / 2., rect.y + rect.h / 2.);

        Camera2D {
            target,
            zoom: vec2(1. / rect.w * 2., -1. / rect.h * 2.),
            offset: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
        }
    }
}

impl Default for Camera2D {
    fn default() -> Camera2D {
        Camera2D {
            zoom: vec2(1., 1.),
            offset: vec2(0., 0.),
            target: vec2(0., 0.),
            rotation: 0.,

            render_target: None,
        }
    }
}

impl Camera for Camera2D {
    fn matrix(&self) -> Mat4 {
        // gleaned from https://github.com/raysan5/raylib/blob/master/src/core.c#L1528

        // The camera in world-space is set by
        //   1. Move it to target
        //   2. Rotate by -rotation and scale by (1/zoom)
        //      When setting higher scale, it's more intuitive for the world to become bigger (= camera become smaller),
        //      not for the camera getting bigger, hence the invert. Same deal with rotation.
        //   3. Move it by (-offset);
        //      Offset defines target transform relative to screen, but since we're effectively "moving" screen (camera)
        //      we need to do it into opposite direction (inverse transform)

        // Having camera transform in world-space, inverse of it gives the modelview transform.
        // Since (A*B*C)' = C'*B'*A', the modelview is
        //   1. Move to offset
        //   2. Rotate and Scale
        //   3. Move by -target
        let mat_origin = Mat4::from_translation(vec3(-self.target.x, -self.target.y, 0.0));
        let mat_rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let mat_scale = Mat4::from_scale(vec3(self.zoom.x, self.zoom.y, 1.0));
        let mat_translation = Mat4::from_translation(vec3(self.offset.x, self.offset.y, 0.0));

        mat_translation * ((mat_scale * mat_rotation) * mat_origin)
    }

    fn depth_enabled(&self) -> bool {
        false
    }

    fn render_pass(&self) -> Option<miniquad::RenderPass> {
        self.render_target.map(|rt| rt.render_pass)
    }
}

impl Camera2D {
    /// Returns the screen space position for a 2d camera world space position
    /// Screen position in window space - from (0, 0) to (screen_width, screen_height())
    pub fn world_to_screen(&self, point: Vec2) -> Vec2 {
        let mat = self.matrix();
        let transform = mat.transform_point3(vec3(point.x, point.y, 0.));

        vec2(
            (transform.x / 2. + 0.5) * screen_width(),
            (0.5 - transform.y / 2.) * screen_height(),
        )
    }

    // Returns the world space position for a 2d camera screen space position
    // Point is a screen space position, often mouse x and y
    pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
        let point = vec2(
            point.x / screen_width() * 2. - 1.,
            1. - point.y / screen_height() * 2.,
        );
        let inv_mat = self.matrix().inverse();
        let transform = inv_mat.transform_point3(vec3(point.x, point.y, 0.));

        vec2(transform.x, transform.y)
    }
}

/// Set active 2D or 3D camera
pub fn set_camera(camera: &dyn Camera) {
    let context = get_context();

    // flush previous camera draw calls
    context.perform_render_passes();

    context.gl.render_pass(camera.render_pass());
    context.gl.depth_test(camera.depth_enabled());
    context.camera_matrix = Some(camera.matrix());
}

/// Reset default 2D camera mode
pub fn set_default_camera() {
    let context = get_context();

    // flush previous camera draw calls
    context.perform_render_passes();

    context.gl.render_pass(None);
    context.gl.depth_test(false);
    context.camera_matrix = None;
}
