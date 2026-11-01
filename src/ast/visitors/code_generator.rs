use log::{debug, trace};
use miette::{Context, IntoDiagnostic, Result};

use crate::ast::nodes::binop::BinopEnum;
use crate::ast::nodes::{StateBinded, into_str};
use crate::ast::visitors::code_generator::container::GeneratorContainer;
use crate::ast::visitors::code_generator::store::Store;
use crate::ast::visitors::code_generator::typing::{GeneralValue, SimpleIntValue, to_fn_type};
use crate::ast::{nodes::FileTreeRoot, visitors::AstMutVisitor};
use crate::interner::get_interner;

pub mod builtins;
pub mod container;
pub mod stdlib_bindings;
pub mod store;
pub mod typing;

pub struct CodeGenerator<'ctx, 'container> {
    container: &'container GeneratorContainer<'ctx>,
    main_empty: bool,
    store: Store<'ctx, 'container>,
}

impl<'ctx, 'container> CodeGenerator<'ctx, 'container> {
    pub fn compile(root: &FileTreeRoot<StateBinded>, name: &str, folder: &str) -> Result<()> {
        GeneratorContainer::using(name, |container| {
            let mut compiler = CodeGenerator {
                container,
                main_empty: true,
                store: Store::new(container),
            };

            compiler.visit_file_tree_root(root)?;

            if compiler.main_empty {
                let arg = compiler.container.int().const_int(0, false);
                compiler.container.build_exit_call(arg.into())?;
            }

            compiler.container.save(name, folder)?;

            Ok(())
        })
    }
}

impl<'ctx, 'container> AstMutVisitor<'_, GeneralValue<'ctx>, miette::Report, StateBinded>
    for CodeGenerator<'ctx, 'container>
{
    fn default_t(_: super::DefaultCause) -> miette::Result<GeneralValue<'static>, miette::Error> {
        Ok(None)
    }

    fn visit_function_call(
        &mut self,
        function_call: &crate::ast::nodes::calls::functions::FunctionCall,
    ) -> Result<GeneralValue<'ctx>, miette::Error> {
        self.main_empty = false;

        trace!("Compiling a function call");

        // Step 1, compile the arguments
        // TODO: support multiple arguments

        let args = self.default_function_call(function_call)?;

        // Step 2, call
        self.main_empty = false;
        self.store.get_function(
            args,
            function_call
                .declaration
                .clone()
                .wrap_err("Unbinded call")?,
        )
    }

    fn visit_number(
        &mut self,
        number: &crate::ast::nodes::numbers::Number,
    ) -> Result<GeneralValue<'ctx>, miette::Error> {
        trace!("Compiling a number");

        let interner = get_interner()?;
        let name = into_str(&interner, number.content);
        let value = name.parse().into_diagnostic()?;

        debug!("Found number {}", value);

        let result = self.container.context.i32_type().const_int(value, false);
        Ok(Some(
            SimpleIntValue {
                value: result,
                typed: self.container.context.i32_type(),
            }
            .into(),
        ))
    }

    fn visit_binop(
        &mut self,
        binop: &crate::ast::nodes::binop::Binop,
    ) -> Result<GeneralValue<'ctx>, miette::Error> {
        trace!("Compiling a binop");

        let base_left: SimpleIntValue = self
            .visit_expression(&binop.left)?
            .wrap_err("No valid left expression")?
            .try_into()?;
        let base_right: SimpleIntValue = self
            .visit_expression(&binop.right)?
            .wrap_err("No valid right expression")?
            .try_into()?;

        let left = base_left.value;
        let right = base_right.value;

        let result = match binop.binop {
            BinopEnum::Add => self.container.builder.build_int_add(left, right, "add"),
            BinopEnum::Substract => self.container.builder.build_int_sub(left, right, "sub"),
            BinopEnum::Multiply => self.container.builder.build_int_mul(left, right, "mul"),
            BinopEnum::Divide => self
                .container
                .builder
                .build_int_signed_div(left, right, "div"),
        }
        .into_diagnostic()?;

        Ok(Some(
            SimpleIntValue {
                value: result,
                typed: base_left.typed,
            }
            .into(),
        ))
    }

    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &crate::ast::nodes::declarations::variable::VariableDeclarationRef,
    ) -> Result<GeneralValue<'ctx>, miette::Error> {
        let content = self
            .default_variable_declaration(variable_declaration)?
            .wrap_err("No valid content")?;

        Ok(Some(self.store.add_variable(
            variable_declaration.read(|v| v.id),
            content,
            variable_declaration.read(|v| v.constant),
        )?))
    }

    fn visit_variable_usage(
        &mut self,
        variable_usage: &crate::ast::nodes::calls::variable::VariableUsage,
    ) -> Result<GeneralValue<'ctx>, miette::Error> {
        trace!("Compiling a variable usage");

        Ok(Some(
            self.store.get_variable(
                variable_usage
                    .declaration
                    .clone()
                    .wrap_err("Undeclared declared variable")?
                    .read(|v| v.id),
            )?,
        ))
    }
}
