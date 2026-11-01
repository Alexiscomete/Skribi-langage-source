use miette::Result;

use crate::{
    ast::visitors::code_generator::{container::GeneratorContainer, typing::GeneralValue},
    bindings::Bindings,
};

pub struct StdLibBinder<'ctx, 'container> {
    container: &'container GeneratorContainer<'ctx>,
}

impl<'ctx, 'container> StdLibBinder<'ctx, 'container> {
    pub fn new(container: &'container GeneratorContainer<'ctx>) -> StdLibBinder<'ctx, 'container> {
        StdLibBinder { container }
    }

    pub fn get_stdlib_call(
        &self,
        name: Bindings,
        args: GeneralValue<'ctx>,
    ) -> Result<GeneralValue<'ctx>> {
        match name {
            Bindings::Exit => {
                let arg = args.map_or_else(|| self.container.const_int(42).into(), |v| v.value);
                self.container.build_exit_call(arg)?;
                Ok(None)
            }
            Bindings::Println => self.container.build_print_call(args),
        }
    }
}
