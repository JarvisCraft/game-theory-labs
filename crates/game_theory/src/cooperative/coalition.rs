use std::{
    fmt::{self, Display, Formatter},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Coalition(pub(super) usize);

impl Display for Coalition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(bits) = self;
        write!(f, "{bits:b}")
    }
}

impl Coalition {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn n_members(&self) -> u8 {
        let _: () = assert!(usize::BITS <= u8::MAX as u32);
        self.0.count_ones() as u8
    }

    pub const fn overlaps(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl Default for Coalition {
    fn default() -> Self {
        Self::empty()
    }
}

impl BitOr for Coalition {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Coalition {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Coalition {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Coalition {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Sub for Coalition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

impl SubAssign for Coalition {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}
