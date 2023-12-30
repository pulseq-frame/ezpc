use std::{
    any::{type_name, Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::{Display, Pointer},
    rc::{Rc, Weak},
};

use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseError, ParseResult};

// Wrapping of Parsers. Further down, the wrapping of matchers is implemented.
// It is not commented as it is basically the same, but the code is a bit simpler
// because no dyn Any is needed: Matchers all have the same type, Parsers<Output = ?> have not.

pub enum ParserRef<O: 'static> {
    Strong(Rc<RefCell<Box<dyn Parse<Output = O>>>>),
    Weak(Weak<RefCell<Box<dyn Parse<Output = O>>>>),
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
            return Err(ParseError::RecursionDepth(depth));
        }

        let parser = match &self.parser {
            ParserRef::Strong(p) => p.clone(),
            // The top level of the recursion is a strong ref, so these inner weak
            // refs can always be upgraded, otherwise we would not reach this point
            ParserRef::Weak(weak) => weak.upgrade().unwrap(),
        };

        DEPTH.with(|d| d.set(d.get() + 1));
        let result = parser.borrow().apply(input);
        DEPTH.with(|d| d.set(d.get() - 1));

        result
    }
}

impl<O: 'static> Display for WrappedParser<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.parser {
            ParserRef::Strong(p) => p.fmt(f),
            ParserRef::Weak(_) => write!(f, "Recursion({})", self.name),
        }
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
            /// `Box<dyn Any> contains Rc<RefCell<Box<dyn Parse<Output = ???>>>>`
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
        // of the recursion. First, build a dummy that can be used in the recursion:
        let dummy_parser: Box<dyn Parse<Output = O>> = Box::new(
            crate::tag("dummy")
                .map(|_| panic!("This parser should never be used!"))
                .0,
        );
        let dummy = Rc::new(RefCell::new(dummy_parser));
        IN_PROGRESS.with(|in_prog| {
            in_prog
                .borrow_mut()
                .insert(type_id, Box::new(dummy.clone()))
        });

        // Then build the true parser itself by executing the function. When it
        // reaches building istelf, it will use the dummy instead.
        let parser: Box<dyn Parse<Output = O>> = Box::new(self().0);

        // Afterwards clean up the thread local to avoid memory leaks
        IN_PROGRESS.with(|in_prog| in_prog.borrow_mut().remove(&type_id).unwrap());

        // Now we need to fix the dummies used in the recursion by replacing it
        // with the true parser. This is why we used a RefCell, so we can do this
        // from the outside. Alternatively, we could step recursively through
        // the parser and swap the reference. This would remove one indirection and
        // the need for interior mutability, but is a much more invasive implementation.
        *(dummy.borrow_mut()) = parser;
        // Return the fixed parser
        Parser(WrappedParser {
            parser: ParserRef::Strong(dummy),
            max_depth,
            name: type_name::<F>(),
        })
    }
}

// Same for matcher, see parser comments for more information.
// We don't need dyn Any because all Matchers have the same type.

pub enum MatcherRef {
    Strong(Rc<RefCell<Box<dyn Match>>>),
    Weak(Weak<RefCell<Box<dyn Match>>>),
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
            return Err(ParseError::RecursionDepth(depth));
        }

        let parser = match &self.matcher {
            MatcherRef::Strong(p) => p.clone(),
            MatcherRef::Weak(weak) => weak.upgrade().unwrap(),
        };

        DEPTH.with(|d| d.set(d.get() + 1));
        let result = parser.borrow().apply(input);
        DEPTH.with(|d| d.set(d.get() - 1));

        result
    }
}

impl Display for WrappedMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.matcher {
            MatcherRef::Strong(p) => p.fmt(f),
            MatcherRef::Weak(_) => write!(f, "Recursion({})", self.name),
        }
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
        thread_local! {
            static IN_PROGRESS: RefCell<HashMap<TypeId, Rc<RefCell<Box<dyn Match>>>>> = Default::default();
        }
        let type_id = self.type_id();

        if let Some(matcher) = IN_PROGRESS.with(|in_prog| {
            in_prog
                .borrow()
                .get(&type_id)
                .map(|matcher| Rc::downgrade(matcher))
        }) {
            return Matcher(WrappedMatcher {
                matcher: MatcherRef::Weak(matcher),
                max_depth,
                name: type_name::<F>(),
            });
        }

        let dummy_matcher: Box<dyn Match> = Box::new(crate::tag("dummy").0);
        let dummy = Rc::new(RefCell::new(dummy_matcher));

        IN_PROGRESS.with(|in_prog| in_prog.borrow_mut().insert(type_id, dummy.clone()));
        let matcher: Box<dyn Match> = Box::new(self().0);
        IN_PROGRESS.with(|in_prog| in_prog.borrow_mut().remove(&type_id).unwrap());

        *(dummy.borrow_mut()) = matcher;
        Matcher(WrappedMatcher {
            matcher: MatcherRef::Strong(dummy),
            max_depth,
            name: type_name::<F>(),
        })
    }
}
