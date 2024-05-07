#![unstable(feature = "ghost_macro", issue = "none")]

//! TODO(ghost)

/// Ghost code !!!!
#[cfg(not(bootstrap))]
#[unstable(feature = "ghost_macro", issue = "none")]
#[rustc_builtin_macro]
pub macro ghost("ghost macro", ($token:ident, $mode:ident) $block:expr) {
    /* compiler built-in */
}

/// TODO(ghost)
#[unstable(feature = "ghost_macro", issue = "none")]
pub trait DerefGhost : crate::ops::Deref {
    /// TODO(ghost)
    fn deref_ghost(self) -> Self::Target;
}