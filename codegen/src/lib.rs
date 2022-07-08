mod parse;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Expr, LitStr};

/// For documentation on how to use this, visit [`SupportedFields`].
///
/// [`SupportedFields`]: ./struct.SupportedFields.html
#[proc_macro]
pub fn arcdps_export(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as parse::ArcDpsGen);
    let sig = input.sig;
    let build = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is not set") + "\0";
    let build = LitStr::new(build.as_str(), Span::call_site());
    let (raw_name, span) = if let Some(input_name) = input.name {
        let name = input_name.value();
        (name, input_name.span())
    } else {
        let name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is not set");
        (name, Span::call_site())
    };
    let name = LitStr::new(raw_name.as_str(), span);
    let out_name = raw_name + "\0";
    let out_name = LitStr::new(out_name.as_str(), span);

    let (abstract_combat, cb_combat) = build_combat_area(input.raw_combat, input.combat);
    let (abstract_combat_local, cb_combat_local) =
        build_combat_local(input.raw_combat_local, input.combat_local);
    let (abstract_imgui, cb_imgui) = build_imgui(input.raw_imgui, input.imgui);
    let (abstract_options_end, cb_options_end) =
        build_options_end(input.raw_options_end, input.options_end);
    let (abstract_options_windows, cb_options_windows) =
        build_options_windows(input.raw_options_windows, input.options_windows);
    let (abstract_wnd_filter, cb_wnd_filter) =
        build_wnd_filter(input.raw_wnd_filter, input.wnd_filter);
    let (abstract_wnd_nofilter, cb_wnd_nofilter) =
        build_wnd_nofilter(input.raw_wnd_nofilter, input.wnd_nofilter);

    let export = quote! {
        ArcDpsExport {
            size: ::std::mem::size_of::<ArcDpsExport>(),
            sig: #sig,
            imgui_version: 18000,
            out_build: #build.as_ptr() as _,
            out_name: #out_name.as_ptr() as _,
            combat: #cb_combat,
            combat_local: #cb_combat_local,
            imgui: #cb_imgui,
            options_end: #cb_options_end,
            options_windows: #cb_options_windows,
            wnd_filter: #cb_wnd_filter,
            wnd_nofilter: #cb_wnd_nofilter,
        }
    };

    let init = if let Some(init) = input.init {
        let span = syn::Error::new_spanned(&init, "").span();
        quote_spanned!(span=> (#init as ::arcdps::callbacks::InitFunc)())
    } else {
        quote! { Ok(()) }
    };

    let release = if let Some(release) = input.release {
        let span = syn::Error::new_spanned(&release, "").span();
        quote_spanned!(span=> (#release as ::arcdps::callbacks::ReleaseFunc)();)
    } else {
        quote! {}
    };

    let (abstract_extras_squad_update, extras_squad_update) = build_extras_squad_update(
        input.raw_unofficial_extras_squad_update,
        input.unofficial_extras_squad_update,
    );
    let abstract_extras_init = build_extras_init(
        input.raw_unofficial_extras_init,
        input.unofficial_extras_init,
        extras_squad_update,
        &out_name,
    );

    let result = quote! {
        mod __arcdps_gen_export {
            use super::*;
            use ::arcdps::__macro::*;

            #abstract_combat
            #abstract_combat_local
            #abstract_imgui
            #abstract_options_end
            #abstract_options_windows
            #abstract_wnd_filter
            #abstract_wnd_nofilter
            #abstract_extras_squad_update
            #abstract_extras_init

            static EXPORT: ArcDpsExport = #export;
            static mut EXPORT_ERROR: ArcDpsExport = ArcDpsExport {
                size: 0,
                sig: 0,
                imgui_version: 18000,
                out_build: #build.as_ptr() as _,
                out_name: #out_name.as_ptr() as _,
                combat: None,
                combat_local: None,
                imgui: None,
                options_end: None,
                options_windows: None,
                wnd_filter: None,
                wnd_nofilter: None,
            };
            static mut ERROR_STRING: String = String::new();

            fn load() -> &'static ArcDpsExport {
                let mut export = &EXPORT;
                let res: Result<(), Box<dyn ::std::error::Error>> = #init;
                if let Err(e) = res {
                    unsafe {
                        ERROR_STRING = e.to_string();
                        EXPORT_ERROR.size = ERROR_STRING.as_ptr() as _;
                        export = &EXPORT_ERROR;
                    }
                }

                export
            }

            fn unload() {
                #release
            }

            /// ArcDPS looks for this exported function and calls the address it returns on client load.
            /// If you need any of the ignored values, create an issue with your use case.
            #[no_mangle]
            pub unsafe extern "system" fn get_init_addr(
                arc_version: *mut c_char,
                imgui_ctx: *mut imgui::sys::ImGuiContext,
                id3d: *mut c_void,
                arc_dll: HINSTANCE,
                malloc: Option<MallocFn>,
                free: Option<FreeFn>,
            ) -> fn() -> &'static ArcDpsExport {
                __init(arc_version, arc_dll, imgui_ctx, malloc, free, id32, #name);
                load
            }

            /// ArcDPS looks for this exported function and calls the address it returns on client exit.
            #[no_mangle]
            pub extern "system" fn get_release_addr() -> *mut c_void {
                unload as *mut c_void
            }
        }
    };

    result.into()
}

fn build_wnd_filter(raw_wnd: Option<Expr>, wnd: Option<Expr>) -> (TokenStream, TokenStream) {
    build_wnd(raw_wnd, wnd, quote! { abstract_wnd_filter })
}

fn build_wnd_nofilter(raw_wnd: Option<Expr>, wnd: Option<Expr>) -> (TokenStream, TokenStream) {
    build_wnd(raw_wnd, wnd, quote! { abstract_wnd_nofilter })
}

fn build_wnd(
    raw_wnd_filter: Option<Expr>,
    wnd_filter: Option<Expr>,
    func_name: TokenStream,
) -> (TokenStream, TokenStream) {
    let mut abstract_wnd_filter = quote! {};
    let cb_wnd_filter = match (raw_wnd_filter, wnd_filter) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            quote_spanned!(span => Some(#raw as _) )
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            abstract_wnd_filter = quote_spanned! {span=>
                unsafe extern "C" fn #func_name(_h_wnd: *mut c_void, u_msg: u32, w_param: WPARAM, l_param: LPARAM) -> u32 {
                    let _ = #safe as WndProcCallback;

                    match u_msg {
                        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
                            let key_down = u_msg & 1 == 0;
                            let prev_key_down = (l_param.0 >> 30) & 1 == 1;

                            if #safe(w_param.0, key_down, prev_key_down) {
                                u_msg
                            } else {
                                0
                            }
                        },
                        _ => u_msg,
                    }
                }
            };
            quote_spanned!(span=> Some(__arcdps_gen_export::#func_name as _) )
        }
        _ => quote! { None },
    };

    (abstract_wnd_filter, cb_wnd_filter)
}

fn build_options_windows(
    raw_options_windows: Option<Expr>,
    options_windows: Option<Expr>,
) -> (TokenStream, TokenStream) {
    let mut abstract_options_windows = quote! {};
    let cb_options_windows = match (raw_options_windows, options_windows) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            quote_spanned!(span=> Some(#raw as _) )
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            abstract_options_windows = quote_spanned! {span =>
                unsafe extern "C" fn abstract_options_windows(window_name: *mut c_char) -> bool {
                    let _ = #safe as OptionsWindowsCallback;

                    #safe(__ui(), str_from_cstr(window_name))
                }
            };
            quote_spanned!(span=> Some(__arcdps_gen_export::abstract_options_windows as _) )
        }
        _ => quote! { None },
    };

    (abstract_options_windows, cb_options_windows)
}

fn build_options_end(
    raw_options_end: Option<Expr>,
    options_end: Option<Expr>,
) -> (TokenStream, TokenStream) {
    let mut abstract_options_end = quote! {};
    let cb_options_end = match (raw_options_end, options_end) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            quote_spanned!(span=> Some(#raw as _) )
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            abstract_options_end = quote_spanned! {span =>
                unsafe extern "C" fn abstract_options_end() {
                    let _ = #safe as OptionsCallback;

                    #safe(__ui())
                }
            };
            quote_spanned!(span=> Some(__arcdps_gen_export::abstract_options_end as _) )
        }
        _ => quote! { None },
    };

    (abstract_options_end, cb_options_end)
}

fn build_imgui(raw_imgui: Option<Expr>, imgui: Option<Expr>) -> (TokenStream, TokenStream) {
    let mut abstract_imgui = quote! {};
    let cb_imgui = match (raw_imgui, imgui) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            quote_spanned!(span=> Some(#raw as _) )
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            abstract_imgui = quote_spanned! {span =>
                unsafe extern "C" fn abstract_imgui(loading: u32) {
                    let _ = #safe as ImguiCallback;

                    #safe(__ui(), loading != 0)
                }
            };
            quote_spanned!(span=> Some(__arcdps_gen_export::abstract_imgui as _) )
        }
        _ => quote! { None },
    };

    (abstract_imgui, cb_imgui)
}

fn build_combat_local(
    raw_combat: Option<Expr>,
    combat: Option<Expr>,
) -> (TokenStream, TokenStream) {
    build_combat_helper(raw_combat, combat, quote! { abstract_combat_local })
}

fn build_combat_area(raw_combat: Option<Expr>, combat: Option<Expr>) -> (TokenStream, TokenStream) {
    build_combat_helper(raw_combat, combat, quote! { abstract_combat })
}

fn build_combat_helper(
    raw_combat: Option<Expr>,
    combat: Option<Expr>,
    func_name: TokenStream,
) -> (TokenStream, TokenStream) {
    let mut abstract_combat = quote! {};
    let cb_combat = match (raw_combat, combat) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            quote_spanned!(span=> Some(#raw as _) )
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            abstract_combat = quote_spanned! {span =>
                unsafe extern "C" fn #func_name(
                        event: Option<&::arcdps::api::RawCombatEvent>,
                        src: Option<&::arcdps::api::RawAgent>,
                        dst: Option<&::arcdps::api::RawAgent>,
                        skill_name: *mut c_char,
                        id: u64,
                        revision: u64,
                    ) {
                        let _ = #safe as CombatCallback;

                        #safe(
                            event.map(Into::into),
                            src.map(Into::into),
                            dst.map(Into::into),
                            str_from_cstr(skill_name),
                            id,
                            revision
                        )
                }
            };
            quote_spanned!(span=> Some(__arcdps_gen_export::#func_name as _) )
        }
        _ => quote! { None },
    };

    (abstract_combat, cb_combat)
}

fn build_extras_init(
    raw: Option<Expr>,
    safe: Option<Expr>,
    squad_update: Option<TokenStream>,
    name: &LitStr,
) -> TokenStream {
    let has_update = squad_update.is_some();
    let squad_cb = squad_update.unwrap_or(quote! { None });

    // we only subscribe if compat check passes
    // info may still be read for safe version
    let subscribe = quote! {
        if addon.check_compat() {
            sub.subscribe(#name, #squad_cb);
        }
    };

    let abstract_wrapper = match (raw, safe) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            quote_spanned! {span=>
                let _ = #raw as RawExtrasSubscriberInit;

                #raw(addon, sub)
            }
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            quote_spanned! {span=>
                let _ = #safe as ExtrasInitFunc;

                #subscribe

                let user = str_from_cstr(addon.self_account_name as _)
                    .map(|n| n.trim_start_matches(':'));
                #safe(addon.into(), user)
            }
        }
        _ if has_update => quote! {
                #subscribe
        },
        _ => return quote! {},
    };

    quote_spanned! {abstract_wrapper.span()=>
        #[no_mangle]
        unsafe extern "system" fn arcdps_unofficial_extras_subscriber_init(
            addon: &::arcdps::extras::RawExtrasAddonInfo,
            sub: &mut ::arcdps::extras::ExtrasSubscriberInfo
        ) {
            #abstract_wrapper
        }
    }
}

fn build_extras_squad_update(
    raw: Option<Expr>,
    safe: Option<Expr>,
) -> (TokenStream, Option<TokenStream>) {
    let mut abstract_wrapper = quote! {};
    let cb_safe = match (raw, safe) {
        (Some(raw), _) => {
            let span = syn::Error::new_spanned(&raw, "").span();
            Some(quote_spanned!(span => Some(#raw as _) ))
        }
        (_, Some(safe)) => {
            let span = syn::Error::new_spanned(&safe, "").span();
            abstract_wrapper = quote_spanned! {span=>
                unsafe extern "C" fn abstract_extras_squad_update(users: *const ::arcdps::extras::RawUserInfo, count: u64) {
                    let _ = #safe as ExtrasSquadUpdateCallback;

                    #safe(::arcdps::extras::to_user_info_iter(users, count))
                }
            };
            Some(
                quote_spanned!(span=> Some(__arcdps_gen_export::abstract_extras_squad_update as _) ),
            )
        }
        _ => None,
    };

    (abstract_wrapper, cb_safe)
}
