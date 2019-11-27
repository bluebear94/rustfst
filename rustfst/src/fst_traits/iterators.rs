use crate::arc::Arc;
use crate::StateId;
use crate::fst_traits::CoreFst;
use failure::Fallible;


/// Trait to iterate over the states of a wFST.
pub trait StateIterator<'a> {
    /// Iterator used to iterate over the `state_id` of the states of an FST.
    type Iter: Iterator<Item = StateId> + Clone;

    /// Creates an iterator over the `state_id` of the states of an FST.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// for state_id in fst.states_iter() {
    ///     println!("State ID : {:?}", state_id);
    /// }
    ///
    /// let states : Vec<_> = fst.states_iter().collect();
    /// assert_eq!(states, vec![s1, s2]);
    /// ```
    fn states_iter(&'a self) -> Self::Iter;
}

/// Trait to iterate over the outgoing arcs of a particular state in a wFST
pub trait ArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    /// Iterator used to iterate over the arcs leaving a state of an FST.
    type Iter: Iterator<Item = &'a Arc<Self::W>> + Clone;

    fn arcs_iter(&'a self, state_id: StateId) -> Fallible<Self::Iter>;
    unsafe fn arcs_iter_unchecked(&'a self, state_id: StateId) -> Self::Iter;
}




/// Trait to iterator over a wFST in order to modify its arcs without changing the number of states or the number of arcs
pub trait FstIterator: CoreFst
{
    type StateIndex: Copy;
    type ArcIndex: Copy;

    /// Iterator used to iterate over the arcs leaving a state of an FST.
    type ArcIter: Iterator<Item = Self::ArcIndex> + Clone;
    /// Iterator used to iterate over states of an FST.
    type StateIter: Iterator<Item = Self::StateIndex> + Clone;

    fn states_index_iter(&self) -> Self::StateIter;
    fn arcs_index_iter(&self, state: Self::StateIndex) -> Fallible<Self::ArcIter>;
    /// Get an arc from its state index and its arc index, generated by the two iterator methods
    fn get_arc<'a>(&'a self, state: Self::StateIndex, arc: Self::ArcIndex) -> Fallible<&'a Arc<Self::W>>;
}

pub trait FstIteratorMut: FstIterator {
    /// Modify in place an arc from the state index and the arc index
    fn modify_arc<F>(&mut self, state: Self::StateIndex, arc: Self::ArcIndex, modify: F) -> Fallible<()> 
            where F: Fn(&mut Arc<Self::W>) -> Fallible<()>;
}

