use std::hash::Hash;

use failure::Fallible;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::matchers::MatchType;
use crate::algorithms::matchers::Matcher;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, StateId};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct ComposeStateTuple<FS> {
    fs: FS,
    s1: StateId,
    s2: StateId,
}

#[derive(Clone, PartialEq)]
struct ComposeFstImpl<
    'matcher,
    'fst,
    F1: Fst + 'fst,
    F2: Fst<W = F1::W> + 'fst,
    CF: ComposeFilter<'matcher, 'fst, F1, F2>,
> {
    fst1: &'fst F1,
    fst2: &'fst F2,
    matcher1: Rc<RefCell<CF::M1>>,
    matcher2: Rc<RefCell<CF::M2>>,
    compose_filter: CF,
    cache_impl: CacheImpl<F1::W>,
    state_table: StateTable<ComposeStateTuple<CF::FS>>,
    match_type: MatchType,
}

impl<
        'iter,
        'fst: 'iter,
        F1: Fst + 'fst,
        F2: Fst<W = F1::W> + 'fst,
        CF: ComposeFilter<'iter, 'fst, F1, F2>,
    > ComposeFstImpl<'iter, 'fst, F1, F2, CF>
where
    <F1 as CoreFst>::W: 'static,
{
    fn match_input(&self, s1: StateId, s2: StateId) -> bool {
        match self.match_type {
            MatchType::MatchInput => true,
            MatchType::MatchOutput => false,
            _ => unimplemented!(),
        }
    }

    fn ordered_expand<FA: Fst, FB: Fst, M>(
        &self,
        s: StateId,
        fsta: &FA,
        sa: StateId,
        fstb: &FB,
        sb: StateId,
        mut matchera: Rc<M>,
        match_input: bool,
    ) {
        unimplemented!()
    }

    fn add_arc(
        &mut self,
        s: StateId,
        mut arc1: Arc<F1::W>,
        arc2: Arc<F1::W>,
        fs: CF::FS,
    ) -> Fallible<()> {
        let tuple = ComposeStateTuple {
            fs,
            s1: arc1.nextstate,
            s2: arc2.nextstate,
        };
        arc1.weight.times_assign(arc2.weight)?;
        self.cache_impl.push_arc(
            s,
            Arc::new(
                arc1.ilabel,
                arc2.olabel,
                arc1.weight,
                self.state_table.find_id(tuple),
            ),
        );

        Ok(())
    }

    fn match_arc<'a, 'b: 'a, F: Fst + 'b, M: Matcher<'a, 'b, F>>(
        &mut self,
        s: StateId,
        sa: StateId,
        matchera: Rc<RefCell<M>>,
        arc: &Arc<F1::W>,
        match_input: bool,
    ) -> Fallible<()> {
        let label = if match_input { arc.olabel } else { arc.ilabel };

        for arca in matchera.borrow_mut().iter(sa, label)? {
            let mut arca = arc.clone();
            let mut arcb = arc.clone();
            if match_input {
                let opt_fs = self.compose_filter.filter_arc(&mut arcb, &mut arca);
                if let Some(fs) = opt_fs {
                    self.add_arc(s, arcb, arca, fs)?;
                }
            } else {
                let opt_fs = self.compose_filter.filter_arc(&mut arca, &mut arcb);
                if let Some(fs) = opt_fs {
                    self.add_arc(s, arca, arcb, fs)?;
                }
            }
        }

        Ok(())
    }
}

impl<
        'matcher,
        'fst: 'matcher,
        F1: Fst + 'fst,
        F2: Fst<W = F1::W> + 'fst,
        CF: ComposeFilter<'matcher, 'fst, F1, F2>,
    > FstImpl for ComposeFstImpl<'matcher, 'fst, F1, F2, CF>
where
    <F1 as CoreFst>::W: 'static,
{
    type W = F1::W;

    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<Self::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        let tuple = self.state_table.find_tuple(state);
        let s1 = tuple.s1;
        let s2 = tuple.s2;
        self.compose_filter.set_state(s1, s2, &tuple.fs);
        drop(tuple);
        if self.match_input(s1, s2) {
            self.ordered_expand(
                state,
                self.fst2,
                s2,
                self.fst1,
                s1,
                Rc::clone(&self.matcher2),
                true,
            );
        } else {
            self.ordered_expand(
                state,
                self.fst1,
                s1,
                self.fst2,
                s2,
                Rc::clone(&self.matcher1),
                false,
            );
        }
        Ok(())
    }

    fn compute_start(&mut self) -> Fallible<Option<StateId>> {
        let s1 = self.fst1.start();
        if s1.is_none() {
            return Ok(None);
        }
        let s1 = s1.unwrap();
        let s2 = self.fst2.start();
        if s2.is_none() {
            return Ok(None);
        }
        let s2 = s2.unwrap();
        let fs = self.compose_filter.start();
        let tuple = ComposeStateTuple { s1, s2, fs };
        Ok(Some(self.state_table.find_id(tuple)))
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<Self::W>> {
        let tuple = self.state_table.find_tuple(state);

        let s1 = tuple.s1;
        let final1 = self.compose_filter.matcher1().borrow().final_weight(s1)?;
        if final1.is_none() {
            return Ok(None);
        }
        let mut final1 = final1.unwrap().clone();

        let s2 = tuple.s2;
        let final2 = self.compose_filter.matcher2().borrow().final_weight(s2)?;
        if final2.is_none() {
            return Ok(None);
        }
        let mut final2 = final2.unwrap().clone();

        self.compose_filter.set_state(s1, s2, &tuple.fs);
        self.compose_filter.filter_final(&mut final1, &mut final2);

        final1.times_assign(&final2)?;
        Ok(Some(final1))
    }
}
