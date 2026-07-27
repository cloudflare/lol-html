use super::SelectorState;
use super::ast::NthChild;
use super::program::AddressRange;
use crate::html::{LocalName, Namespace, Tag};
use crate::memory::{LimitedVec, MemoryLimitExceededError, SharedMemoryLimiter};
use crate::selectors_vm::DenseHashSet;
// use hashbrown for raw entry, switch back to std once it stablizes there
use hashbrown::HashMap;
use hashbrown::hash_map::RawEntryMut;
use std::hash::BuildHasher;

#[inline]
fn is_void_element(local_name: &LocalName<'_>, enable_esi_tags: bool) -> bool {
    // NOTE: fast path for the most commonly used elements
    if tag_is_one_of!(*local_name, [Div, A, Span, Li]) {
        return false;
    }

    if tag_is_one_of!(
        *local_name,
        [
            Area, Base, Basefont, Bgsound, Br, Col, Embed, Hr, Img, Input, Keygen, Link, Meta,
            Param, Source, Track, Wbr
        ]
    ) {
        return true;
    }

    if enable_esi_tags {
        if let LocalName::Bytes(bytes) = local_name {
            // https://www.w3.org/TR/esi-lang/
            if &**bytes == b"esi:include" || &**bytes == b"esi:comment" {
                return true;
            }
        }
    }

    false
}

pub(crate) trait ElementData: 'static {
    fn matched_ids_mut(&mut self) -> &mut DenseHashSet;
    fn new() -> Self;
}

pub(crate) enum StackDirective {
    Push,
    PushIfNotSelfClosing,
    PopImmediately,
}

#[derive(Default)]
pub(crate) struct ChildCounter {
    cumulative: i32,
}

impl ChildCounter {
    #[inline]
    #[must_use]
    pub const fn new_and_inc() -> Self {
        Self { cumulative: 1 }
    }

    #[inline]
    pub fn inc(&mut self) {
        self.cumulative += 1;
    }

    #[inline]
    #[must_use]
    pub const fn is_nth(&self, nth: NthChild) -> bool {
        nth.has_index(self.cumulative)
    }
}

struct CounterItem {
    /// The counter at this index
    pub counter: ChildCounter,
    /// The index of this counter in the stack
    pub index: usize,
}

struct CounterList {
    items: Vec<CounterItem>,
    // we always have at least one item, an empty list shouldn't exist in the map
    current: CounterItem,
}

impl CounterList {
    pub const fn new(start: usize) -> Self {
        Self {
            items: Vec::new(),
            current: CounterItem {
                counter: ChildCounter::new_and_inc(),
                index: start,
            },
        }
    }
}

/// A more efficient counter that only requires one owned local name to track counters across multiple stack frames
pub(crate) struct TypedChildCounterMap(HashMap<LocalName<'static>, CounterList>);

impl TypedChildCounterMap {
    #[must_use]
    #[inline]
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    fn hash_name(&self, name: &LocalName<'_>) -> u64 {
        self.0.hasher().hash_one(name)
    }

    /// Adds a seen child to the map. The index is the level of the item
    pub fn add_child(&mut self, name: &LocalName<'_>, index: usize) {
        let hash = self.hash_name(name);
        let entry = self.0.raw_entry_mut().from_hash(hash, |n| name == n);
        match entry {
            RawEntryMut::Vacant(vacant) => {
                vacant.insert_hashed_nocheck(
                    hash,
                    name.clone().into_owned(), // the hash won't change just because we've got ownership
                    CounterList::new(index),
                );
            }
            RawEntryMut::Occupied(mut occupied) => {
                let CounterList { items, current } = occupied.get_mut();
                if current.index == index {
                    current.counter.inc();
                } else {
                    let counter = ChildCounter::new_and_inc();
                    let old = std::mem::replace(current, CounterItem { counter, index });
                    items.push(old);
                }
            }
        }
    }

    #[inline]
    pub fn pop_to(&mut self, index: usize) {
        self.0.retain(|_, v| {
            while v.current.index > index {
                match v.items.pop() {
                    Some(next) => {
                        v.current = next;
                    }
                    None => return false,
                }
            }
            true
        });
    }

    #[inline]
    pub fn get<'a, 'i>(&'a self, name: &LocalName<'i>, index: usize) -> Option<&'i ChildCounter>
    where
        'a: 'i,
    {
        match self.0.get(name) {
            Some(CounterList {
                current:
                    CounterItem {
                        counter,
                        index: current_index,
                    },
                ..
            }) if *current_index == index => Some(counter),
            _ => None,
        }
    }
}

pub(crate) struct StackItem<'i, E: ElementData> {
    pub local_name: LocalName<'i>,
    pub element_data: E,
    pub jumps: Vec<AddressRange>,
    pub hereditary_jumps: Vec<AddressRange>,
    pub child_counter: ChildCounter,
    pub stack_directive: StackDirective,
}

impl<'i, E: ElementData> StackItem<'i, E> {
    #[inline]
    #[must_use]
    pub fn new(local_name: LocalName<'i>) -> Self {
        StackItem {
            local_name,
            element_data: E::new(),
            jumps: Vec::default(),
            hereditary_jumps: Vec::default(),
            child_counter: Default::default(),
            stack_directive: StackDirective::Push,
        }
    }

    #[inline]
    pub fn into_owned(self) -> StackItem<'static, E> {
        StackItem {
            local_name: self.local_name.into_owned(),
            element_data: self.element_data,
            jumps: self.jumps,
            hereditary_jumps: self.hereditary_jumps,
            child_counter: self.child_counter,
            stack_directive: self.stack_directive,
        }
    }
}

pub(crate) struct Stack<E: ElementData> {
    /// A counter for root elements
    root_child_counter: ChildCounter,
    /// A typed counter for all elements on all frames. This is optional to indicate if types are actually being counted.
    typed_child_counters: Option<TypedChildCounterMap>,
    items: LimitedVec<StackItem<'static, E>>,
    /// Per-name open-item counts so `pop_up_to` can reject a stray end tag in O(1).
    open_name_counts: HashMap<LocalName<'static>, usize>,
    /// Distinct hereditary-jump ranges from open items, with the shallowest depth that introduced each.
    active_hereditary_jumps: Vec<(AddressRange, usize)>,
}

impl<E: ElementData> Stack<E> {
    #[must_use]
    #[inline]
    pub fn new(memory_limiter: SharedMemoryLimiter, enable_nth_of_type: bool) -> Self {
        Self {
            root_child_counter: Default::default(),
            typed_child_counters: enable_nth_of_type.then(TypedChildCounterMap::new),
            items: LimitedVec::new(memory_limiter),
            open_name_counts: HashMap::new(),
            active_hereditary_jumps: Vec::new(),
        }
    }

    /// Adds a child to child counters. Called before pushing the element to the stack.
    pub fn add_child(&mut self, name: &LocalName<'_>) {
        match self.items.last_mut() {
            Some(last) => &mut last.child_counter,
            None => &mut self.root_child_counter,
        }
        .inc();

        if let Some(counters) = &mut self.typed_child_counters {
            counters.add_child(name, self.items.len());
        }
    }

    #[must_use]
    pub fn build_state<'a, 'i>(&'a self, name: &LocalName<'i>) -> SelectorState<'i>
    where
        'a: 'i, // 'a outlives 'i, required to downcast 'a lifetimes into 'i
    {
        let cumulative = self
            .items
            .last()
            .map_or(&self.root_child_counter, |last| &last.child_counter);
        SelectorState {
            cumulative,
            typed: self
                .typed_child_counters
                .as_ref()
                .and_then(|f| f.get(name, self.items.len())),
        }
    }

    #[inline]
    #[must_use]
    pub fn get_stack_directive(
        item: &StackItem<'_, E>,
        ns: Namespace,
        enable_esi_tags: bool,
    ) -> StackDirective {
        if ns == Namespace::Html {
            if is_void_element(&item.local_name, enable_esi_tags) {
                StackDirective::PopImmediately
            } else {
                StackDirective::Push
            }
        } else {
            StackDirective::PushIfNotSelfClosing
        }
    }

    pub fn pop_up_to(
        &mut self,
        local_name: LocalName<'_>,
        mut popped_element_data_handler: impl FnMut(E),
    ) {
        if !self.open_name_counts.contains_key(&local_name) {
            return;
        }
        let pop_to_index = self
            .items
            .iter()
            .rposition(|item| item.local_name == local_name);
        if let Some(index) = pop_to_index {
            if let Some(c) = self.typed_child_counters.as_mut() {
                c.pop_to(index);
            }
            self.active_hereditary_jumps.retain(|(_, d)| *d < index);
            for item in self.items.drain(index..) {
                if let RawEntryMut::Occupied(mut e) = self
                    .open_name_counts
                    .raw_entry_mut()
                    .from_key(&item.local_name)
                {
                    *e.get_mut() -= 1;
                    if *e.get() == 0 {
                        e.remove();
                    }
                }
                popped_element_data_handler(item.element_data);
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn active_hereditary_jumps(&self) -> &[(AddressRange, usize)] {
        &self.active_hereditary_jumps
    }

    #[inline]
    #[must_use]
    pub fn items(&self) -> &[StackItem<'_, E>] {
        &self.items
    }

    #[inline]
    pub fn current_element_data_mut(&mut self) -> Option<&mut E> {
        self.items.last_mut().map(|i| &mut i.element_data)
    }

    #[inline]
    pub fn push_item(
        &mut self,
        item: StackItem<'static, E>,
    ) -> Result<(), MemoryLimitExceededError> {
        let depth = self.items.len();
        self.items.push(item)?;
        let item = self.items.last().expect("just pushed");
        *self
            .open_name_counts
            .entry(item.local_name.clone())
            .or_default() += 1;
        for r in &item.hereditary_jumps {
            if !self.active_hereditary_jumps.iter().any(|(a, _)| a == r) {
                self.active_hereditary_jumps.push((r.clone(), depth));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::SharedMemoryLimiter;
    use crate::selectors_vm::DenseHashSet;
    use encoding_rs::UTF_8;

    #[derive(Default)]
    struct TestElementData(usize);

    impl ElementData for TestElementData {
        fn matched_ids_mut(&mut self) -> &mut DenseHashSet {
            unreachable!();
        }
        fn new() -> Self {
            Self::default()
        }
    }

    fn local_name(name: &'static str) -> LocalName<'static> {
        LocalName::from_str_without_replacements(name, UTF_8).unwrap()
    }

    fn item(name: &'static str, data: usize) -> StackItem<'static, TestElementData> {
        let mut item = StackItem::new(local_name(name));

        item.element_data = TestElementData(data);

        item
    }

    fn active_hj(stack: &Stack<TestElementData>) -> Vec<AddressRange> {
        stack
            .active_hereditary_jumps()
            .iter()
            .map(|(r, _)| r.clone())
            .collect()
    }

    #[test]
    fn active_hereditary_jumps_dedup_and_prune() {
        let mut stack = Stack::new(SharedMemoryLimiter::new(2048), false);

        stack.push_item(item("a", 0)).unwrap();

        let mut b = item("b", 1);
        b.hereditary_jumps.push(0..1);
        stack.push_item(b).unwrap();
        assert_eq!(active_hj(&stack), vec![0..1]);

        let mut c = item("c", 2);
        c.hereditary_jumps.push(0..1);
        c.hereditary_jumps.push(2..4);
        stack.push_item(c).unwrap();
        assert_eq!(active_hj(&stack), vec![0..1, 2..4]);

        stack.push_item(item("d", 3)).unwrap();
        assert_eq!(active_hj(&stack), vec![0..1, 2..4]);

        stack.pop_up_to(local_name("c"), |_| {});
        assert_eq!(active_hj(&stack), vec![0..1]);

        stack.pop_up_to(local_name("a"), |_| {});
        assert!(active_hj(&stack).is_empty());
    }

    #[test]
    fn open_name_counts_track_push_and_drain() {
        let mut stack = Stack::new(SharedMemoryLimiter::new(2048), false);

        stack.push_item(item("a", 0)).unwrap();
        stack.push_item(item("b", 1)).unwrap();
        stack.push_item(item("a", 2)).unwrap();

        stack.pop_up_to(local_name("c"), |_| unreachable!("should not pop"));
        assert_eq!(stack.items().len(), 3);

        let mut popped = Vec::new();
        stack.pop_up_to(local_name("a"), |d| popped.push(d.0));
        assert_eq!(popped, vec![2]);
        assert_eq!(stack.items().len(), 2);

        stack.pop_up_to(local_name("a"), |d| popped.push(d.0));
        assert_eq!(popped, vec![2, 0, 1]);
        assert!(stack.items().is_empty());

        stack.pop_up_to(local_name("a"), |_| unreachable!("stack is empty"));
    }

    #[test]
    fn pop_up_to() {
        macro_rules! assert_pop_result {
            ($up_to:expr, $expected_unmatched:expr, $expected_items:expr) => {{
                let mut stack = Stack::new(SharedMemoryLimiter::new(2048), false);

                stack.push_item(item("html", 0)).unwrap();
                stack.push_item(item("body", 1)).unwrap();
                stack.push_item(item("div", 2)).unwrap();
                stack.push_item(item("div", 3)).unwrap();
                stack.push_item(item("span", 4)).unwrap();

                let mut unmatched = Vec::default();

                stack.pop_up_to(local_name($up_to), |d| {
                    unmatched.push(d.0);
                });

                assert_eq!(unmatched, $expected_unmatched);

                assert_eq!(
                    stack
                        .items()
                        .iter()
                        .map(|i| i.local_name.clone())
                        .collect::<Vec<_>>(),
                    $expected_items
                        .iter()
                        .map(|&i| local_name(i))
                        .collect::<Vec<_>>()
                );
            }};
        }

        assert_pop_result!("span", vec![4], ["html", "body", "div", "div"]);
        assert_pop_result!("div", vec![3, 4], ["html", "body", "div"]);
        assert_pop_result!("body", vec![1, 2, 3, 4], ["html"]);
        assert_pop_result!("html", vec![0, 1, 2, 3, 4], []);

        let empty: Vec<usize> = Vec::default();

        assert_pop_result!("table", empty, ["html", "body", "div", "div", "span"]);
    }

    #[test]
    fn pop_up_to_on_empty_stack() {
        let mut stack = Stack::new(SharedMemoryLimiter::new(2048), false);
        let mut handler_called = false;

        stack.pop_up_to(local_name("div"), |_: TestElementData| {
            handler_called = true;
        });

        assert!(!handler_called);
        assert_eq!(stack.items().len(), 0);
    }
}
