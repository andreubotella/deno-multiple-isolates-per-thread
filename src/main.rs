use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use deno_runtime::deno_core::anyhow::Error;
use deno_runtime::deno_core::error::generic_error;
use deno_runtime::deno_core::resolve_import;
use deno_runtime::deno_core::ModuleLoader;
use deno_runtime::deno_core::ModuleSource;
use deno_runtime::deno_core::ModuleSourceFuture;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::deno_core::ModuleType;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use once_cell::sync::Lazy;
use tokio::try_join;

#[cfg(feature = "tla")]
static MAIN_MODULE_SOURCE: &'static str = include_str!("./main_tla.js");
#[cfg(not(feature = "tla"))]
static MAIN_MODULE_SOURCE: &'static str = include_str!("./main_classic.js");

static MAIN_MODULE_SPECIFIER: Lazy<ModuleSpecifier> =
    Lazy::new(|| ModuleSpecifier::parse("test:main").unwrap());

struct MainModuleLoader;
impl ModuleLoader for MainModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module = if module_specifier == &*MAIN_MODULE_SPECIFIER {
            Ok(ModuleSource {
                code: String::from(MAIN_MODULE_SOURCE),
                module_type: ModuleType::JavaScript,
                module_url_specified: module_specifier.to_string(),
                module_url_found: MAIN_MODULE_SPECIFIER.to_string(),
            })
        } else {
            Err(generic_error(format!(
                "Provided module specifier \"{}\" couldn't be fetched.",
                module_specifier
            )))
        };
        Box::pin(async move { module })
    }
}

async fn run_deno_runtime(runtime_num: usize) -> Result<(), Error> {
    let options = WorkerOptions {
        bootstrap: deno_runtime::BootstrapOptions {
            args: vec![],
            apply_source_maps: false,
            cpu_count: 0,
            debug_flag: false,
            enable_testing_features: false,
            location: None,
            no_color: false,
            runtime_version: String::new(),
            ts_version: String::new(),
            unstable: true,
        },
        extensions: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        user_agent: String::new(),
        seed: None,
        module_loader: Rc::new(MainModuleLoader),
        create_web_worker_cb: Arc::new(|_| panic!("Workers not supported")),
        js_error_create_fn: None,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        get_error_class_fn: None,
        origin_storage_dir: None,
        blob_store: Default::default(),
        broadcast_channel: Default::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
    };

    let mut worker = MainWorker::bootstrap_from_options(
        MAIN_MODULE_SPECIFIER.clone(),
        Permissions::allow_all(),
        options,
    );

    worker
        .execute_script(
            "test:init",
            &format!("globalThis.runtimeNum = {};", runtime_num),
        )
        .unwrap();

    worker.execute_main_module(&*MAIN_MODULE_SPECIFIER).await?;

    worker.run_event_loop(false).await?;

    println!("Runtime {} terminated!", runtime_num);

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    #[cfg(feature = "tla")]
    println!("Running with TLA");
    #[cfg(not(feature = "tla"))]
    println!("Running without TLA");

    try_join!(
        run_deno_runtime(1),
        run_deno_runtime(2),
        run_deno_runtime(3),
        run_deno_runtime(4)
    )
    .unwrap();

    println!("Done");
}
