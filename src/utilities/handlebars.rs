use anyhow::Result;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderErrorReason,
    Renderable,
};
use std::{collections::VecDeque, fs::read_dir, path::PathBuf};

pub struct DigestAssetHandlebarsHelper {
    pub key: String,
}

impl HelperDef for DigestAssetHandlebarsHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let file = h
            .param(0)
            .map(|v| v.value())
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("digest_asset", 0))?;

        let mut path = "/assets/".to_string();

        path.push_str(&file.to_string().replace("\"", ""));
        path.push_str("?v=");
        path.push_str(&self.key);

        out.write(&path)?;
        Ok(())
    }
}

pub struct EqHandlebarsHelper {}

impl HelperDef for EqHandlebarsHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let left = h
            .param(0)
            .map(|v| v.value())
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("eq", 0))?;

        let right = h
            .param(1)
            .map(|v| v.value())
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("eq", 1))?;

        let tmpl = if left.to_string().eq(&right.to_string()) {
            h.template()
        } else {
            h.inverse()
        };

        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

pub fn walk_directory(start_path: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dirs_to_visit = VecDeque::new();
    dirs_to_visit.push_back(PathBuf::from(start_path));

    while let Some(current_dir) = dirs_to_visit.pop_front() {
        for entry in read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs_to_visit.push_back(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}
