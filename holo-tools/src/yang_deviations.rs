//
// Copyright (c) The Holo Core Contributors
//
// SPDX-License-Identifier: MIT
//

use holo_yang as yang;
use yang5::context::ContextFlags;
use yang5::schema::SchemaNode;

// Usage summary printed by --help and on command-line errors.
const USAGE: &str = "\
Usage: yang_deviations <MODULE>

Parses a YANG module and generate a \"not-supported\" deviation for all of
its nodes.

Arguments:
  <MODULE>  YANG module name

Options:
  -h, --help  Print this help message and exit";

// Parse the command line, returning the YANG module name.
fn parse_args() -> Result<String, lexopt::Error> {
    let mut module_name = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            lexopt::Arg::Value(value) if module_name.is_none() => {
                module_name = Some(value.into_string()?);
            }
            lexopt::Arg::Short('h') | lexopt::Arg::Long("help") => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(module_name.ok_or("missing argument <MODULE>")?)
}

// Generate fully-prefixed schema path.
fn gen_fully_prefixed_path(snode: &SchemaNode<'_>) -> String {
    let mut path = String::new();

    for snode in snode
        .inclusive_ancestors()
        .collect::<Vec<SchemaNode<'_>>>()
        .iter()
        .rev()
    {
        // Append node name prefixed by its module prefix.
        path += &format!("/{}:{}", snode.module().prefix(), snode.name());
    }

    path
}

fn main() {
    // Parse command-line parameters.
    let module_name = parse_args().unwrap_or_else(|error| {
        eprintln!("{error}\n\n{USAGE}");
        std::process::exit(1);
    });

    // Initialize YANG context.
    let mut yang_ctx = yang::new_context();
    yang_ctx.set_options(ContextFlags::ALL_IMPLEMENTED).unwrap();

    // Load base YANG modules that define features used by other modules.
    yang::load_module(&mut yang_ctx, "ietf-bfd-types");
    yang::load_module(&mut yang_ctx, "iana-bgp-types");

    // Load requested YANG module.
    yang::load_module(&mut yang_ctx, &module_name);
    let module = yang_ctx
        .get_module_latest(&module_name)
        .expect("Failed to find loaded module");

    // Header.
    println!(
        "\
        module holo-{}-deviations {{\
        \n  yang-version 1.1;\
        \n  namespace \"http://holo-routing.org/yang/holo-{}-deviations\";\
        \n  prefix holo-{}-deviations;\
        \n\
        \n  import {} {{\
        \n    prefix {};\
        \n  }}\
        \n\
        \n  organization\
        \n    \"Holo Routing Stack\";\
        \n\
        \n  description\
        \n    \"This module defines deviation statements for the {}\
        \n     module.\";",
        module_name,
        module_name,
        module_name,
        module_name,
        module.prefix(),
        module_name
    );

    // "not-supported" deviations.
    fn print_deviation(snode: &SchemaNode<'_>) {
        println!(
            "\
        \n  /*\
        \n  deviation \"{}\" {{\
        \n    deviate not-supported;\
        \n  }}\
        \n  */",
            gen_fully_prefixed_path(snode),
        );
    }
    for snode in yang_ctx
        .traverse()
        .filter(|snode| snode.is_status_current())
        .filter(|snode| snode.module() == module)
    {
        print_deviation(&snode);
        for snode in snode.actions().flat_map(|snode| snode.traverse()) {
            print_deviation(&snode);
        }
        for snode in snode.notifications().flat_map(|snode| snode.traverse()) {
            print_deviation(&snode);
        }
    }

    // Footer.
    println!("}}");
}
