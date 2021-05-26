use nalgebra as na;

use super::Rect;

impl<T: na::RealField> Rect<T> {
    /// Check if a point is contained within this rectangle.
    ///
    /// # Returns
    ///
    /// - `true` when the point is contained within the rectangle. Comparison
    ///   is inclusive, so the edges of the rectangle are considered 'inside'.
    pub fn contains(&self, vec: &na::Vector2<T>) -> bool {
        if vec.x >= self.left
            && vec.x <= self.right
            && vec.y >= self.bottom
            && vec.y <= self.top
        {
            true
        } else {
            false
        }
    }

    pub fn width(&self) -> T {
        (self.right - self.left).abs()
    }

    pub fn height(&self) -> T {
        (self.top - self.bottom).abs()
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    use nalgebra as na;

    #[test]
    fn not_inside() {
        let rect = Rect {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
        };
        let point = na::Point2::new(-2.0, 2.0);
        assert!(
            rect.contains(&point.coords) == false,
            "the rectangle {:#?} should not contain the point {}",
            rect,
            point
        );
    }

    #[test]
    fn inside() {
        let rect = Rect {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
        };
        let point = na::Point2::new(0.5, -0.5);
        assert!(
            rect.contains(&point.coords) == true,
            "the rectangle {:#?} should contain the point {}",
            rect,
            point
        );
    }
}
