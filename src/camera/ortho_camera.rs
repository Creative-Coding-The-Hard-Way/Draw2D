use nalgebra as na;

use crate::geometry::Rect;

use super::OrthoCamera;

impl OrthoCamera {
    /// Build a new camera with a given viewport height and aspect ratio.
    ///
    /// # Params
    ///
    /// - `viewport_height` defines the height of the view rectangle in world
    ///   space.
    /// - `aspect_ratio` is the ratio of the desired viewport's `width/height`.
    pub fn with_viewport(viewport_height: f32, aspect_ratio: f32) -> Self {
        let viewport_width = viewport_height * aspect_ratio;
        Self {
            projection: Self::centered_ortho(viewport_width, viewport_height),
            view: na::Translation2::identity(),
            viewport_height,
            viewport_width,
        }
    }

    /// Get the camera's full transformation matrix. This can be passed to a
    /// shader for transformations.
    pub fn as_matrix(&self) -> na::Matrix4<f32> {
        let view_3d = na::Translation3::new(self.view.x, self.view.y, 0.0);
        self.projection.as_matrix() * view_3d.to_homogeneous()
    }

    /// The camera's bounds in world-space.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// let ortho = OrthoCamera::with_viewport(1.0, 2.0);
    /// let bounds = ortho.bounds();
    ///
    /// assert_relative_eq!(bounds.left, -1.0);
    /// assert_relative_eq!(bounds.right, 1.0);
    /// assert_relative_eq!(bounds.top, 0.5);
    /// assert_relative_eq!(bounds.bottom, -0.5);
    /// ```
    pub fn bounds(&self) -> Rect<f32> {
        let viewport_top_left = na::Point2::new(
            -self.viewport_width / 2.0,
            self.viewport_height / 2.0,
        );
        let viewport_bottom_right = na::Point2::new(
            self.viewport_width / 2.0,
            -self.viewport_height / 2.0,
        );
        let inverse = self.view.inverse();
        let world_top_left = inverse.transform_point(&viewport_top_left);
        let world_bottom_right =
            inverse.transform_point(&viewport_bottom_right);
        Rect {
            left: world_top_left.x,
            right: world_bottom_right.x,
            top: world_top_left.y,
            bottom: world_bottom_right.y,
        }
    }

    /// Set the camera's position in world-space.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// let mut ortho = OrthoCamera::with_viewport(2.0, 1.0);
    /// ortho.set_world_position(&na::Point2::new(30.0, -0.5));
    ///
    /// assert_relative_eq!(
    ///   ortho.world_position(),
    ///   na::Point2::new(30.0, -0.5)
    /// );
    ///
    /// let bounds = ortho.bounds();
    /// assert_relative_eq!(bounds.left, -1.0 + 30.0);
    /// assert_relative_eq!(bounds.right, 1.0 + 30.0);
    /// assert_relative_eq!(bounds.top, 1.0 - 0.5);
    /// assert_relative_eq!(bounds.bottom, -1.0 - 0.5);
    /// ```
    pub fn set_world_position(&mut self, world_pos: &na::Point2<f32>) {
        self.view.x = -world_pos.x;
        self.view.y = -world_pos.y;
    }

    /// Get the camera's position in world space.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// let pos = OrthoCamera::with_viewport(1.0, 1.0).world_position();
    ///
    /// assert_relative_eq!(pos, na::Point2::new(0.0, 0.0));
    /// ```
    pub fn world_position(&self) -> na::Point2<f32> {
        na::Point2::new(-self.view.x, -self.view.y)
    }

    /// Resize the viewport's width such that the viewing rectangle has the
    /// desired aspect ratio.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// let mut ortho = OrthoCamera::with_viewport(1.0, 1.0);
    /// ortho.set_aspect_ratio(2.0);
    ///
    /// let bounds = ortho.bounds();
    /// assert_relative_eq!(bounds.left, -1.0);
    /// assert_relative_eq!(bounds.right, 1.0);
    /// assert_relative_eq!(bounds.top, 0.5);
    /// assert_relative_eq!(bounds.bottom, -0.5);
    /// ```
    pub fn set_aspect_ratio(&mut self, desired_aspect_ratio: f32) {
        self.viewport_width = self.viewport_height * desired_aspect_ratio;
        self.projection =
            Self::centered_ortho(self.viewport_width, self.viewport_height);
    }

    /// The camera viewport's aspect ratio.
    pub fn aspect_ratio(&self) -> f32 {
        self.viewport_width / self.viewport_height
    }

    /// Get the height of the viewport.
    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }

    /// Get the width of the viewport.
    pub fn viewport_width(&self) -> f32 {
        self.viewport_width
    }

    /// Set the viewport's height to a new value.
    ///
    /// Automatically resizes the viewport's width to maintain the current
    /// aspect ratio.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// // a camera which is 3x as wide as it is tall
    /// let mut ortho = OrthoCamera::with_viewport(2.0, 3.0);
    ///
    /// assert_relative_eq!(ortho.aspect_ratio(), 3.0);
    /// assert_relative_eq!(ortho.viewport_height(), 2.0);
    /// assert_relative_eq!(ortho.viewport_width(), 6.0);
    ///
    /// ortho.set_viewport_height(3.3);
    ///
    /// assert_relative_eq!(ortho.aspect_ratio(), 3.0);
    /// assert_relative_eq!(ortho.viewport_height(), 3.3);
    /// assert_relative_eq!(ortho.viewport_width(), 9.9);
    /// ```
    pub fn set_viewport_height(&mut self, desired_height: f32) {
        let current_aspect_ratio = self.aspect_ratio();
        self.viewport_height = desired_height;
        self.set_aspect_ratio(current_aspect_ratio);
    }

    /// Unproject a vector from normalized device coordinates (NDC) to view
    /// space.
    ///
    /// Vectors are just a direction and a magnitude, so this transformation
    /// does not apply the camera's translation in world space.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// // a camera which is 3x as wide as it is tall
    /// let mut ortho = OrthoCamera::with_viewport(2.0, 3.0);
    ///
    /// // the camera's world position is ignored when unprojecting a vector
    /// ortho.set_world_position(&na::Point2::new(100.0, -34523.0));
    ///
    /// // Vulkan ndc coords have Y ranging from -1 at the top of the screen,
    /// // to 1 at the bottom of the screen.
    /// let top_right_ndc = na::Vector2::new(1.0, -1.0);
    ///
    /// // The unprojected vector should point to the top right of the viewport
    /// // rectangle, but is not influenced by the camera's world position.
    /// let unprojected = ortho.unproject_vec(&top_right_ndc);
    /// assert_relative_eq!(unprojected, na::Vector2::new(3.0, 1.0));
    /// ```
    pub fn unproject_vec(&self, ndc: &na::Vector2<f32>) -> na::Vector2<f32> {
        self.projection
            .inverse()
            .transform_vector(&na::Vector3::new(ndc.x, ndc.y, 0.0))
            .xy()
    }

    /// Unproject a point from normalized device coordinates (NDC) to world
    /// space.
    ///
    /// Points are logically a specific location in space. As such, the point's
    /// coordinates will be transformed b ythe camera's location in world
    /// space.
    ///
    /// e.g. this method returns where the ndc point would *actually* be
    /// located in world coordinates.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use draw2d::camera::*;
    /// # use approx::assert_relative_eq;
    /// # use nalgebra as na;
    /// #
    /// // a camera which is 3x as wide as it is tall
    /// let mut ortho = OrthoCamera::with_viewport(2.0, 3.0);
    ///
    /// // the camera's world position is ignored when unprojecting a vector
    /// ortho.set_world_position(&na::Point2::new(100.0, -34523.0));
    ///
    /// // Vulkan ndc coords have Y ranging from -1 at the top of the screen,
    /// // to 1 at the bottom of the screen.
    /// let bottom_left_ndc = na::Point2::new(-1.0, 1.0);
    ///
    /// // The unprojected point should account for both the camera's viewing
    /// // rectangle, and the camera's world position.
    /// let unprojected = ortho.unproject_point(&bottom_left_ndc);
    /// assert_relative_eq!(
    ///     unprojected,
    ///     na::Point2::new(-3.0, -1.0) + ortho.world_position().coords
    /// );
    /// ```
    pub fn unproject_point(&self, ndc: &na::Point2<f32>) -> na::Point2<f32> {
        let unprojected = self.unproject_vec(&ndc.coords);
        self.view
            .inverse_transform_point(&na::Point2::from(unprojected))
    }

    /// Construct an orthographic projection centered around the origin with
    /// the provided width and height.
    fn centered_ortho(width: f32, height: f32) -> na::Orthographic3<f32> {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        na::Orthographic3::new(
            -half_width,
            half_width,
            half_height,
            -half_height,
            1.0,
            -1.0,
        )
    }
}
