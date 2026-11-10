//! The window: wiring only.
//!
//! Pattern: Facade — one session per data type over its backend, the generated Admin frontend as a
//! surface each, the standard shell around them (ADR 0065). No drawing code of our own.

use std::path::PathBuf;
use std::sync::Arc;

use fs_admin_frontend::view::{AboutInfo, AdminSurface};
use fs_admin_frontend::window::{TypeView, admin_window};
use fs_admin_frontend::{Backend, Session, SystemClock};
use fs_components_app::{AppError, AppWidget, WindowOptions, launch, parse_size};
use fs_components_fields::{Look, Say};
use fs_components_styles::StyleId;
use fs_surveys_core::describe::{DECISION, QUESTION, RESPONSE, SURVEY};
use fs_surveys_core::type_registry;

use crate::{Ctx, DecisionBackend, QuestionBackend, ResponseBackend, SurveyBackend, TITLE, Words};

/// The help text.
pub const HELP: &str = "fs-surveys: the surveys window (without arguments it opens)

  --data <dir>       where the surveys are kept (default: $XDG_DATA_HOME/freesynergy/surveys)
  --snapshot <png>   draw a picture instead of opening a window
  --size <WxH>       window or picture size (default 1200x800)";

const ACTOR: &str = "https://local.invalid/users/owner";
const INSTANCE: &str = "local.invalid";

/// The window over a shared core.
///
/// # Errors
/// [`AppError`] if the shell cannot be made.
pub fn app_widget(ctx: &Arc<Ctx>) -> Result<AppWidget, AppError> {
    let say: Arc<dyn Say> = Arc::new(Words);
    let registry = type_registry().map_err(|e| AppError::Window(e.to_string()))?;
    let backends: [(&str, Arc<dyn Backend>); 4] = [
        (SURVEY, Arc::new(SurveyBackend::new(ctx.clone()))),
        (QUESTION, Arc::new(QuestionBackend::new(ctx.clone()))),
        (RESPONSE, Arc::new(ResponseBackend::new(ctx.clone()))),
        (DECISION, Arc::new(DecisionBackend::new(ctx.clone()))),
    ];
    let views = backends
        .into_iter()
        .filter_map(|(name, backend)| {
            let description = registry.get(name)?.clone();
            let label = description.label.clone();
            let session = Session::new(description, backend, Arc::new(SystemClock::new()));
            let surface =
                AdminSurface::new(session, Look::new(StyleId::Admin, false), say.clone(), None);
            Some(TypeView {
                id: name.to_owned(),
                label,
                surface: Arc::new(surface),
            })
        })
        .collect();
    admin_window(
        TITLE,
        &say,
        views,
        AboutInfo {
            name: TITLE.to_owned(),
            version: concat!("v", env!("CARGO_PKG_VERSION")).to_owned(),
            origin: "source".to_owned(),
            licence: "MIT OR Apache-2.0".to_owned(),
            repository: "https://github.com/FreeSynergy/fs-surveys".to_owned(),
        },
        "https://github.com/FreeSynergy/fs-surveys".to_owned(),
    )
}

struct Args {
    data: Option<PathBuf>,
    snapshot: Option<PathBuf>,
    size: (u32, u32),
    help: bool,
}

fn parse(words: &[String]) -> Result<Args, String> {
    let mut out = Args {
        data: None,
        snapshot: None,
        size: (1200, 800),
        help: false,
    };
    let mut it = words.iter();
    while let Some(word) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match word.as_str() {
            "--data" => out.data = Some(PathBuf::from(value("--data")?)),
            "--snapshot" => out.snapshot = Some(PathBuf::from(value("--snapshot")?)),
            "--size" => out.size = parse_size(&value("--size")?)?,
            "-h" | "--help" => out.help = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(out)
}

fn default_data() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("freesynergy/surveys"))
}

/// Open the window. `words` are the words without the program name.
///
/// # Errors
/// The message for the person: bad words, no data directory, a picture that cannot be drawn.
pub fn run(words: &[String]) -> Result<(), String> {
    let args = parse(words)?;
    if args.help {
        println!("{HELP}");
        return Ok(());
    }
    let dir = args
        .data
        .or_else(default_data)
        .ok_or("no data directory: give --data")?;
    let ctx = Ctx::open(&dir, ACTOR, INSTANCE).map_err(|e| format!("{e:?}"))?;
    let app = app_widget(&ctx).map_err(|e| e.to_string())?;
    let (width, height) = args.size;
    if let Some(file) = &args.snapshot {
        app.picture(width, height, file)
            .map_err(|e| e.to_string())?;
        eprintln!("wrote {}", file.display());
        return Ok(());
    }
    let mut options = WindowOptions::default();
    options.width = width;
    options.height = height;
    launch(options, app).map_err(|e| e.to_string())
}
