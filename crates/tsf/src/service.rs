//! The text service: TSF activation, owns the composer state (shared with `Sinks`).
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use std::time::{Duration, Instant, SystemTime};

use evkey_engine::composer::Composer;
use evkey_engine::engine::Config;
use evkey_engine::hotkey::{parse_hotkey, Hotkey};
use evkey_engine::settings::{self, Settings, TempOffKey};
use windows::core::{implement, ComObject, Interface, Ref, Result, GUID, IUnknown};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, IEnumTfDisplayAttributeInfo, ITfCategoryMgr, ITfCompositionSink, ITfComposition,
    ITfDisplayAttributeInfo, ITfDisplayAttributeProvider, ITfDisplayAttributeProvider_Impl,
    ITfKeyEventSink, ITfKeystrokeMgr, ITfSource, ITfTextInputProcessor_Impl, ITfTextInputProcessorEx,
    ITfTextInputProcessorEx_Impl, ITfThreadMgr, ITfThreadMgrEventSink,
    ITfContext, TF_MOD_ALT, TF_MOD_CONTROL, TF_MOD_SHIFT, TF_PRESERVEDKEY,
};

use crate::guids::{GUID_DISPLAY_ATTR, GUID_PRESERVED_TOGGLE};
use crate::sinks::Sinks;
use crate::{display, log, Lock};

/// Shared state of one activated TIP instance. `Sinks` holds an `Rc` too; `Deactivate` breaks the cycle.
pub struct Core {
    pub composer: RefCell<Composer>,
    pub tm: RefCell<Option<ITfThreadMgr>>,
    pub tid: Cell<u32>,
    pub cookie: Cell<u32>,
    pub da_atom: Cell<u32>,
    pub comp: RefCell<Option<ITfComposition>>,
    pub sink: RefCell<Option<ITfCompositionSink>>,
    /// Executable name of the host process, matched against `excluded_apps`.
    pub exe: String,
    pub excluded: Cell<bool>,
    pub temp_off_key: Cell<TempOffKey>,
    /// Toggle hotkey (`None` = disabled), applied on reload.
    pub hotkey: Cell<Option<Hotkey>>,
    /// A modifier-only chord is held and nothing else was pressed since (see `toggle.rs`).
    pub chord_armed: Cell<bool>,
    /// Vk and modifier bits of the letter toggle hotkey registered with TSF as a preserved key (`None` = none).
    preserved: Cell<Option<(u32, u32)>>,
    stamp: Cell<Option<[Option<SystemTime>; 2]>>,
    last_check: Cell<Option<Instant>>,
}

impl Core {
    fn new() -> Self {
        Self {
            composer: RefCell::new(Composer::new(Config::default())),
            tm: RefCell::new(None),
            tid: Cell::new(0),
            cookie: Cell::new(0),
            da_atom: Cell::new(0),
            comp: RefCell::new(None),
            sink: RefCell::new(None),
            exe: std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_default(),
            excluded: Cell::new(false),
            temp_off_key: Cell::new(TempOffKey::None),
            hotkey: Cell::new(parse_hotkey(&Settings::default().toggle_hotkey).ok().flatten()),
            chord_armed: Cell::new(false),
            preserved: Cell::new(None),
            stamp: Cell::new(None),
            last_check: Cell::new(None),
        }
    }

    /// Forget the composition and the engine buffer (focus change, termination, deactivate).
    pub fn lose(&self) {
        self.composer.borrow_mut().reset();
        self.comp.borrow_mut().take();
    }

    /// Reload `config.toml` / `macros.toml` if they changed (checked at most once a second). A malformed file keeps
    /// the previous settings. The hotkey applies at once; the rest waits for a word boundary, so it never lands mid-word.
    pub fn refresh(&self) {
        let now = Instant::now();
        if self.last_check.get().is_some_and(|t| now.duration_since(t) < Duration::from_secs(1)) {
            return;
        }
        self.last_check.set(Some(now));
        let Some(dir) = settings::default_dir() else { return };
        let stamp = settings::stamp(&dir);
        if self.stamp.get() == Some(stamp) {
            return;
        }
        let loaded = match settings::load(&dir) {
            Ok(loaded) => loaded,
            Err(e) => {
                self.stamp.set(Some(stamp));
                log::line(&format!("settings error, keeping previous: {e}"));
                return;
            }
        };
        match parse_hotkey(&loaded.settings.toggle_hotkey) {
            Ok(hk) => self.hotkey.set(hk),
            Err(e) => log::line(&format!("toggle_hotkey error, keeping previous: {e}")),
        }
        self.sync_preserved();
        if self.composer.borrow().is_composing() {
            // Stamp stays stale, so the next check (about a second later) applies the rest.
            self.stamp.set(None);
            return;
        }
        self.stamp.set(Some(stamp));
        self.composer.borrow_mut().apply(&loaded);
        self.excluded.set(loaded.settings.is_excluded(&self.exe));
        self.temp_off_key.set(loaded.settings.temp_off_key);
        log::line(&format!("settings loaded (excluded={})", self.excluded.get()));
    }

    /// Keeps the letter toggle hotkey registered with TSF as a preserved key, so Alt+key combos reach
    /// `OnPreservedKey`; the key sinks never see them. Win combos have no `TF_MOD_*` bit and stay on the key sinks.
    pub fn sync_preserved(&self) {
        let want = self.hotkey.get().filter(|hk| !hk.win).and_then(|hk| {
            let mods = [(hk.ctrl, TF_MOD_CONTROL), (hk.alt, TF_MOD_ALT), (hk.shift, TF_MOD_SHIFT)]
                .iter()
                .filter(|(on, _)| *on)
                .fold(0u32, |m, (_, bit)| m | *bit);
            hk.vk.map(|vk| (vk as u32, mods))
        });
        if self.preserved.get() == want {
            return;
        }
        let Some(tm) = self.tm.borrow().clone() else { return };
        self.unpreserve(&tm);
        let Some((vk, mods)) = want else { return };
        let key = TF_PRESERVEDKEY { uVKey: vk, uModifiers: mods };
        let desc: Vec<u16> = "EVKey: Vietnamese/English".encode_utf16().collect();
        let registered = tm
            .cast::<ITfKeystrokeMgr>()
            .and_then(|km| unsafe { km.PreserveKey(self.tid.get(), &GUID_PRESERVED_TOGGLE, &key, &desc) });
        match registered {
            Ok(()) => self.preserved.set(want),
            Err(e) => log::line(&format!("preserve toggle key failed: {e}")),
        }
    }

    /// Unregisters the preserved toggle key, if one is registered.
    fn unpreserve(&self, tm: &ITfThreadMgr) {
        let Some((vk, mods)) = self.preserved.take() else { return };
        let key = TF_PRESERVEDKEY { uVKey: vk, uModifiers: mods };
        if let Ok(km) = tm.cast::<ITfKeystrokeMgr>() {
            let _ = unsafe { km.UnpreserveKey(&GUID_PRESERVED_TOGGLE, &key) };
        }
    }

    /// Context of the focused document, for a preserved key that arrives without one.
    pub fn focus_context(&self) -> Option<ITfContext> {
        let tm = self.tm.borrow().clone()?;
        unsafe { tm.GetFocus().ok()?.GetTop().ok() }
    }
}


#[implement(ITfTextInputProcessorEx, ITfDisplayAttributeProvider)]
pub struct TextService {
    core: Rc<Core>,
    _lock: Lock,
}

impl TextService {
    pub fn new() -> Self {
        Self { core: Rc::new(Core::new()), _lock: Lock::new() }
    }

    fn activate(&self, tm: ITfThreadMgr, tid: u32) -> Result<()> {
        let core = &self.core;
        let sinks = ComObject::new(Sinks::new(core.clone()));
        let key: ITfKeyEventSink = sinks.to_interface();
        let thread: IUnknown = sinks.to_interface();
        let comp: ITfCompositionSink = sinks.to_interface();
        unsafe {
            tm.cast::<ITfKeystrokeMgr>()?.AdviseKeyEventSink(tid, &key, true)?;
            let cookie = tm.cast::<ITfSource>()?.AdviseSink(&ITfThreadMgrEventSink::IID, &thread)?;
            core.cookie.set(cookie);
            let cat: ITfCategoryMgr = CoCreateInstance(&CLSID_TF_CategoryMgr, None::<&IUnknown>, CLSCTX_INPROC_SERVER)?;
            core.da_atom.set(cat.RegisterGUID(&GUID_DISPLAY_ATTR)?);
        }
        core.tid.set(tid);
        *core.sink.borrow_mut() = Some(comp);
        *core.tm.borrow_mut() = Some(tm);
        core.sync_preserved();
        log::line(&format!("activated tid={tid}"));
        Ok(())
    }
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        let core = &self.core;
        let tm = core.tm.borrow_mut().take();
        if let Some(tm) = tm {
            core.unpreserve(&tm);
            unsafe {
                if let Ok(km) = tm.cast::<ITfKeystrokeMgr>() {
                    let _ = km.UnadviseKeyEventSink(core.tid.get());
                }
                if let Ok(src) = tm.cast::<ITfSource>() {
                    let _ = src.UnadviseSink(core.cookie.get());
                }
            }
        }
        core.lose();
        // Drops our reference to Sinks, which releases its Rc<Core> (cycle broken).
        core.sink.borrow_mut().take();
        core.tid.set(0);
        log::line("deactivated");
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<ITfThreadMgr>, tid: u32, _dwflags: u32) -> Result<()> {
        self.activate(ptim.ok()?.clone(), tid)
    }
}

impl ITfDisplayAttributeProvider_Impl for TextService_Impl {
    fn EnumDisplayAttributeInfo(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(display::new_enum())
    }

    fn GetDisplayAttributeInfo(&self, guid: *const GUID) -> Result<ITfDisplayAttributeInfo> {
        if unsafe { *guid } == GUID_DISPLAY_ATTR {
            Ok(display::new_info())
        } else {
            Err(E_INVALIDARG.into())
        }
    }
}
