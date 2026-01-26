use crate::interpreter::{Context, Navigate};

pub struct Descend;

impl Navigate for Descend {
    fn apply(&self, ctx: &mut Context) {
        ctx.descend();
    }
}

pub struct Ascend;

impl Navigate for Ascend {
    fn apply(&self, ctx: &mut Context) {
        ctx.ascend();
    }
}
