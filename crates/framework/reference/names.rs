use linkme::distributed_slice;
use typeid::ConstTypeId;

// TODO: Add API that allows a slice to string names, rather than all in one string

#[derive(Clone, Copy)]
pub(crate) struct Name {
    characters: [char; Self::MAX_LETTERS],
}

impl Name {
    pub(crate) const MAX_LETTERS: usize = 16;

    const fn copy(&mut self, bytes: &[u8], start_index: &mut usize) -> Result<(), &'static str> {
        // Trim all leading spaces
        while *start_index < bytes.len() && bytes[*start_index] == b' ' {
            *start_index += 1;
        }

        // Check to make sure name has characters
        if *start_index >= bytes.len() || bytes[*start_index] == b',' {
            return Err("empty name!");
        }

        // Copy word
        let mut letter_index = 0;
        while *start_index < bytes.len()
            && bytes[*start_index] != b','
            && letter_index < Self::MAX_LETTERS
        {
            self.characters[letter_index] = bytes[*start_index] as char;
            *start_index += 1;
            letter_index += 1;
        }

        // Check name length
        if *start_index < bytes.len() && bytes[*start_index] != b',' {
            return Err("Word length was greater than the maximum allowed characters");
        }

        Ok(())
    }

    pub(crate) const fn empty() -> Self {
        Self {
            characters: [' '; Self::MAX_LETTERS],
        }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.characters[0] == ' '
    }

    pub(crate) const fn equals(&self, other: &Name) -> bool {
        let mut i = 0;
        while i < Self::MAX_LETTERS {
            if self.characters[i] != other.characters[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    pub(crate) const fn from_str(string: &str) -> Result<Self, &'static str> {
        let mut name = Self::empty();
        let mut byte_index = 0;
        match name.copy(string.as_bytes(), &mut byte_index) {
            Ok(()) => {
                if byte_index < string.len() {
                    Err("The provided string had multiple names, when only one was expected!")
                } else {
                    Ok(name)
                }
            }
            Err(message) => Err(message),
        }
    }
}

pub struct Convention {
    name: Name,
    type_id: ConstTypeId,
}

impl Convention {
    pub const fn unwrapped<T>(name: &str) -> Self {
        match Self::new::<T>(name) {
            Ok(convention) => convention,
            Err(_) => {
                panic!("Error in creating convention in const context");
            }
        }
    }

    pub const fn new<T>(name: &str) -> Result<Self, &'static str> {
        match Name::from_str(name) {
            Ok(name) => Ok(Self::from_name::<T>(name)),
            Err(message) => Err(message),
        }
    }

    pub const fn from_name<T>(name: Name) -> Self {
        Self {
            name,
            type_id: ConstTypeId::of::<T>(),
        }
    }

    pub const fn name(&self) -> &Name {
        &self.name
    }
}

// TODO: Add the convention source crate as field
#[distributed_slice]
pub static CONVENTIONS: [Convention];

#[macro_export]
macro_rules! convention {
    ($static_id: ident: $type: ident = $names: literal) => {
        #[distributed_slice($crate::CONVENTIONS)]
        static $static_id: $crate::Convention = $crate::Convention::unwrapped::<$type>($names);
    };
    ($conventions_id: ident, $static_id: ident: $type: ident = $names: literal) => {
        #[distributed_slice($conventions_id)]
        static $static_id: $crate::NamedType = $crate::Convention::unwrapped::<$type>($names);
    };
}

convention!(SOLID: bool = "solid");

#[derive(Clone, Copy)]
pub struct Names {
    names: [Name; Self::MAX_WORDS],
    len: usize,
}

impl Names {
    pub const MAX_WORDS: usize = 8;

    const fn split_and_trim(
        sequence: &str,
        mut buffer: [Name; Self::MAX_WORDS],
    ) -> Result<usize, &'static str> {
        let bytes = sequence.as_bytes();
        let (mut name_index, mut byte_index) = (0, 0);
        while name_index < Self::MAX_WORDS {
            // If we reached the end, don't loop
            if byte_index >= bytes.len() {
                break;
            }

            if let Err(message) = buffer[name_index].copy(bytes, &mut byte_index) {
                return Err(message);
            }

            byte_index += 1;
            name_index += 1;
        }

        // Check word count
        if byte_index < bytes.len() {
            return Err("Too many names provided");
        }

        Ok(name_index)
    }

    const fn duplicates(names: [Name; Self::MAX_WORDS]) -> bool {
        let mut i = 0;
        while i < Self::MAX_WORDS {
            // No words after first empty
            if names[i].is_empty() {
                break;
            }

            let mut j = i + 1;
            while j < Self::MAX_WORDS {
                // No words after first empty
                if names[j].is_empty() {
                    break;
                } else if names[i].equals(&names[j]) {
                    return true;
                }
                j += 1;
            }
            i += 1;
        }
        false
    }

    pub const fn new(sequence: &str) -> Result<Self, &'static str> {
        let names = [Name::empty(); Self::MAX_WORDS];
        match Self::split_and_trim(sequence, names) {
            Ok(name_count) => match Self::duplicates(names) {
                true => Err("duplicates found"),
                false => Ok(Self {
                    names,
                    len: name_count,
                }),
            },
            Err(message) => Err(message),
        }
    }

    pub const fn get(&self, index: usize) -> Option<Name> {
        if index >= Self::MAX_WORDS {
            None
        } else {
            Some(self.names[index])
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::Names;

    #[test]
    fn parsing_ok() {
        assert!(Names::new("test").is_ok());
        assert!(Names::new("test, test2").is_ok());
    }

    #[test]
    fn parsing_end_before_name() {
        assert!(Names::new("test,").is_err());
        assert!(Names::new("test, test2,").is_err());
    }

    #[test]
    fn parsing_comma_before_name() {
        assert!(Names::new("test,,test2").is_err());
        assert!(Names::new("test,test2,,test3").is_err());
    }

    #[test]
    fn parsing_empty_string() {
        assert!(Names::new("").is_err());
    }

    #[test]
    fn parsing_comma_only() {
        assert!(Names::new(",").is_err());
    }

    #[test]
    fn too_many_characters() {
        assert!(Names::new("abcdefghijklmnopqrstuvwxyz, other").is_err());
        assert!(Names::new("first, abcdefghijklmnopqrstuvwxyz").is_err());
    }

    #[test]
    fn too_many_names() {
        assert!(Names::new("1, 2, 3, 4, 5, 6, 7, 8, 9").is_err());
    }

    #[test]
    fn duplicates() {
        assert!(Names::new("double, double").is_err());
        assert!(Names::new("  double,      double").is_err());
    }

    #[test]
    fn trailing_spaces() {
        assert!(Names::new("test, test2   ").is_err());
        assert!(Names::new("test, test2                        , test3").is_err());
    }
}
