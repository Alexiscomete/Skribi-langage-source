use std::collections::HashMap;

use log::debug;
use miette::{Context, IntoDiagnostic};

use crate::ast::visitors::code_generator::{
    container::GeneratorContainer,
    typing::{SimplePointerValue, SimpleValue, Variable},
};

pub struct Store<'ctx, 'container> {
    container: &'container GeneratorContainer<'ctx>,
    var_values: HashMap<usize, Variable<'ctx>>,
}

impl<'ctx, 'container> Store<'ctx, 'container> {
    pub fn new(container: &'container GeneratorContainer<'ctx>) -> Store<'ctx, 'container> {
        Store {
            container,
            var_values: Default::default(),
        }
    }

    pub fn add_variable(
        &mut self,
        variable_id: usize,
        content: SimpleValue<'ctx>,
        constant: bool,
    ) -> Result<SimpleValue<'ctx>, miette::Error> {
        if constant {
            self.var_values
                .insert(variable_id, Variable::SimpleValue(content.clone()));

            return Ok(content);
        }

        debug!("Declaring {:?}", content);

        let content_type = content.typed;

        let allocated = self
            .container
            .builder
            .build_alloca(content_type, "var")
            .into_diagnostic()?;

        self.container
            .builder
            .build_store(allocated, content.value)
            .into_diagnostic()?;

        self.var_values.insert(
            variable_id,
            Variable::PointerValue(SimplePointerValue {
                value: allocated,
                pointed_type: content_type,
            }),
        );

        Ok(content)
    }

    pub fn get_variable(&mut self, variable_id: usize) -> Result<SimpleValue<'ctx>, miette::Error> {
        let stored = self
            .var_values
            .get(&variable_id)
            .wrap_err("Declared variable was removed from AST")?;

        debug!("Stored is {:?}", stored);

        match stored {
            Variable::SimpleValue(v) => Ok(v.clone()),
            Variable::PointerValue(p) => Ok(SimpleValue {
                value: self
                    .container
                    .builder
                    .build_load(p.pointed_type, p.value, "var")
                    .into_diagnostic()?,
                typed: p.pointed_type,
            }),
        }
    }

    // - get function
}
