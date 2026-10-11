//
// Copyright (c) The Holo Core Contributors
//
// SPDX-License-Identifier: MIT
//

use convert_case::Case;
use holo_northbound::yang_codegen::SchemaNodeCodegenExt;
use holo_northbound::yang_codegen::struct_builder::StructBuilder;
use holo_yang as yang;
use yang5::context::{Context, ContextFlags};
use yang5::schema::{SchemaModule, SchemaNodeKind};

#[derive(Debug)]
struct Args {
    modules: Vec<String>,
    impl_type: String,
}

// Usage summary printed by --help and on command-line errors.
const USAGE: &str = "\
Usage: yang_impls --module <MODULE>... --type <TYPE>

Parses a YANG module and generate all required northbound callbacks for its
nodes.

Options:
      --module <MODULE>  YANG module name
      --type <TYPE>      Type (state/rpc)
  -h, --help             Print this help message and exit";

fn parse_args() -> Result<Args, lexopt::Error> {
    let mut modules = vec![];
    let mut impl_type = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            lexopt::Arg::Long("module") => {
                for value in parser.values()? {
                    modules.push(value.into_string()?);
                }
            }
            lexopt::Arg::Long("type") => {
                impl_type = Some(parser.value()?.into_string()?);
            }
            lexopt::Arg::Short('h') | lexopt::Arg::Long("help") => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    if modules.is_empty() {
        return Err("missing option '--module'".into());
    }

    Ok(Args {
        modules,
        impl_type: impl_type.ok_or("missing option '--type'")?,
    })
}

fn gen_impl_blocks_rpc(yang_ctx: &Context, modules: Vec<SchemaModule<'_>>) {
    let indent1 = " ".repeat(4);
    let indent2 = " ".repeat(2 * 4);

    for snode in yang_ctx
        .traverse()
        .filter(|snode| snode.is_status_current())
        .filter(|snode| modules.iter().any(|module| snode.module() == *module))
    {
        if snode.kind() == SchemaNodeKind::Rpc {
            let module = snode.rust_module_path();
            let struct_name = snode.rust_name(Case::Pascal);
            let builder = StructBuilder::new(snode.clone());
            let lifetime = if builder.needs_lifetime() { "<'_>" } else { "" };
            println!(
                "impl YangRpc<Provider> for yang::{module}::{struct_name}{lifetime}"
            );
            println!("{{");
            println!(
                "{indent1}fn invoke(&mut self, _provider: &mut Provider) -> RpcResult {{"
            );
            println!("{indent2}Ok(())");
            println!("{indent1}}}");
            println!("}}");
            println!();
        }

        for snode in snode.actions() {
            let module = snode.rust_module_path();
            let struct_name = snode.rust_name(Case::Pascal);
            let builder = StructBuilder::new(snode.clone());
            let lifetime = if builder.needs_lifetime() { "<'_>" } else { "" };
            println!(
                "impl YangRpc<Provider> for yang::{module}::{struct_name}{lifetime}"
            );
            println!("{{");
            println!(
                "{indent1}fn invoke(&mut self, _provider: &mut Provider) -> RpcResult {{"
            );
            println!("{indent2}Ok(())");
            println!("{indent1}}}");
            println!("}}");
            println!();
        }
    }
}

fn gen_impl_blocks_state(yang_ctx: &Context, modules: Vec<SchemaModule<'_>>) {
    let indent1 = " ".repeat(4);
    let indent2 = " ".repeat(2 * 4);
    let indent3 = " ".repeat(3 * 4);

    for snode in yang_ctx
        .traverse()
        .filter(|snode| !snode.is_schema_only())
        .filter(|snode| snode.is_status_current())
        .filter(|snode| {
            matches!(
                snode.kind(),
                SchemaNodeKind::Container | SchemaNodeKind::List
            )
        })
        .filter(|snode| modules.iter().any(|module| snode.module() == *module))
    {
        let module = snode.rust_module_path();
        let struct_name = snode.rust_name(Case::Pascal);
        let builder = StructBuilder::new(snode.clone());
        let fields = &builder.fields;
        let lifetime = if builder.needs_lifetime() { "<'a>" } else { "" };

        if snode.is_state_container() {
            if fields.is_empty() {
                continue;
            }
            println!(
                "impl<'a> YangContainer<'a, Provider> for {module}::{struct_name}{lifetime}"
            );
            println!("{{");
            println!("{indent1}type ParentListEntry = (); // TODO");
            println!();
            println!(
                "{indent1}fn new(_provider: &'a Provider, _: &Self::ParentListEntry) -> Option<Self> {{"
            );
            println!("{indent2}Some(Self {{");
            for snode in fields
                .iter()
                .filter(|snode| {
                    matches!(
                        snode.kind(),
                        SchemaNodeKind::Leaf | SchemaNodeKind::LeafList
                    )
                })
                .filter(|snode| snode.is_state() || snode.is_list_key())
            {
                let field_name = snode.rust_name(Case::Snake);
                println!("{indent3}{field_name}: todo!(),");
            }
            println!("{indent2}}})");
            println!("{indent1}}}");
            println!("}}");
            println!();
        }
        if snode.is_state_list() {
            println!(
                "impl<'a> YangList<'a, Provider> for {module}::{struct_name}{lifetime}"
            );
            println!("{{");
            println!("{indent1}type ParentListEntry = (); // TODO");
            println!("{indent1}type ListEntry = (); // TODO");
            println!();
            println!(
                "{indent1}fn iter(_provider: &'a Provider, _list_entry: &Self::ParentListEntry) -> Option<impl ListIterator<'a, Self::ListEntry>> {{"
            );
            println!("{indent2}None::<std::iter::Empty<_>>");
            println!("{indent1}}}");
            println!();
            println!(
                "{indent1}fn new(_provider: &'a Provider, _list_entry: &Self::ListEntry) -> Self {{"
            );
            println!("{indent2}Self {{");
            for snode in fields
                .iter()
                .filter(|snode| {
                    matches!(
                        snode.kind(),
                        SchemaNodeKind::Leaf | SchemaNodeKind::LeafList
                    )
                })
                .filter(|snode| snode.is_state() || snode.is_list_key())
            {
                let field_name = snode.rust_name(Case::Snake);
                println!("{indent3}{field_name}: todo!(),");
            }
            println!("{indent2}}}");
            println!("{indent1}}}");
            println!("}}");
            println!();
        }
    }
}

fn main() {
    // Parse command-line parameters.
    let args = parse_args().unwrap_or_else(|error| {
        eprintln!("{error}\n\n{USAGE}");
        std::process::exit(1);
    });

    // Initialize context.
    let mut yang_ctx = yang::new_context();
    yang_ctx.set_options(ContextFlags::ALL_IMPLEMENTED).unwrap();

    // Load provided YANG module.
    for module_name in &args.modules {
        yang::load_module(&mut yang_ctx, module_name);
        yang::load_deviations(&mut yang_ctx, module_name);
    }

    // Generate impl blocks.
    let modules = args
        .modules
        .iter()
        .map(|module_name| {
            yang_ctx
                .get_module_latest(module_name)
                .expect("Failed to find loaded module")
        })
        .collect::<Vec<_>>();
    match args.impl_type.as_str() {
        "rpc" => gen_impl_blocks_rpc(&yang_ctx, modules),
        "state" => gen_impl_blocks_state(&yang_ctx, modules),
        _ => panic!("Unknown type (valid options: rpc, state)"),
    }
}
