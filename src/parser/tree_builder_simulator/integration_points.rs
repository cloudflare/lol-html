use crate::html::{LocalNameHash, Tag};
use crate::memory::{LimitedVec, MemoryLimitExceededError, SharedMemoryLimiter};
use hashbrown::HashMap;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use std::num::NonZeroUsize;

/// An integration opener or an HTML descendant within its namespace frame.
pub(super) struct IntegrationPoint {
    pub(super) tag_name: LocalNameHash,
    pub(super) namespace_depth: usize,
    name_start: usize,
    name_length: usize,
    previous_with_hash: Option<NonZeroUsize>,
    previous_special: Option<NonZeroUsize>,
    previous_scope_boundary: Option<NonZeroUsize>,
}

/// Explicit ancestry is needed only while an integration point is open.
pub(super) struct IntegrationPointStack {
    memory_limiter: SharedMemoryLimiter,
    state: Option<Box<IntegrationPointState>>,
}

struct IntegrationPointState {
    elements: LimitedVec<IntegrationPoint>,
    names: LimitedVec<u8>,
    last_by_hash: HashMap<(u64, usize), NonZeroUsize, RandomState>,
    last_special: Option<NonZeroUsize>,
    last_scope_boundary: Option<NonZeroUsize>,
}

fn is_scope_boundary(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(tag_name, [Table, Template])
        || ["applet", "caption", "td", "th", "marquee", "object"]
            .iter()
            .any(|name| tag_name == LocalNameHash::from(*name))
}

fn is_special(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(
        tag_name,
        [
            Area, Base, Basefont, Bgsound, Blockquote, Body, Br, Center, Col, Dd, Div, Dl, Dt,
            Embed, Frameset, H1, H2, H3, H4, H5, H6, Head, Hr, Iframe, Img, Input, Keygen, Li,
            Link, Listing, Menu, Meta, Noembed, Noframes, Noscript, Ol, P, Param, Plaintext, Pre,
            Script, Select, Source, Style, Table, Template, Textarea, Title, Track, Ul, Wbr, Xmp
        ]
    ) || [
        "address",
        "applet",
        "article",
        "aside",
        "button",
        "caption",
        "colgroup",
        "details",
        "dir",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "form",
        "frame",
        "header",
        "hgroup",
        "html",
        "main",
        "marquee",
        "nav",
        "object",
        "search",
        "section",
        "summary",
        "tbody",
        "td",
        "tfoot",
        "th",
        "thead",
        "tr",
    ]
    .iter()
    .any(|name| tag_name == LocalNameHash::from(*name))
}

fn closes_special_descendants(tag_name: LocalNameHash) -> bool {
    tag_is_one_of!(
        tag_name,
        [
            Blockquote, Center, Dd, Div, Dl, Dt, H1, H2, H3, H4, H5, H6, Li, Listing, Menu, Ol, P,
            Pre, Ul
        ]
    ) || [
        "address",
        "article",
        "aside",
        "button",
        "details",
        "dialog",
        "dir",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "header",
        "hgroup",
        "main",
        "nav",
        "search",
        "section",
        "summary",
    ]
    .iter()
    .any(|name| tag_name == LocalNameHash::from(*name))
}

fn hash_name(hasher: &RandomState, tag_name: LocalNameHash, name: &[u8]) -> u64 {
    let mut hasher = hasher.build_hasher();

    tag_name.hash(&mut hasher);
    if tag_name.is_empty() {
        name.iter()
            .for_each(|byte| byte.to_ascii_lowercase().hash(&mut hasher));
    }

    hasher.finish()
}

impl IntegrationPointStack {
    pub(super) fn new(memory_limiter: SharedMemoryLimiter) -> Self {
        Self {
            memory_limiter,
            state: None,
        }
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.state.as_ref().map_or(0, |state| state.elements.len())
    }

    #[inline]
    pub(super) fn last(&self) -> Option<&IntegrationPoint> {
        self.state.as_ref()?.elements.last()
    }

    pub(super) fn push(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
        namespace_depth: usize,
    ) -> Result<(), MemoryLimitExceededError> {
        if self.state.is_none() {
            self.memory_limiter
                .increase_usage(size_of::<IntegrationPointState>())?;
            self.state = Some(Box::new(IntegrationPointState {
                elements: LimitedVec::new(self.memory_limiter.clone()),
                names: LimitedVec::new(self.memory_limiter.clone()),
                last_by_hash: HashMap::with_hasher(RandomState::new()),
                last_special: None,
                last_scope_boundary: None,
            }));
        }

        self.state
            .as_mut()
            .expect("integration-point state not initialized")
            .push(tag_name, name, namespace_depth, &self.memory_limiter)
    }

    pub(super) fn close(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
        namespace_depth: usize,
    ) -> bool {
        let Some(state) = &mut self.state else {
            return false;
        };
        let Some(index) = state.find(tag_name, name, namespace_depth) else {
            return false;
        };
        let integration_opener =
            index == 0 || state.elements[index - 1].namespace_depth != namespace_depth;

        // HTML end-tag matching cannot reach a foreign opener through unclosed HTML descendants.
        if integration_opener && index + 1 != state.elements.len() {
            return false;
        }

        // Generic HTML end tags cannot consume an ancestor across a special element.
        if state
            .last_scope_boundary
            .is_some_and(|boundary| boundary.get() - 1 > index)
            || (!closes_special_descendants(tag_name)
                && state
                    .last_special
                    .is_some_and(|special| special.get() - 1 > index))
        {
            return false;
        }

        state.pop_from(index);
        !state
            .elements
            .last()
            .is_some_and(|entry| entry.namespace_depth == namespace_depth)
    }
}

impl Drop for IntegrationPointStack {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            self.memory_limiter.decrease_usage(
                size_of::<IntegrationPointState>() + state.last_by_hash.allocation_size(),
            );
        }
    }
}

impl IntegrationPointState {
    fn push(
        &mut self,
        tag_name: LocalNameHash,
        name: &[u8],
        namespace_depth: usize,
        memory_limiter: &SharedMemoryLimiter,
    ) -> Result<(), MemoryLimitExceededError> {
        let hash = hash_name(self.last_by_hash.hasher(), tag_name, name);
        let key = (hash, namespace_depth);
        let previous_with_hash = self.last_by_hash.get(&key).copied();
        let integration_opener = self
            .elements
            .last()
            .is_none_or(|entry| entry.namespace_depth != namespace_depth);

        if previous_with_hash.is_none() {
            let old_allocation = self.last_by_hash.allocation_size();

            self.last_by_hash
                .try_reserve(1)
                .map_err(|_| MemoryLimitExceededError)?;
            memory_limiter.increase_usage(self.last_by_hash.allocation_size() - old_allocation)?;
        }

        let name_start = self.names.len();
        let name_length = match tag_name.is_empty() {
            true => name.len(),
            false => 0,
        };

        name.iter()
            .take(name_length)
            .try_for_each(|byte| self.names.push(byte.to_ascii_lowercase()))?;
        self.elements.push(IntegrationPoint {
            tag_name,
            namespace_depth,
            name_start,
            name_length,
            previous_with_hash,
            previous_special: self.last_special,
            previous_scope_boundary: self.last_scope_boundary,
        })?;

        let index = NonZeroUsize::new(self.elements.len()).expect("element stack not empty");

        self.last_by_hash.insert(key, index);
        if integration_opener || is_special(tag_name) {
            self.last_special = Some(index);
        }
        if integration_opener || is_scope_boundary(tag_name) {
            self.last_scope_boundary = Some(index);
        }
        Ok(())
    }

    fn find(&self, tag_name: LocalNameHash, name: &[u8], namespace_depth: usize) -> Option<usize> {
        let hash = hash_name(self.last_by_hash.hasher(), tag_name, name);
        let mut candidate = self.last_by_hash.get(&(hash, namespace_depth)).copied();
        let names = &*self.names;

        while let Some(index) = candidate {
            let index = index.get() - 1;
            let entry = &self.elements[index];
            let same_name = entry.tag_name == tag_name
                && (!tag_name.is_empty()
                    || names[entry.name_start..entry.name_start + entry.name_length]
                        .eq_ignore_ascii_case(name));

            if same_name {
                return Some(index);
            }

            // Only randomized hash collisions need a search; absent names do not scan ancestry.
            candidate = entry.previous_with_hash;
        }

        None
    }

    fn pop_from(&mut self, index: usize) {
        let name_start = self.elements[index].name_start;
        let hasher = self.last_by_hash.hasher().clone();
        let names = &*self.names;

        for entry in self.elements.drain(index..).rev() {
            let name = &names[entry.name_start..entry.name_start + entry.name_length];
            let hash = hash_name(&hasher, entry.tag_name, name);
            let key = (hash, entry.namespace_depth);

            match entry.previous_with_hash {
                Some(previous) => {
                    self.last_by_hash.insert(key, previous);
                }
                None => {
                    self.last_by_hash.remove(&key);
                }
            }
            self.last_special = entry.previous_special;
            self.last_scope_boundary = entry.previous_scope_boundary;
        }

        self.names.drain(name_start..);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Index storage, literal names, and element capacity must all be charged and released.
    #[test]
    fn test_integration_points_all_storage_accounted() {
        let limiter = SharedMemoryLimiter::new(usize::MAX);
        let mut stack = IntegrationPointStack::new(limiter.clone());

        stack.push("desc".into(), b"desc", 3).unwrap();
        for index in 0..64 {
            let name = format!("custom-element-{index}");

            stack
                .push(name.as_str().into(), name.as_bytes(), 3)
                .unwrap();
        }

        let state = stack.state.as_ref().unwrap();
        let charged = limiter.current_usage();
        let records = state.elements.len() * size_of::<IntegrationPoint>();
        let literals = (0..64)
            .map(|index| format!("custom-element-{index}").len())
            .sum::<usize>();

        assert!(
            charged
                >= size_of::<IntegrationPointState>()
                    + records
                    + literals
                    + state.last_by_hash.allocation_size(),
            "all retained metadata and index allocations must be charged"
        );
        assert!(
            !stack.close("custom-element-0".into(), b"custom-element-0", 3),
            "closing the HTML ancestor must preserve the foreign opener"
        );
        assert!(
            stack.close("desc".into(), b"desc", 3),
            "the foreign opener must then close"
        );
        assert_eq!(stack.len(), 0, "the ancestor close removes all records");
        assert_eq!(
            limiter.current_usage(),
            charged,
            "reusable capacity must remain charged"
        );
        assert!(
            stack.state.as_ref().unwrap().last_by_hash.is_empty(),
            "closed names must leave the index"
        );
        assert_eq!(
            stack.state.as_ref().unwrap().names.len(),
            0,
            "closed literal names must be discarded"
        );
        drop(stack);
        assert_eq!(
            limiter.current_usage(),
            0,
            "drop must release records, names, index, and state"
        );
    }

    /// Popping to an ancestor restores the previous occurrence of every name in the index.
    #[test]
    fn test_integration_points_ancestor_restores_name_index() {
        let mut stack = IntegrationPointStack::new(SharedMemoryLimiter::new(usize::MAX));

        for name in ["desc", "span", "desc", "span", "desc"] {
            stack.push(name.into(), name.as_bytes(), 3).unwrap();
        }

        assert!(
            !stack.close("span".into(), b"span", 3),
            "the inner ancestor must not close the opener"
        );
        assert_eq!(
            stack.len(),
            3,
            "both the inner span and its desc must close"
        );
        assert!(
            !stack.close("span".into(), b"span", 3),
            "the outer span must still be indexed"
        );
        assert_eq!(
            stack.len(),
            1,
            "the outer descendant records must close together"
        );
        assert!(
            stack.close("desc".into(), b"desc", 3),
            "the final desc must close the integration point"
        );
    }

    /// Literal names must compare exactly and case-insensitively rather than share an empty hash.
    #[test]
    fn test_integration_points_literal_name_identity() {
        let mut stack = IntegrationPointStack::new(SharedMemoryLimiter::new(usize::MAX));

        for name in ["annotation-xml", "first-custom", "annotation-xml"] {
            stack.push(name.into(), name.as_bytes(), 3).unwrap();
        }

        assert!(
            !stack.close("other-custom".into(), b"other-custom", 3),
            "an absent literal name must not close anything"
        );
        assert_eq!(
            stack.len(),
            3,
            "an unmatched custom end tag must preserve ancestry"
        );
        assert!(
            !stack.close("first-custom".into(), b"FIRST-CUSTOM", 3),
            "a real ancestor close preserves the opener"
        );
        assert_eq!(
            stack.len(),
            1,
            "a case-insensitive ancestor close removes its descendant"
        );
        assert!(
            stack.close("annotation-xml".into(), b"ANNOTATION-XML", 3),
            "the real opener must close"
        );
    }

    /// An inner namespace frame must not consume an indexed name from a suspended outer frame.
    #[test]
    fn test_integration_points_namespace_depth_identity() {
        let mut stack = IntegrationPointStack::new(SharedMemoryLimiter::new(usize::MAX));

        stack.push("desc".into(), b"desc", 3).unwrap();
        stack.push("span".into(), b"span", 3).unwrap();
        stack.push("desc".into(), b"desc", 5).unwrap();
        assert_eq!(
            stack
                .state
                .as_ref()
                .unwrap()
                .find("span".into(), b"span", 5),
            None,
            "the name index must not return a suspended outer ancestor"
        );
        assert!(
            !stack.close("span".into(), b"span", 5),
            "an inner close must not consume an outer ancestor"
        );
        assert_eq!(
            stack.len(),
            3,
            "a suspended outer name is not an inner match"
        );
        assert!(
            stack.close("desc".into(), b"desc", 5),
            "the inner opener must close independently"
        );
        assert!(
            !stack.close("span".into(), b"span", 3),
            "the outer ancestor must still be indexed"
        );
        assert!(
            stack.close("desc".into(), b"desc", 3),
            "the outer opener must close independently"
        );
    }

    /// An explicit block close must not consume records across an HTML scope boundary.
    #[test]
    fn test_integration_points_scope_barrier() {
        let mut stack = IntegrationPointStack::new(SharedMemoryLimiter::new(usize::MAX));

        for name in ["desc", "div", "table", "desc"] {
            stack.push(name.into(), name.as_bytes(), 3).unwrap();
        }

        assert!(
            !stack.close("div".into(), b"div", 3),
            "the intervening table must block an outer div close"
        );
        assert_eq!(
            stack.len(),
            4,
            "a blocked close must not consume descendant metadata"
        );
        assert!(
            !stack.close("table".into(), b"table", 3),
            "the boundary can close without closing the point"
        );
        assert_eq!(
            stack.len(),
            2,
            "the real table close must consume only its own descendants"
        );
        assert!(
            !stack.close("div".into(), b"div", 3),
            "removing the boundary must restore the outer scope"
        );
        assert!(
            stack.close("desc".into(), b"desc", 3),
            "the real integration opener must still close"
        );
    }

    /// A forced fingerprint collision must still compare the complete literal name.
    #[test]
    fn test_integration_points_fingerprint_collision_identity() {
        let mut stack = IntegrationPointStack::new(SharedMemoryLimiter::new(usize::MAX));

        stack
            .push("first-custom".into(), b"first-custom", 3)
            .unwrap();
        stack
            .push("other-custom".into(), b"other-custom", 3)
            .unwrap();

        let state = stack.state.as_mut().unwrap();
        let first_hash = hash_name(
            state.last_by_hash.hasher(),
            "first-custom".into(),
            b"first-custom",
        );

        state.elements.as_mut_slice()[1].previous_with_hash = NonZeroUsize::new(1);
        state
            .last_by_hash
            .insert((first_hash, 3), NonZeroUsize::new(2).unwrap());
        assert_eq!(
            state.find("first-custom".into(), b"first-custom", 3),
            Some(0),
            "a collision must not match the wrong custom element"
        );
    }

    /// Literal bytes must be charged even when the element vector already has spare capacity.
    #[test]
    fn test_integration_points_literal_name_memory_limit() {
        let probe_limiter = SharedMemoryLimiter::new(usize::MAX);
        let mut probe = IntegrationPointStack::new(probe_limiter.clone());

        probe.push("desc".into(), b"desc", 3).unwrap();

        let limit = probe_limiter.current_usage() + 128;
        let mut stack = IntegrationPointStack::new(SharedMemoryLimiter::new(limit));

        stack.push("desc".into(), b"desc", 3).unwrap();

        let name = "custom-element-".repeat(16);
        let result = stack.push(name.as_str().into(), name.as_bytes(), 3);

        assert_eq!(
            result,
            Err(MemoryLimitExceededError),
            "literal names must not grow without being charged"
        );
        assert_eq!(
            stack.len(),
            1,
            "a failed literal allocation must not add an element record"
        );
    }

    /// Index growth must report the shared memory error before adding the new element record.
    #[test]
    fn test_integration_points_name_index_memory_limit() {
        let probe_limiter = SharedMemoryLimiter::new(usize::MAX);
        let mut probe = IntegrationPointStack::new(probe_limiter.clone());

        probe.push("desc".into(), b"desc", 3).unwrap();

        let initial_capacity = probe.state.as_ref().unwrap().last_by_hash.capacity();
        let limiter = SharedMemoryLimiter::new(probe_limiter.current_usage());
        let mut stack = IntegrationPointStack::new(limiter.clone());

        stack.push("desc".into(), b"desc", 3).unwrap();
        for index in 1..initial_capacity {
            let name = format!("ancestor{}", char::from(b'a' + index as u8));

            stack
                .push(name.as_str().into(), name.as_bytes(), 3)
                .unwrap();
        }

        let result = stack.push("span".into(), b"span", 3);

        assert_eq!(
            result,
            Err(MemoryLimitExceededError),
            "the name index must not grow without being charged"
        );
        assert_eq!(
            stack.len(),
            initial_capacity,
            "failed index growth must preserve existing ancestry"
        );
        drop(stack);
        assert_eq!(
            limiter.current_usage(),
            0,
            "drop must release the entire successful index allocation even after a limit error"
        );
    }
}
