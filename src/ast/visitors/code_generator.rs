use std::fs::create_dir_all;
use std::path::Path;

use inkwell::basic_block::BasicBlock;
use inkwell::context::Context as InkContext;
use inkwell::module::Linkage;
use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::FunctionValue;
use inkwell::{builder::Builder, module::Module};
use log::trace;
use miette::{Context, IntoDiagnostic, Result, miette};

use crate::ast::{nodes::FileTreeRoot, visitors::AstMutVisitor};
use crate::interner::INTERNER;

pub struct CodeGenerator<'ctx> {
    context: &'ctx InkContext,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
    fn goin(&self, block: BasicBlock) {
        self.builder.position_at_end(block);
    }

    /// This function is made to be a simplified way to create fonctions
    /// This is not a way to import fonctions
    fn create_function(
        &self,
        name: &str,
        return_type: BasicTypeEnum<'ctx>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<BasicBlock<'_>> {
        // TODO: add void type
        let main_function_type = return_type.fn_type(parameters_types, is_var_args);
        let main_function = self.module.add_function(name, main_function_type, linkage);
        let main_block = self.context.append_basic_block(main_function, name);
        Ok(main_block)
    }

    fn import_function(
        &self,
        name: &str,
        return_type: AnyTypeEnum<'ctx>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<FunctionValue<'_>> {
        match return_type {
            AnyTypeEnum::IntType(return_type) => {
                let main_function_type = return_type.fn_type(parameters_types, is_var_args);
                let main_function = self.module.add_function(name, main_function_type, linkage);
                Ok(main_function)
            }
            // Same code...
            AnyTypeEnum::VoidType(return_type) => {
                let main_function_type = return_type.fn_type(parameters_types, is_var_args);
                let main_function = self.module.add_function(name, main_function_type, linkage);
                Ok(main_function)
            }
            _ => Err(miette!("Type not supported for return type")),
        }
    }

    fn create_base(&self) -> Result<()> {
        // Create main function
        // TODO: add arguments
        let ret_type = self.context.i32_type();
        let function = self.create_function("main", ret_type.into(), &[], false, None)?;
        self.goin(function);
        Ok(())
    }

    fn save(&self, name: &str, folder: &str) -> Result<()> {
        let path = Path::new(folder).join(name).with_added_extension("ll");
        let parent = path.as_path().parent().context("No parent folder")?;
        create_dir_all(parent)
            .into_diagnostic()
            .context(format!("Cannot create folders for `{}`", name))?;
        let path_str = path.to_str().context("Invalid path format")?.to_owned();
        self.module
            .print_to_file(path)
            .into_diagnostic()
            .context(format!(
                "Failed to save program in HIR format file {}",
                path_str
            ))?;
        Ok(())
    }

    pub fn compile(root: &FileTreeRoot, name: &str, folder: &str) -> Result<()> {
        let context = InkContext::create();
        let module = context.create_module(name);
        let builder = context.create_builder();

        let mut compiler = CodeGenerator {
            context: &context,
            module,
            builder,
        };
        compiler.create_base()?;
        compiler.visit_file_tree_root(root)?;
        compiler.save(name, folder)?;

        Ok(())
    }
}

impl AstMutVisitor<'_, ()> for CodeGenerator<'_> {
    fn default_t(_: super::DefaultCause) -> miette::Result<(), miette::Error> {
        Ok(())
    }

    fn visit_function_call(
        &mut self,
        function_call: &crate::ast::nodes::calls::functions::FunctionCall,
    ) -> Result<(), miette::Error> {
        trace!("Compiling a native function call");

        let interner = INTERNER
            .try_lock()
            .map_err(|e| miette!("Unable to access interner: {}", e))?;
        let name = interner.resolve(function_call.name).unwrap_or("ERROR");
        match name {
            "exit" => {
                trace!("Found an exit call");

                let argument_type = self.context.i32_type();
                let return_type = self.context.void_type();
                let exit_function = self.import_function(
                    "exit",
                    return_type.into(),
                    &[argument_type.into()],
                    false,
                    None,
                )?;

                trace!("Function declared");

                let argument = argument_type.const_int(42, false);
                self.builder
                    .build_call(exit_function, &[argument.into()], "call_exit")
                    .into_diagnostic()
                    .context("While creating call to exit")?;
                self.builder
                    .build_unreachable()
                    .into_diagnostic()
                    .context("While creating unreachable end of branch")?;

                trace!("Function called");

                Ok(())
            }
            _ => todo!("Cannot compile other functions for now"),
        }
    }
}
