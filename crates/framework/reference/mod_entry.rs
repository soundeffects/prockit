use crate::{CONVENTIONS, Convention, Names};
use linkme::DistributedSlice;
use std::any::Any;
use typeid::ConstTypeId;

#[derive(Clone, Copy)]
pub struct NamedType {
    names: Names,
    type_id: ConstTypeId,
}

impl NamedType {
    pub const fn new<T>(name_string: &str) -> Result<Self, &'static str> {
        match Names::new(name_string) {
            Ok(names) => Self::explicit::<T>(names, CONVENTIONS),
            Err(message) => Err(message),
        }
    }

    pub const fn explicit<T>(
        names: Names,
        conventions: DistributedSlice<[Convention]>,
    ) -> Result<Self, &'static str> {
        let (mut i, mut j) = (0, 0);
        while i < names.len() {
            while j < conventions.len() {
                if names.get(i).unwrap().equals(conventions[j].name()) {
                    if typeid::of::<bool>() == typeid::of::<bool>() {
                        todo!()
                    }
                }
            }
        }

        Ok(Self {
            names,
            type_id: ConstTypeId::of::<T>(),
        })
    }
}

pub struct Entry {
    function: &'static dyn Any,
    return_type: NamedType,
    argument_types: [Option<NamedType>; Self::MAX_ARGS],
}

impl Entry {
    pub const MAX_ARGS: usize = 4;

    pub const fn new_0<R: 'static>(
        function: &'static fn() -> R,
        return_names: &'static str,
    ) -> Result<Self, &'static str> {
        match NamedType::new::<R>(return_names) {
            Ok(return_type) => Ok(Self {
                function,
                return_type,
                argument_types: [None; Self::MAX_ARGS],
            }),
            Err(message) => Err(message),
        }
    }

    pub const fn new_1<R: 'static, A: 'static>(
        function: &'static fn(A) -> R,
        return_names: &'static str,
        argument_names: &'static str,
    ) -> Result<Self, &'static str> {
        match (
            NamedType::new::<R>(return_names),
            NamedType::new::<A>(argument_names),
        ) {
            (Ok(return_type), Ok(argument_type)) => Ok(Self {
                function,
                return_type,
                argument_types: [Some(argument_type), None, None, None],
            }),
            (Err(message), _) => Err(message),
            (_, Err(message)) => Err(message),
        }
    }

    pub const fn new_2<R: 'static, A1: 'static, A2: 'static>(
        function: &'static fn(A1, A2) -> R,
        r_names: &'static str,
        a1_names: &'static str,
        a2_names: &'static str,
    ) -> Result<Self, &'static str> {
        match (
            NamedType::new::<R>(r_names),
            NamedType::new::<A1>(a1_names),
            NamedType::new::<A2>(a2_names),
        ) {
            (Ok(r_type), Ok(a1_type), Ok(a2_type)) => Ok(Self {
                function,
                return_type: r_type,
                argument_types: [Some(a1_type), Some(a2_type), None, None],
            }),
            (Err(message), _, _) => Err(message),
            (_, Err(message), _) => Err(message),
            (_, _, Err(message)) => Err(message),
        }
    }

    pub const fn new_3<R: 'static, A1: 'static, A2: 'static, A3: 'static>(
        function: &'static fn(A1, A2) -> R,
        r_names: &'static str,
        a1_names: &'static str,
        a2_names: &'static str,
        a3_names: &'static str,
    ) -> Result<Self, &'static str> {
        match (
            NamedType::new::<R>(r_names),
            NamedType::new::<A1>(a1_names),
            NamedType::new::<A2>(a2_names),
            NamedType::new::<A3>(a3_names),
        ) {
            (Ok(r_type), Ok(a1_type), Ok(a2_type), Ok(a3_type)) => Ok(Self {
                function,
                return_type: r_type,
                argument_types: [Some(a1_type), Some(a2_type), Some(a3_type), None],
            }),
            (Err(message), _, _, _) => Err(message),
            (_, Err(message), _, _) => Err(message),
            (_, _, Err(message), _) => Err(message),
            (_, _, _, Err(message)) => Err(message),
        }
    }

    pub const fn new_4<R: 'static, A1: 'static, A2: 'static, A3: 'static, A4: 'static>(
        function: &'static fn(A1, A2) -> R,
        r_names: &'static str,
        a1_names: &'static str,
        a2_names: &'static str,
        a3_names: &'static str,
        a4_names: &'static str,
    ) -> Result<Self, &'static str> {
        match (
            NamedType::new::<R>(r_names),
            NamedType::new::<A1>(a1_names),
            NamedType::new::<A2>(a2_names),
            NamedType::new::<A3>(a3_names),
            NamedType::new::<A4>(a4_names),
        ) {
            (Ok(r_type), Ok(a1_type), Ok(a2_type), Ok(a3_type), Ok(a4_type)) => Ok(Self {
                function,
                return_type: r_type,
                argument_types: [Some(a1_type), Some(a2_type), Some(a3_type), Some(a4_type)],
            }),
            (Err(message), _, _, _, _) => Err(message),
            (_, Err(message), _, _, _) => Err(message),
            (_, _, Err(message), _, _) => Err(message),
            (_, _, _, Err(message), _) => Err(message),
            (_, _, _, _, Err(message)) => Err(message),
        }
    }
}

#[macro_export]
macro_rules! entry {
    (@version) => {$crate::Entry::new_0};
    (@version $a: literal) => {$crate::Entry::new_1};
    (@version $a1: literal $a2: literal) => {$crate::Entry::new_2};
    (@version $a1: literal $a2: literal $a3: literal) => {$crate::Entry::new_3};
    (@version $a1: literal $a2: literal $a3: literal $a4: literal) => {$crate::Entry::new_4};
    ($function: expr, $r_names: literal$(, $a_names: literal)*) => {
        const {
            $crate::entry!(@version $($a_names)*)($function, $r_names$(, $a_names)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{Entry, entry};

    #[test]
    fn test_zero_args() {
        assert!(Entry::new_0(|| 0.0, "zero").is_ok());
    }

    #[test]
    fn test_one_arg() {
        assert!(Entry::new_1(|i: i32| i += 1, "increment", "i, a").is_ok());
    }

    #[test]
    fn test_two_args() {
        assert!(
            Entry::new_2(
                |x: f32, y: f32| x + y,
                "add, addition",
                "x, a, first",
                "y, b, second"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_three_args() {
        assert!(
            Entry::new_3(
                |x: f32, y: f32, z: f32| [x, y, z],
                "position, vec3",
                "x",
                "y",
                "z"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_four_args() {
        assert!(
            Entry::new_4(
                |w: f32, x: f32, y: f32, z: f32| [w, x, y, z],
                "rotation, quat, vec4",
                "w",
                "x",
                "y",
                "z"
            )
            .is_ok()
        );
    }

    #[test]
    fn entry_macro_0() {
        assert!(entry!(|| 0.0, "zero").is_ok());
    }

    #[test]
    fn entry_macro_1() {
        assert!(entry!(|i: i32| i + 1, "increment", "i").is_ok());
    }

    #[test]
    fn entry_macro_2() {
        assert!(
            entry!(
                |x: f32, y: f32| x + y,
                "add, addition",
                "x, a, first",
                "y, b, second"
            )
            .is_ok()
        );
    }

    #[test]
    fn entry_macro_3() {
        assert!(
            entry!(
                |x: f32, y: f32, z: f32| [x, y, z],
                "position, vec3",
                "x",
                "y",
                "z"
            )
            .is_ok()
        );
    }

    #[test]
    fn entry_macro_4() {
        assert!(
            entry!(
                |w: f32, x: f32, y: f32, z: f32| [w, x, y, z],
                "rotation, quat, vec4",
                "w",
                "x",
                "y",
                "z"
            )
            .is_ok()
        );
    }
}
