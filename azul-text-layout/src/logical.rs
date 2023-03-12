#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LogicalRect {
    pub origin: LogicalPosition,
    pub size: LogicalSize,
}

impl core::fmt::Debug for LogicalRect {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{} @ {}", self.size, self.origin)
    }
}

impl core::fmt::Display for LogicalRect {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{} @ {}", self.size, self.origin)
    }
}

impl LogicalRect {
    pub const fn zero() -> Self {
        Self::new(LogicalPosition::zero(), LogicalSize::zero())
    }
    pub const fn new(origin: LogicalPosition, size: LogicalSize) -> Self {
        Self { origin, size }
    }

    #[inline(always)]
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }
    #[inline(always)]
    pub fn min_x(&self) -> f32 {
        self.origin.x
    }
    #[inline(always)]
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }
    #[inline(always)]
    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    /// Faster union for a Vec<LayoutRect>
    #[inline]
    pub fn union<I: Iterator<Item = Self>>(mut rects: I) -> Option<Self> {
        let first = rects.next()?;

        let mut max_width = first.size.width;
        let mut max_height = first.size.height;
        let mut min_x = first.origin.x;
        let mut min_y = first.origin.y;

        for Self {
            origin: LogicalPosition { x, y },
            size: LogicalSize { width, height },
        } in rects
        {
            let cur_lower_right_x = x + width;
            let cur_lower_right_y = y + height;
            max_width = max_width.max(cur_lower_right_x - min_x);
            max_height = max_height.max(cur_lower_right_y - min_y);
            min_x = min_x.min(x);
            min_y = min_y.min(y);
        }

        Some(Self {
            origin: LogicalPosition { x: min_x, y: min_y },
            size: LogicalSize {
                width: max_width,
                height: max_height,
            },
        })
    }

    /// Same as `contains()`, but returns the (x, y) offset of the hit point
    ///
    /// On a regular computer this function takes ~3.2ns to run
    #[inline]
    pub fn hit_test(&self, other: &LogicalPosition) -> Option<LogicalPosition> {
        let dx_left_edge = other.x - self.min_x();
        let dx_right_edge = self.max_x() - other.x;
        let dy_top_edge = other.y - self.min_y();
        let dy_bottom_edge = self.max_y() - other.y;
        if dx_left_edge > 0.0 && dx_right_edge > 0.0 && dy_top_edge > 0.0 && dy_bottom_edge > 0.0 {
            Some(LogicalPosition::new(dx_left_edge, dy_top_edge))
        } else {
            None
        }
    }

    // pub fn to_layout_rect(&self) -> LayoutRect {
    //     LayoutRect {
    //         origin: LayoutPoint::new(
    //             libm::roundf(self.origin.x) as isize,
    //             libm::roundf(self.origin.y) as isize,
    //         ),
    //         size: LayoutSize::new(
    //             libm::roundf(self.size.width) as isize,
    //             libm::roundf(self.size.height) as isize,
    //         ),
    //     }
    // }
}

use core::ops::AddAssign;
use core::ops::SubAssign;
use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops;

#[derive(Default, Copy, Clone)]
pub struct LogicalPosition {
    pub x: f32,
    pub y: f32,
}

impl LogicalPosition {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    #[inline(always)]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl SubAssign<LogicalPosition> for LogicalPosition {
    fn sub_assign(&mut self, other: LogicalPosition) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl AddAssign<LogicalPosition> for LogicalPosition {
    fn add_assign(&mut self, other: LogicalPosition) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl core::fmt::Debug for LogicalPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl core::fmt::Display for LogicalPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl ops::Add for LogicalPosition {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for LogicalPosition {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

const DECIMAL_MULTIPLIER: f32 = 1000.0;

impl PartialOrd for LogicalPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for LogicalPosition {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Ord for LogicalPosition {
    fn cmp(&self, other: &LogicalPosition) -> Ordering {
        let self_x = (self.x * DECIMAL_MULTIPLIER) as usize;
        let self_y = (self.y * DECIMAL_MULTIPLIER) as usize;
        let other_x = (other.x * DECIMAL_MULTIPLIER) as usize;
        let other_y = (other.y * DECIMAL_MULTIPLIER) as usize;
        self_x.cmp(&other_x).then(self_y.cmp(&other_y))
    }
}

impl Eq for LogicalPosition {}

impl Hash for LogicalPosition {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let self_x = (self.x * DECIMAL_MULTIPLIER) as usize;
        let self_y = (self.y * DECIMAL_MULTIPLIER) as usize;
        self_x.hash(state);
        self_y.hash(state);
    }
}

#[derive(Default, Copy, Clone)]
pub struct LogicalSize {
    pub width: f32,
    pub height: f32,
}

impl LogicalSize {
    #[inline(always)]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    #[inline(always)]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl core::fmt::Debug for LogicalSize {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl core::fmt::Display for LogicalSize {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl PartialOrd for LogicalSize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for LogicalSize {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Ord for LogicalSize {
    fn cmp(&self, other: &LogicalSize) -> Ordering {
        let self_width = (self.width * DECIMAL_MULTIPLIER) as usize;
        let self_height = (self.height * DECIMAL_MULTIPLIER) as usize;
        let other_width = (other.width * DECIMAL_MULTIPLIER) as usize;
        let other_height = (other.height * DECIMAL_MULTIPLIER) as usize;
        self_width
            .cmp(&other_width)
            .then(self_height.cmp(&other_height))
    }
}

impl Eq for LogicalSize {}

impl Hash for LogicalSize {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let self_width = (self.width * DECIMAL_MULTIPLIER) as usize;
        let self_height = (self.height * DECIMAL_MULTIPLIER) as usize;
        self_width.hash(state);
        self_height.hash(state);
    }
}
