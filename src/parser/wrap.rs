use std::{
    any::{type_name, Any, TypeId},
    cell::{Cell, OnceCell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseResult, RawEzpcError};

// Wrapping of Parsers. Further down, the wrapping of matchers is implemented.
// It is not commented as it is basically the same, but the code is a bit simpler
// because no dyn Any is needed: Matchers all have the same type, Parsers<Output = ?> have not.

pub enum ParserRef<O: 'static> {
    Strong(Rc<OnceCell<Box<dyn Parse<Output = O>>>>),
    Weak(Weak<OnceCell<Box<dyn Parse<Output = O>>>>),
}

pub struct WrappedParser<O: 'static> {
    parser: ParserRef<O>,
    max_depth: usize,
    name: &'static str,
}

impl<O: 'static> Parse for WrappedParser<O> {
    type Output = O;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        thread_local! { static DEPTH: Cell<usize> = const { Cell::new(0) }; }

        let depth = DEPTH.with(|d| d.get());
        if depth > self.max_depth {
            return Err(RawEzpcError::Recursion {
                max_depth: self.max_depth,
                parser_name: self.name,
                pos: input.as_ptr(),
            });
        }

        let parser = match &self.parser {
            ParserRef::Strong(p) => p.clone(),
            // The top level of the recursion is a strong ref, so these inner weak
            // refs can always be upgraded, otherwise we would not reach this point
            ParserRef::Weak(weak) => weak.upgrade().unwrap(),
        };

        DEPTH.with(|d| d.set(d.get() + 1));
        let result = parser.get().unwrap().apply(input);
        DEPTH.with(|d| d.set(d.get() - 1));

        result
    }
}

pub trait WrapParser<O> {
    fn wrap(self, max_depth: usize) -> Parser<WrappedParser<O>>;
}

impl<O: 'static + Clone, P, F> WrapParser<O> for F
where
    P: Parse<Output = O> + 'static,
    F: Fn() -> Parser<P> + 'static,
{
    fn wrap(self, max_depth: usize) -> Parser<WrappedParser<O>> {
        thread_local! {
            /// The parsers have different types based on their outputs, so they are wrapped in Any:
            /// `Box<dyn Any> contains Rc<OnceCell<Box<dyn Parse<Output = ???>>>>`
            static IN_PROGRESS: RefCell<HashMap<TypeId, Box<dyn Any>>> = Default::default();
        }

        // Use the type of the recursive parser builder Fn as the identifier key
        let type_id = self.type_id();

        if let Some(parser) = IN_PROGRESS.with(|in_prog| {
            in_prog
                .borrow()
                .get(&type_id)
                .map(|parser| Rc::downgrade(parser.downcast_ref().unwrap()))
        }) {
            return Parser(WrappedParser {
                parser: ParserRef::Weak(parser),
                max_depth,
                name: type_name::<F>(),
            });
        }

        // The parser does not exist in IN_PROGRESS, so we are at the root level
        // of the recursion. First, build an empty cell that can be used by the recursion:
        let parser_ref = Rc::new(OnceCell::new());
        IN_PROGRESS.with(|in_prog| {
            in_prog
                .borrow_mut()
                .insert(type_id, Box::new(parser_ref.clone()))
        });

        // Then build the true parser itself by executing the function. When it
        // reaches building istelf, it will use the still empty OnceCell.
        let parser: Box<dyn Parse<Output = O>> = Box::new(self().0);

        // Afterwards clean up the thread local to avoid memory leaks
        IN_PROGRESS.with(|in_prog| in_prog.borrow_mut().remove(&type_id).unwrap());

        // Now we actually have the parser and can initialize the OnceCell that is
        // already being referenced by the recursion points inside of the parser.
        parser_ref.set(parser).unwrap_or_else(|_| unreachable!());

        Parser(WrappedParser {
            parser: ParserRef::Strong(parser_ref),
            max_depth,
            name: type_name::<F>(),
        })
    }
}

// Same for matcher, see parser comments for more information.
// We don't need dyn Any because all Matchers have the same type.

pub enum MatcherRef {
    Strong(Rc<OnceCell<Box<dyn Match>>>),
    Weak(Weak<OnceCell<Box<dyn Match>>>),
}

pub struct WrappedMatcher {
    matcher: MatcherRef,
    max_depth: usize,
    name: &'static str,
}

impl Match for WrappedMatcher {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        thread_local! { static DEPTH: Cell<usize> = const { Cell::new(0) }; }

        let depth = DEPTH.with(|d| d.get());
        if depth > self.max_depth {
            return Err(RawEzpcError::Recursion {
                max_depth: self.max_depth,
                parser_name: self.name,
                pos: input.as_ptr(),
            });
        }

        let parser = match &self.matcher {
            MatcherRef::Strong(p) => p.clone(),
            MatcherRef::Weak(weak) => weak.upgrade().unwrap(),
        };

        DEPTH.with(|d| d.set(d.get() + 1));
        let result = parser.get().unwrap().apply(input);
        DEPTH.with(|d| d.set(d.get() - 1));

        result
    }
}

pub trait WrapMatcher {
    fn wrap(self, max_depth: usize) -> Matcher<WrappedMatcher>;
}

impl<M, F> WrapMatcher for F
where
    M: Match + 'static,
    F: Fn() -> Matcher<M> + 'static,
{
    fn wrap(self, max_depth: usize) -> Matcher<WrappedMatcher> {
        type MatchRef = Rc<OnceCell<Box<dyn Match>>>;
        thread_local! {
            static IN_PROGRESS: RefCell<HashMap<TypeId, MatchRef>> = Default::default();
        }
        let type_id = self.type_id();

        if let Some(matcher) = IN_PROGRESS.with(|in_prog| {
            in_prog
                .borrow()
                .get(&type_id)
                .map(Rc::downgrade)
        }) {
            return Matcher(WrappedMatcher {
                matcher: MatcherRef::Weak(matcher),
                max_depth,
                name: type_name::<F>(),
            });
        }

        let matcher_ref = Rc::new(OnceCell::new());

        IN_PROGRESS.with(|in_prog| in_prog.borrow_mut().insert(type_id, matcher_ref.clone()));
        let matcher: Box<dyn Match> = Box::new(self().0);
        IN_PROGRESS.with(|in_prog| in_prog.borrow_mut().remove(&type_id).unwrap());

        matcher_ref.set(matcher).unwrap_or_else(|_| unreachable!());
        Matcher(WrappedMatcher {
            matcher: MatcherRef::Strong(matcher_ref),
            max_depth,
            name: type_name::<F>(),
        })
    }
}
