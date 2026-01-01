use crate::Entry;
use bevy::prelude::Vec;
use std::any::TypeId;

#[derive(PartialEq, Eq, Debug)]
pub enum Needs {
    None,
    Ref,
    Mut,
}

#[derive(Default)]
pub struct Provides {
    by: Option<TypeId>,
    entries: Vec<(Entry, Needs)>,
}

impl Provides {
    pub fn unchecked(by: Option<TypeId>, entries: Vec<(Entry, Needs)>) -> Self {
        Self { by, entries }
    }

    pub fn entries(&self) -> impl Iterator<Item = &(Entry, Needs)> {
        self.entries.iter()
    }
}

#[macro_export]
macro_rules! self_provides {
    (@first $first: literal $($rest: literal)*) => {$first};
    (@needs ref) => {$crate::Needs::Ref};
    (@needs mut) => {$crate::Needs::Mut};
    (@needs) => {$crate::Needs::None};

    ($(entry $($needs: ident)? $function: expr, $($ret_lit: literal)+ $(, $($arg_lit: literal)+)*),+) => {
        {
            const {
                assert!(!$crate::NameSet::check_duplicates([$($crate::self_provides!(@first $($ret_lit)*)),*]),
                    "There was a duplicate among the first names of the returns, which is disallowed!");
            };
            $crate::Provides::unchecked(Some(std::any::TypeId::of::<Self>()), vec![
                $(
                    (
                        $crate::entry!($function, $($ret_lit)*$(, $($arg_lit)*)*),
                        $crate::self_provides!(@needs $($needs)?)
                    )
                ),*
            ])
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn provides_macro() {
        use super::{Needs, Provides};
        struct _Test;

        impl _Test {
            fn _provides() -> Provides {
                let provides = self_provides! {
                    entry || 0.0,
                    "zero" "none",
                    entry ref |test: &_Test| test,
                    "identity",
                    "test" "x" "input",
                    entry |x: f32, y: f32| x + y,
                    "add" "addition",
                    "x" "first",
                    "y" "second" "last"
                };

                let needs = [Needs::None, Needs::Ref, Needs::None];
                for (expected_needs, (_, actual_needs)) in needs.iter().zip(provides.entries()) {
                    assert_eq!(*expected_needs, *actual_needs);
                }
                provides
            }
        }
    }
}
