use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use std::borrow::Borrow;

#[derive(Debug)]
pub struct Shared<T: ?Sized> {
    v: Rc<RefCell<T>>
}

impl <T> Shared<T> {
    pub fn new(t: T) -> Self {
        Shared{
            v: Rc::new(RefCell::new(t))
        }
    }

    pub fn get_mut_ref(&self) -> RefMut<'_, T> {
        let s: &RefCell<T> = self.v.borrow();
        s.borrow_mut()
    }

    pub fn get_ref(&self) -> Ref<'_, T> {
        let s: &RefCell<T> = self.v.borrow();
        s.borrow()
    }

    pub fn into_inner(self) -> T {
        let refcell = Rc::try_unwrap(self.v);
        match refcell {
            Ok(refcell) => refcell.into_inner(),
            Err(_) => panic!("Failed to unwrap Reference"),
        }
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            v: self.v.clone()
        }
    }
}