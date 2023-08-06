use std::cell::RefMut;
use cursive::Cursive;
use crate::model::model::RootModel;
use crate::shared::Shared;

pub trait WithRootModel {
    fn get_root_model(&mut self) -> RefMut<RootModel>;
}

impl WithRootModel for Cursive {
    fn get_root_model(&mut self) -> RefMut<RootModel> {
        let state: &Shared<RootModel> = self.user_data().unwrap();
        state.get_mut_ref()
    }
}