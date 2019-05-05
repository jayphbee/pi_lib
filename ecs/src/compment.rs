
use std::{
    sync::Arc,
    marker::PhantomData,
    intrinsics::type_name,
    ops::{Deref, DerefMut},
};


use pointer::cell::{Ref, RefMut, TrustCell};
use slab::{Slab, SlabIter, SlabIterMut};
use map::{Map, vecmap::VecMap};


use system::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};
use entity::CellEntity;

pub trait SingleCase: Notify {


}
pub trait MultiCase: Notify {
    //fn create(&mut self);
}

pub type CellMultiCase<E, C> = TrustCell<MultiCaseImpl<E, C>>;
// TODO 以后用宏生成
impl<E, C> Notify for CellMultiCase<E, C> {
    fn add_create(&self, listener: CreateFn) {
        self.borrow_mut().notify.create.push_back(listener)
    }
    fn add_delete(&self, listener: DeleteFn) {
        self.borrow_mut().notify.delete.push_back(listener)
    }
    fn add_modify(&self, listener: ModifyFn) {
        self.borrow_mut().notify.modify.push_back(listener)
    }
    fn create_event(&self, id: usize) {
        self.borrow().notify.create_event(id);
    }
    fn delete_event(&self, id: usize) {
        self.borrow().notify.delete_event(id);
    }
    fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        self.borrow().notify.modify_event(id, field, index);
    }
    fn remove_create(&self, listener: &CreateFn) {
        self.borrow_mut().notify.create.delete(listener);
    }
    fn remove_delete(&self, listener: &DeleteFn) {
        self.borrow_mut().notify.delete.delete(listener);
    }
    fn remove_modify(&self, listener: &ModifyFn) {
        self.borrow_mut().notify.modify.delete(listener);
    }
}
impl<E, C> MultiCase for CellMultiCase<E, C> {
}

#[derive(Default)]
pub struct MultiCaseImpl<E, C> {
    map: VecMap<C>,
    notify: NotifyImpl,
    entity: Arc<CellEntity>,
    bit_index: usize,
    marker: PhantomData<E>,
}

impl<E, C> MultiCaseImpl<E, C> {
    pub fn new(entity: Arc<CellEntity>, bit_index: usize) -> TrustCell<Self>{
        TrustCell::new(MultiCaseImpl{
            map: VecMap::default(),
            notify: NotifyImpl::default(),
            entity: entity,
            bit_index: bit_index,
            marker: PhantomData,
        })
    }
}