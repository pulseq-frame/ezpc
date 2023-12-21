use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Input {
    text: Rc<str>,
    pos: usize,
}

impl Input {
    pub fn starts_with(&self, pat: &str) -> bool {
        self.text[self.pos..].starts_with(pat)
    }

    pub fn skip(&self, bytes: usize) -> Self {
        Self {
            text: self.text.clone(),
            pos: self.pos + bytes,
        }
    }

    pub fn pop_char(&self) -> Option<(char, Self)> {
        if let Some(c) = self.text[self.pos..].chars().next() {
            Some((
                c,
                Self {
                    text: self.text.clone(),
                    pos: self.pos + c.len_utf8(),
                },
            ))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text[self.pos..]
    }

    pub fn get_consumed<'a>(&'a self, before: &'a Self) -> &'a str {
        assert!(Rc::ptr_eq(&self.text, &before.text));
        &before.text[before.pos..self.pos]
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.text.len()
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Self {
            text: Rc::from(value),
            pos: 0,
        }
    }
}