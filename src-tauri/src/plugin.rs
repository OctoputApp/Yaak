use std::path;
use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::{
    js_string, module::SimpleModuleLoader, property::Attribute, Context, JsNativeError, JsValue,
    Module, Source,
};
use boa_runtime::Console;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::deno::run_plugin_deno_block;
use crate::models::{HttpRequest, WorkspaceExportResources};

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct FilterResult {
    pub filtered: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ImportResult {
    pub resources: WorkspaceExportResources,
}

pub async fn run_plugin_filter(
    app_handle: &AppHandle,
    plugin_name: &str,
    response_body: &str,
    filter: &str,
) -> Option<FilterResult> {
    let result_json = run_plugin(
        app_handle,
        plugin_name,
        "pluginHookResponseFilter",
        &[js_string!(response_body).into(), js_string!(filter).into()],
    );

    if result_json.is_null() {
        error!("Plugin {} failed to run", plugin_name);
        return None;
    }

    let resources: FilterResult =
        serde_json::from_value(result_json).expect("failed to parse filter plugin result json");
    Some(resources)
}

pub fn run_plugin_export_curl(
    app_handle: &AppHandle,
    request: &HttpRequest,
) -> Result<String, String> {
    let mut context = Context::default();
    let request_json = serde_json::to_value(request).map_err(|e| e.to_string())?;
    let result_json = run_plugin(
        app_handle,
        "exporter-curl",
        "pluginHookExport",
        &[JsValue::from_json(&request_json, &mut context).map_err(|e| e.to_string())?],
    );

    let resources: String = serde_json::from_value(result_json).map_err(|e| e.to_string())?;
    Ok(resources)
}

pub async fn run_plugin_import(
    plugin_name: &str,
    file_contents: &str,
) -> Result<Option<ImportResult>, String> {
    let plugin_dir = path::Path::new("/Users/gschier/Workspace/yaak/plugins");
    let plugin_index_file = plugin_dir.join(plugin_name).join("src/index.ts");

    run_plugin_deno_block(
        plugin_index_file.to_str().unwrap(),
        "pluginHookImport",
        file_contents,
    ).map_err(|e| e.to_string())
}

fn run_plugin(
    app_handle: &AppHandle,
    plugin_name: &str,
    entrypoint: &str,
    js_args: &[JsValue],
) -> serde_json::Value {
    let plugin_dir = app_handle
        .path()
        .resolve("plugins", BaseDirectory::Resource)
        .expect("failed to resolve plugin directory resource")
        .join(plugin_name);
    let plugin_index_file = plugin_dir.join("index.mjs");

    debug!(
        "Running plugin dir={:?} file={:?}",
        plugin_dir, plugin_index_file
    );

    let loader = Rc::new(SimpleModuleLoader::new(plugin_dir).unwrap());
    let context = &mut Context::builder()
        .module_loader(loader.clone())
        .build()
        .expect("failed to create context");

    add_runtime(context);

    let source = Source::from_filepath(&plugin_index_file).expect("Error opening file");

    // Can also pass a `Some(realm)` if you need to execute the module in another realm.
    let module = Module::parse(source, None, context).expect("failed to parse module");

    // Insert parsed entrypoint into the module loader
    loader.insert(plugin_index_file, module.clone());

    let promise_result = module.load_link_evaluate(context);

    // Very important to push forward the job queue after queueing promises.
    context.run_jobs();

    // Checking if the final promise didn't return an error.
    match promise_result.state() {
        PromiseState::Pending => {
            panic!("Promise was pending");
        }
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::undefined())
        }
        PromiseState::Rejected(err) => {
            panic!("Failed to link: {}", err.display());
        }
    }

    let namespace = module.namespace(context);

    let result = namespace
        .get(js_string!(entrypoint), context)
        .expect("failed to get entrypoint")
        .as_callable()
        .cloned()
        .ok_or_else(|| JsNativeError::typ().with_message("export wasn't a function!"))
        .expect("Failed to get entrypoint")
        .call(&JsValue::undefined(), js_args, context)
        .expect("Failed to call entrypoint");

    match result.is_undefined() {
        true => json!(null), // to_json doesn't work with undefined (yet)
        false => result
            .to_json(context)
            .expect("failed to convert result to json"),
    }
}

fn add_runtime(context: &mut Context) {
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}
