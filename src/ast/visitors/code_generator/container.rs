use std::fs::create_dir_all;
use std::path::Path;

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context as InkContext;
use inkwell::module::Linkage;
use inkwell::types::{AnyType, BasicMetadataTypeEnum, IntType, VoidType};
use inkwell::values::{BasicMetadataValueEnum, IntValue, PointerValue};
use inkwell::{builder::Builder, module::Module};
use miette::{Context, IntoDiagnostic, Result};

use crate::ast::visitors::code_generator::to_fn_type;
use crate::ast::visitors::code_generator::typing::{
    GeneralValue, SimpleFunctionValue, SimpleValue, TypeType, ValueType,
};

pub struct GeneratorContainer<'ctx> {
    pub(super) context: &'ctx InkContext,
    pub(super) module: Module<'ctx>,
    pub(super) builder: Builder<'ctx>,
}

impl<'ctx> GeneratorContainer<'ctx> {
    pub fn goin(&self, block: BasicBlock) {
        self.builder.position_at_end(block);
    }

    pub fn void(&self) -> VoidType<'ctx> {
        self.context.void_type()
    }

    pub fn int(&self) -> IntType<'ctx> {
        self.context.i32_type()
    }

    pub fn const_int(&self, value: u32) -> IntValue<'ctx> {
        // We might want to sign extend
        self.int().const_int(value.into(), false)
    }

    pub fn import_function(
        &self,
        name: &str,
        return_type: Option<TypeType<'ctx>>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<SimpleFunctionValue<'ctx>> {
        let main_function_type = to_fn_type(
            return_type.map_or_else(|| self.void().into(), |v| v.as_any_type_enum()),
            parameters_types,
            is_var_args,
        )?;
        let main_function = self.module.add_function(name, main_function_type, linkage);

        Ok(SimpleFunctionValue {
            value: main_function,
            return_type,
        })
    }

    pub fn get_or_import(
        &self,
        name: &str,
        return_type: Option<TypeType<'ctx>>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<SimpleFunctionValue<'ctx>> {
        Ok(if let Some(func) = self.module.get_function(name) {
            SimpleFunctionValue {
                value: func,
                return_type,
            }
        } else {
            self.import_function(name, return_type, parameters_types, is_var_args, linkage)?
        })
    }

    /// This function is made to be a simplified way to create fonctions
    /// This is not a way to import fonctions
    pub fn create_function(
        &self,
        name: &str,
        return_type: Option<TypeType<'ctx>>,
        parameters_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        linkage: Option<Linkage>,
    ) -> Result<BasicBlock<'_>> {
        let main_function =
            self.import_function(name, return_type, parameters_types, is_var_args, linkage)?;
        let main_block = self.context.append_basic_block(main_function.value, name);
        Ok(main_block)
    }

    pub fn create_base(&self) -> Result<()> {
        // Create main function
        // TODO: add arguments
        let ret_type = self.int();
        let function = self.create_function("main", Some(ret_type.into()), &[], false, None)?;
        self.goin(function);
        Ok(())
    }

    pub fn save(&self, name: &str, folder: &str) -> Result<()> {
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

    pub fn build_exit_call(&self, code: ValueType<'ctx>) -> Result<()> {
        let argument_type = self.int();

        let exit_function =
            self.get_or_import("exit", None, &[argument_type.into()], false, None)?;

        // We might want to simplify this later
        // Not enough data for now
        self.builder
            .build_call(exit_function.value, &[code.into()], "call_exit")
            .into_diagnostic()
            .context("While creating call to exit")?;
        self.builder
            .build_unreachable()
            .into_diagnostic()
            .context("While creating unreachable end of branch")?;

        Ok(())
    }

    pub fn build_print_fct(&self) -> Result<SimpleFunctionValue<'ctx>> {
        let argument_type = self.context.ptr_type(AddressSpace::default());
        let return_type = self.context.i32_type();

        self.get_or_import(
            "printf",
            Some(return_type.into()),
            &[argument_type.into()],
            true,
            None,
        )
    }

    pub fn create_format_str(&self, value: GeneralValue<'ctx>) -> Result<PointerValue<'ctx>> {
        Ok(self
            .builder
            .build_global_string_ptr(if value.is_some() { "%i\n" } else { "\n" }, "format")
            .into_diagnostic()?
            .as_pointer_value())
    }

    pub fn build_print_call(&self, value: GeneralValue<'ctx>) -> Result<GeneralValue<'ctx>> {
        let print = self.build_print_fct()?;
        let format = self.create_format_str(value.clone())?;
        let args: &[BasicMetadataValueEnum<'ctx>] = if let Some(value) = value {
            &[format.into(), value.value.into()]
        } else {
            &[format.into()]
        };

        let value = self
            .builder
            .build_call(print.value, args, "call_printf")
            .into_diagnostic()
            .context("While creating call to printf")?
            .try_as_basic_value()
            .expect_basic("Printf always returns")
            .into();

        let typed = print
            .return_type
            .wrap_err("Return type of print was null")?;

        Ok(Some(SimpleValue {
            value: value,
            typed: typed,
        }))
    }
}

// Without lifetime on this impl: static
impl GeneratorContainer<'_> {
    pub fn using<F>(name: &str, callback: F) -> Result<()>
    where
        F: FnOnce(&GeneratorContainer) -> Result<()>,
    {
        let context = InkContext::create();
        let module = context.create_module(name);
        let builder = context.create_builder();

        let current = GeneratorContainer {
            context: &context,
            module,
            builder,
        };

        current.create_base()?;
        callback(&current)?;

        Ok(())
    }
}
