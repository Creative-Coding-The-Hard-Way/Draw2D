mod rect;

use nalgebra as na;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rect<T: na::Scalar> {
    pub left: T,
    pub right: T,
    pub bottom: T,
    pub top: T,
}
