//! The CF parsing module (issue #45): reads ColdFusion source into the same
//! AST the BoxLang parser produces, so the existing compiler and VM run it.

use crate::ast::Statement;
use anyhow::Result;
use std::path::Path;

mod transpile;
pub use transpile::transpile;

/// Which front end reads a source file, chosen by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// `.bx`, `.bxs`, and anything unknown.
    BoxLangScript,
    /// `.bxm`
    BoxLangTemplate,
    /// `.cfc`, `.cfs`
    CfScript,
    /// `.cfm`
    CfTemplate,
}

impl SourceKind {
    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        match ext.as_deref() {
            Some("bxm") => Self::BoxLangTemplate,
            Some("cfm") => Self::CfTemplate,
            Some("cfc") | Some("cfs") => Self::CfScript,
            _ => Self::BoxLangScript,
        }
    }

    pub fn is_template(self) -> bool {
        matches!(self, Self::BoxLangTemplate | Self::CfTemplate)
    }
}

/// The one entry point every caller uses to turn a source file into MatchBox's AST.
pub fn parse_source(kind: SourceKind, source: &str, filename: Option<&str>) -> Result<Vec<Statement>> {
    match kind {
        SourceKind::BoxLangScript => crate::parser::parse(source, filename),
        SourceKind::BoxLangTemplate => crate::parser::parse_bxm(source, filename),
        SourceKind::CfTemplate => {
            let mut ast = crate::parser::template::parse_template_with_prefix(source, filename, "cf")?;
            transpile(&mut ast);
            Ok(ast)
        }
        SourceKind::CfScript => {
            let mut ast = crate::parser::parse_cf_script(source, filename)?;
            transpile(&mut ast);
            Ok(ast)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_from_extension() {
        let cases = [
            ("page.cfm", SourceKind::CfTemplate),
            ("PAGE.CFM", SourceKind::CfTemplate),
            ("User.cfc", SourceKind::CfScript),
            ("job.cfs", SourceKind::CfScript),
            ("page.bxm", SourceKind::BoxLangTemplate),
            ("main.bxs", SourceKind::BoxLangScript),
            ("User.bx", SourceKind::BoxLangScript),
            ("noext", SourceKind::BoxLangScript),
        ];
        for (path, kind) in cases {
            assert_eq!(SourceKind::from_path(Path::new(path)), kind, "path: {path}");
        }
    }
}
