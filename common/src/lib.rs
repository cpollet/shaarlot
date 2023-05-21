use regex::Regex;

#[derive(Clone, PartialEq)]
pub struct PasswordFlags {
    pub same: bool,
    pub length: bool,
    pub lower_case: bool,
    pub upper_case: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl PasswordFlags {
    pub fn valid() -> Self {
        Self {
            same: true,
            length: true,
            lower_case: true,
            upper_case: true,
            digits: true,
            symbols: true,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.same
            && self.length
            && self.lower_case
            && self.upper_case
            && self.digits
            && self.symbols
    }
}

impl Default for PasswordFlags {
    fn default() -> Self {
        Self {
            same: true,
            length: false,
            lower_case: false,
            upper_case: false,
            digits: false,
            symbols: false,
        }
    }
}

pub struct PasswordRules {
    min_len: usize,
    lower_case_regex: Regex,
    upper_case_regex: Regex,
    digits_regex: Regex,
    symbols_regex: Regex,
}

impl Default for PasswordRules {
    fn default() -> Self {
        Self {
            min_len: 8,
            lower_case_regex: Regex::new("[a-z]").unwrap(),
            upper_case_regex: Regex::new("[A-Z]").unwrap(),
            digits_regex: Regex::new("[0-9]").unwrap(),
            symbols_regex: Regex::new("[.\\-_+*\\\\%&/${}\\[\\]=?!\"§°~#@]").unwrap(),
        }
    }
}

impl PasswordRules {
    // fixme must take (Secret<>, Secret<>) as param
    pub fn validate<'t, P>(&self, a: P, b: P) -> PasswordFlags
    where
        P: Into<&'t str>,
    {
        let a = a.into();
        let b = b.into();
        PasswordFlags {
            same: a.eq(b),
            length: a.len() >= self.min_len,
            lower_case: self.lower_case_regex.is_match(a),
            upper_case: self.upper_case_regex.is_match(a),
            digits: self.digits_regex.is_match(a),
            symbols: self.symbols_regex.is_match(a),
        }
    }
}
