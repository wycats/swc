use crate::util::IdentExt;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_fold_type, Fold};

/// `@babel/plugin-proposal-export-default-from` and
/// `@babel/plugin-proposal-export-namespace-from`
pub fn export() -> impl Fold {
    ExportDefaultFrom
}

#[derive(Clone)]
struct ExportDefaultFrom;

impl Fold for ExportDefaultFrom {
    noop_fold_type!();

    fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        // Imports
        let mut stmts = Vec::with_capacity(items.len() + 4);
        // Statements except import
        let mut extra_stmts = Vec::with_capacity(items.len() + 4);

        for item in items {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(mut export)) => {
                    // Skip if it does not have neither default export and namespace export
                    if export.specifiers.iter().all(|s| match *s {
                        ExportSpecifier::Named(..) => true,
                        _ => false,
                    }) {
                        extra_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export)));
                        continue;
                    }

                    match export.specifiers.remove(0) {
                        ExportSpecifier::Default(ExportDefaultSpecifier { exported: default }) => {
                            let local = default.prefix("_").private();

                            stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                                span: DUMMY_SP,
                                specifiers: vec![ImportSpecifier::Default(
                                    ImportDefaultSpecifier {
                                        span: DUMMY_SP,
                                        local: local.clone(),
                                    },
                                )],
                                src: export
                                    .src
                                    .clone()
                                    .expect("`export default from` requires source"),
                                type_only: false,
                            })));
                            extra_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                                NamedExport {
                                    span: DUMMY_SP,
                                    specifiers: vec![ExportSpecifier::Named(
                                        ExportNamedSpecifier {
                                            span: DUMMY_SP,
                                            orig: local,
                                            exported: Some(default),
                                        },
                                    )],
                                    src: None,
                                    type_only: false,
                                },
                            )));
                        }
                        ExportSpecifier::Namespace(ns) => {
                            let local = ns.name.prefix("_").private();

                            stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                                span: DUMMY_SP,
                                specifiers: vec![ImportSpecifier::Namespace(
                                    ImportStarAsSpecifier {
                                        span: DUMMY_SP,
                                        local: local.clone(),
                                    },
                                )],
                                src: export
                                    .src
                                    .clone()
                                    .expect("`export default from` requires source"),
                                type_only: false,
                            })));
                            extra_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                                NamedExport {
                                    span: DUMMY_SP,
                                    specifiers: vec![ExportSpecifier::Named(
                                        ExportNamedSpecifier {
                                            span: DUMMY_SP,
                                            orig: local,
                                            exported: Some(ns.name),
                                        },
                                    )],
                                    src: None,
                                    type_only: false,
                                },
                            )));
                        }
                        _ => unreachable!(),
                    };
                    if !export.specifiers.is_empty() {
                        extra_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export)));
                    }
                }
                ModuleItem::ModuleDecl(ModuleDecl::Import(..)) => stmts.push(item),
                _ => extra_stmts.push(item),
            }
        }

        stmts.append(&mut extra_stmts);

        stmts.shrink_to_fit();

        stmts
    }
}
