use super::storage::{timestamp, Bookmark, Settings, Visit};

pub(super) fn toolbar_html() -> &'static str {
    r#"<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
:root{color-scheme:dark;--bg:#202126;--tabs:#25262b;--bar:#2b2c31;--line:#3a3b41;--text:#e5e5e8;--muted:#a1a2a8;--accent:#9a83d7;--hover:#383940;--field:#202126;--panel:#2b2c31}
:root[data-theme=light]{color-scheme:light;--bg:#f2f2f4;--tabs:#e7e7ea;--bar:#f8f8fa;--line:#d5d5da;--text:#242429;--muted:#676870;--accent:#7256b4;--hover:#e6e5e9;--field:#fff;--panel:#fff}
*{box-sizing:border-box;font-family:Arial,sans-serif!important}body{height:100vh;margin:0;background:var(--bg);color:var(--text);font:13px Arial,sans-serif;user-select:none}
.tabstrip{height:34px;display:flex;align-items:stretch;gap:2px;padding:3px 6px 0;background:var(--tabs);border-bottom:1px solid var(--line);overflow:hidden}
.tab{position:relative;display:flex;align-items:center;gap:7px;flex:0 1 205px;min-width:88px;max-width:205px;height:30px;padding:0 8px;border:1px solid transparent;border-bottom:0;border-radius:4px 4px 0 0;background:transparent;color:var(--muted);text-align:left;cursor:pointer}
.tab.active{background:var(--bar);color:var(--text);border-color:var(--line)}
.tab .label{flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}
.tab .dot{flex:none;color:var(--accent);font-size:9px}.tab .close{flex:none;width:19px;height:19px;border-radius:3px;color:var(--muted);font-size:15px;line-height:17px;text-align:center}.tab .close:hover{background:var(--hover);color:var(--text)}
.tab-add,.icon-button,.menu-trigger{display:grid;place-items:center;flex:none;width:30px;height:29px;border:0;border-radius:4px;background:transparent;color:var(--muted);cursor:pointer}
.tab-add:hover,.icon-button:hover,.menu-trigger:hover,.menu-trigger[aria-expanded=true]{background:var(--hover);color:var(--text)}
.tab-add{font-size:18px}.toolbar{height:44px;display:flex;align-items:center;gap:4px;padding:0 8px;background:var(--bar);border-bottom:1px solid var(--line)}
.icon-button{width:30px;height:30px}.icon-button svg{width:16px;height:16px;fill:none;stroke:currentColor;stroke-width:1.7;stroke-linecap:round;stroke-linejoin:round}
.address{height:30px;display:flex;align-items:center;gap:7px;flex:1;min-width:80px;margin:0 4px;padding:0 8px;border:1px solid var(--line);border-radius:5px;background:var(--field)}
.address:focus-within{border-color:var(--accent)}.address input{flex:1;min-width:0;height:100%;padding:0;border:0;outline:0;background:transparent;color:var(--text);font:13px Arial,sans-serif!important}
.address input::placeholder{color:var(--muted)}.address-icon{width:14px;height:14px;color:var(--muted)}.stop-button{width:22px;height:22px;font-size:15px}
.go{height:28px;padding:0 10px;border:1px solid var(--line);border-radius:4px;background:transparent;color:var(--text);font-size:12px;cursor:pointer}.go:hover{background:var(--hover)}
.status{max-width:150px;overflow:hidden;color:var(--muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}
.menu-wrap{position:relative;flex:none}.menu-trigger{font-size:20px;line-height:1}
.menu-panel{display:none;position:absolute;z-index:10;top:34px;right:0;width:260px;max-height:calc(100vh - 42px);overflow:auto;padding:5px;border:1px solid var(--line);border-radius:5px;background:var(--panel);box-shadow:0 8px 22px #0005}
.menu-panel.open{display:block}.menu-heading{padding:7px 8px 3px;color:var(--muted);font-size:10px;font-weight:bold;letter-spacing:.5px;text-transform:uppercase}
.menu-item{display:flex;align-items:center;justify-content:space-between;width:100%;min-height:27px;padding:4px 8px;border:0;border-radius:3px;background:transparent;color:var(--text);font-size:12px;text-align:left;cursor:pointer}
.menu-item:hover{background:var(--hover)}.shortcut{margin-left:16px;color:var(--muted);font-size:11px}.menu-separator{height:1px;margin:4px 6px;background:var(--line)}
</style></head>
<body>
<div class="tabstrip" id="tabs"></div>
<div class="toolbar">
 <button class="icon-button" title="Back (Alt+Left)" aria-label="Back" onclick="send('back')"><svg viewBox="0 0 24 24"><path d="M19 12H5M12 19l-7-7 7-7"/></svg></button>
 <button class="icon-button" title="Forward (Alt+Right)" aria-label="Forward" onclick="send('forward')"><svg viewBox="0 0 24 24"><path d="M5 12h14m-7-7 7 7-7 7"/></svg></button>
 <button class="icon-button" id="reload" title="Reload (Ctrl+R)" aria-label="Reload" onclick="send('reload')"><svg viewBox="0 0 24 24"><path d="M20 11a8 8 0 0 0-14-5L4 8"/><path d="M4 4v4h4m-4 5a8 8 0 0 0 14 5l2-2m0 4v-4h-4"/></svg></button>
 <button class="icon-button" title="Home" aria-label="Home" onclick="send('home')"><svg viewBox="0 0 24 24"><path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z"/></svg></button>
 <div class="address"><svg class="address-icon" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-width="1.7" d="M7 10V7a5 5 0 0 1 10 0v3m-12 0h14v11H5z"/></svg><input id="address" spellcheck="false" autocomplete="off" placeholder="Search or enter address"><button class="icon-button stop-button" title="Stop loading" aria-label="Stop loading" onclick="send('stop')">×</button></div>
 <button class="go" onclick="navigate()">Go</button>
 <button class="icon-button" title="Bookmark this page (Ctrl+D)" aria-label="Bookmark this page" onclick="send('bookmark')"><svg viewBox="0 0 24 24"><path d="M6 4h12v17l-6-4-6 4z"/></svg></button>
 <span class="status" id="status">RUSK Browser</span>
 <div class="menu-wrap"><button class="menu-trigger" id="menu-trigger" title="Browser menu" aria-label="Browser menu" aria-expanded="false">⋮</button>
  <div class="menu-panel" id="menu-panel">
   <div class="menu-heading">File</div><button class="menu-item" data-action="new_tab">New tab <span class="shortcut">Ctrl+T</span></button><button class="menu-item" data-action="close_tab">Close tab <span class="shortcut">Ctrl+W</span></button><button class="menu-item" data-action="new_window">New window</button><button class="menu-item" data-action="exit">Exit</button>
   <div class="menu-separator"></div><div class="menu-heading">Edit</div><button class="menu-item" data-action="cut">Cut</button><button class="menu-item" data-action="copy">Copy</button><button class="menu-item" data-action="paste">Paste</button><button class="menu-item" data-action="select_all">Select all</button>
   <div class="menu-separator"></div><div class="menu-heading">View</div><button class="menu-item" data-action="zoom_in">Zoom in <span class="shortcut">Ctrl++</span></button><button class="menu-item" data-action="zoom_out">Zoom out <span class="shortcut">Ctrl+-</span></button><button class="menu-item" data-action="zoom_reset">Reset zoom <span class="shortcut">Ctrl+0</span></button><button class="menu-item" data-action="fullscreen">Fullscreen <span class="shortcut">F11</span></button><button class="menu-item" data-action="devtools">Developer tools</button>
   <div class="menu-separator"></div><div class="menu-heading">History</div><button class="menu-item" data-action="back">Back</button><button class="menu-item" data-action="forward">Forward</button><button class="menu-item" data-action="show_history">Recently visited pages <span class="shortcut">Ctrl+H</span></button>
   <div class="menu-separator"></div><div class="menu-heading">Bookmarks</div><button class="menu-item" data-action="bookmark">Add bookmark <span class="shortcut">Ctrl+D</span></button><button class="menu-item" data-action="show_bookmarks">Manage bookmarks <span class="shortcut">Ctrl+B</span></button><button class="menu-item" data-action="show_settings">Settings</button>
  </div>
 </div>
</div>
<script>
const post=window.ipc&&window.ipc.postMessage?window.ipc.postMessage.bind(window.ipc):()=>{};
function closeMenu(){document.getElementById('menu-panel').classList.remove('open');document.getElementById('menu-trigger').setAttribute('aria-expanded','false')}
function send(action,value){post(JSON.stringify({action,value:value||''}));closeMenu()}
function navigate(){const input=document.getElementById('address');send('navigate',input.value);input.blur()}
document.getElementById('address').addEventListener('keydown',e=>{if(e.key==='Enter')navigate()});
document.getElementById('address').addEventListener('focus',e=>e.target.select());
document.getElementById('menu-trigger').addEventListener('click',e=>{e.stopPropagation();const panel=document.getElementById('menu-panel');panel.classList.toggle('open');const opened=panel.classList.contains('open');e.currentTarget.setAttribute('aria-expanded',opened);post(JSON.stringify({action:opened?'menu_open':'menu_close'}))});
document.querySelectorAll('.menu-item').forEach(item=>item.addEventListener('click',()=>send(item.dataset.action)));
document.addEventListener('click',e=>{if(!e.target.closest('.menu-wrap')){const wasOpen=document.getElementById('menu-panel').classList.contains('open');closeMenu();if(wasOpen)post(JSON.stringify({action:'menu_close'}))}});
document.addEventListener('keydown',e=>{if(!(e.ctrlKey||e.metaKey))return;const key=e.key.toLowerCase();const actions={t:'new_tab',w:'close_tab',r:'reload',d:'bookmark',h:'show_history',b:'show_bookmarks'};if(key==='l'){e.preventDefault();const a=document.getElementById('address');a.focus();a.select()}else if(actions[key]){e.preventDefault();send(actions[key])}else if(key==='tab'){e.preventDefault();send(e.shiftKey?'previous_tab':'next_tab')}else if(key==='+'||key==='='){e.preventDefault();send('zoom_in')}else if(key==='-'){e.preventDefault();send('zoom_out')}else if(key==='0'){e.preventDefault();send('zoom_reset')}});
document.addEventListener('keydown',e=>{if(e.key==='Escape'&&document.getElementById('menu-panel').classList.contains('open')){closeMenu();post(JSON.stringify({action:'menu_close'}))}});
document.addEventListener('keydown',e=>{if(e.key==='F11'){e.preventDefault();send('fullscreen')}else if(e.altKey&&e.key==='ArrowLeft'){e.preventDefault();send('back')}else if(e.altKey&&e.key==='ArrowRight'){e.preventDefault();send('forward')}});
window.ruskUpdate=data=>{const tabs=document.getElementById('tabs');tabs.replaceChildren();for(const t of data.tabs){const b=document.createElement('button');b.className='tab'+(t.active?' active':'');b.title=t.title;b.onclick=()=>send('switch_tab',String(t.id));const dot=document.createElement('span');dot.className='dot';dot.textContent=t.loading?'◌':'●';const label=document.createElement('span');label.className='label';label.textContent=t.title;const close=document.createElement('span');close.className='close';close.textContent='×';close.onclick=e=>{e.stopPropagation();send('close_tab',String(t.id))};b.append(dot,label,close);tabs.append(b)}const plus=document.createElement('button');plus.className='tab-add';plus.textContent='+';plus.title='New tab';plus.onclick=()=>send('new_tab');tabs.append(plus);const address=document.getElementById('address');if(document.activeElement!==address)address.value=data.url||'';document.getElementById('status').textContent=data.status||'RUSK Browser';const reload=document.getElementById('reload');reload.onclick=()=>send(data.loading?'stop':'reload');reload.title=data.loading?'Stop loading':'Reload (Ctrl+R)';reload.innerHTML=data.loading?'■':'<svg viewBox="0 0 24 24"><path d="M20 11a8 8 0 0 0-14-5L4 8"/><path d="M4 4v4h4m-4 5a8 8 0 0 0 14 5l2-2m0 4v-4h-4"/></svg>';document.documentElement.dataset.theme=data.theme||'dark'};
</script></body></html>"#
}

pub(super) fn home_html(bookmarks: &[Bookmark], history: &[Visit], theme: &str) -> String {
    let bookmark_cards = if bookmarks.is_empty() {
        [
            ("Google", "https://www.google.com"),
            ("YouTube", "https://www.youtube.com"),
            ("Wikipedia", "https://www.wikipedia.org"),
        ]
        .iter()
        .map(|(title, url)| card(title, url))
        .collect::<String>()
    } else {
        bookmarks
            .iter()
            .take(8)
            .map(|item| card(&item.title, &item.url))
            .collect::<String>()
    };
    let recent_cards = history
        .iter()
        .rev()
        .take(5)
        .map(|item| card(&item.title, &item.url))
        .collect::<String>();
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>RUSK Browser</title>{}</head><body data-theme="{}">
<main class="home"><header class="brand"><span class="mark">R</span><span class="brandname">RUSK <span>Browser</span></span></header>
<form id="search"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="10.8" cy="10.8" r="6.8"/><path d="m16 16 5 5"/></svg><input id="q" autofocus placeholder="Search or enter an address"><button aria-label="Search">Search</button></form>
<section><h2>Bookmarks</h2><div class="site-list">{saved}</div></section>
<section><h2>Recently visited</h2><div class="site-list">{recent}</div></section>
</main><script>{script}</script></body></html>"#,
        content_style(),
        escape(theme),
        recent = if recent_cards.is_empty() {
            empty_card("Pages you visit will appear here.")
        } else {
            recent_cards
        },
        saved = bookmark_cards,
        script = script()
    )
}

pub(super) fn history_html(history: &[Visit], theme: &str) -> String {
    let rows = history
        .iter()
        .rev()
        .take(100)
        .map(|item| {
            format!(
                r#"<button class="row" data-url="{}"><span>{}</span><small>{}</small></button>"#,
                escape(&item.url),
                escape(&item.title),
                escape(&format!("{} · {}", item.url, date_label(item.visited_at)))
            )
        })
        .collect::<String>();
    internal_page("History", "Recently visited", &rows, script(), theme)
}

pub(super) fn bookmarks_html(bookmarks: &[Bookmark], theme: &str) -> String {
    let rows = bookmarks
        .iter()
        .enumerate()
        .map(|(index, item)| {
            format!(
                r#"<div class="rowwrap"><button class="row" data-url="{}"><span>{}</span><small>{}</small></button><button class="remove" data-remove="{}" title="Remove bookmark">×</button></div>"#,
                escape(&item.url),
                escape(&item.title),
                escape(&item.url),
                index
            )
        })
        .collect::<String>();
    internal_page("Bookmarks", "Your bookmarks", &rows, script(), theme)
}

pub(super) fn settings_html(settings: &Settings) -> String {
    let content = format!(
        r#"<form id="settings">
<label>Homepage URL<input name="homepage" value="{}"></label>
<label>Default search URL <small>Use {{query}} where the search terms go.</small><input name="search_engine" value="{}"></label>
<label>Theme<select name="theme"><option value="dark" {}>Dark</option><option value="light" {}>Light</option></select></label>
<label>Startup behavior<select name="startup"><option value="home" {}>Open RUSK home</option><option value="homepage" {}>Open homepage URL</option></select></label>
<div class="actions"><button class="primary">Save settings</button><button type="button" class="secondary" data-action="clear_history">Clear browsing history</button><button type="button" class="secondary" data-action="reset_settings">Reset settings</button></div>
</form>"#,
        escape(&settings.homepage),
        escape(&settings.search_engine),
        selected(&settings.theme, "dark"),
        selected(&settings.theme, "light"),
        selected(&settings.startup, "home"),
        selected(&settings.startup, "homepage"),
    );
    internal_page(
        "Settings",
        "Browser settings",
        &content,
        script(),
        &settings.theme,
    )
}

fn internal_page(title: &str, heading: &str, content: &str, script: String, theme: &str) -> String {
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{}</title>{}</head><body data-theme="{}"><main class="page"><div class="eyebrow">RUSK BROWSER</div><h1>{}</h1>{}</main><script>{}</script></body></html>"#,
        escape(title),
        content_style(),
        escape(theme),
        escape(heading),
        content,
        script
    )
}

fn card(title: &str, url: &str) -> String {
    format!(
        r#"<button class="site-link" data-url="{}"><span class="site-mark">{}</span><span class="site-title">{}</span><span class="site-url">{}</span></button>"#,
        escape(url),
        escape(
            &title
                .chars()
                .next()
                .unwrap_or('•')
                .to_string()
                .to_uppercase()
        ),
        escape(title),
        escape(&display_host(url))
    )
}

fn empty_card(text: &str) -> String {
    format!(r#"<div class="empty">{}</div>"#, escape(text))
}

fn display_host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}

fn selected(actual: &str, expected: &str) -> &'static str {
    if actual == expected {
        "selected"
    } else {
        ""
    }
}

fn date_label(visited_at: u64) -> String {
    let elapsed = timestamp().saturating_sub(visited_at);
    if elapsed < 60 {
        "Just now".to_owned()
    } else if elapsed < 3600 {
        format!("{} min ago", elapsed / 60)
    } else if elapsed < 86_400 {
        format!("{} hr ago", elapsed / 3600)
    } else {
        format!("{} days ago", elapsed / 86_400)
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn script() -> String {
    r#"const send=(action,value)=>window.ipc.postMessage(JSON.stringify({action,value:value||''}));
document.querySelectorAll('[data-url]').forEach(x=>x.addEventListener('click',()=>send('navigate',x.dataset.url)));
document.querySelectorAll('[data-remove]').forEach(x=>x.addEventListener('click',()=>send('remove_bookmark',x.dataset.remove)));
const search=document.getElementById('search');if(search)search.addEventListener('submit',e=>{e.preventDefault();send('navigate',document.getElementById('q').value)});
const settings=document.getElementById('settings');if(settings)settings.addEventListener('submit',e=>{e.preventDefault();send('save_settings',Object.fromEntries(new FormData(settings)))});
document.querySelectorAll('[data-action]').forEach(x=>x.addEventListener('click',()=>send(x.dataset.action)));
"#.to_owned()
}

fn content_style() -> &'static str {
    r#"<style>
:root{color-scheme:dark;--bg:#202126;--panel:#292a2f;--line:#3a3b41;--text:#e5e5e8;--muted:#a1a2a8;--accent:#9a83d7}
body[data-theme=light]{color-scheme:light;--bg:#f2f2f4;--panel:#fff;--line:#d5d5da;--text:#242429;--muted:#676870;--accent:#7256b4}
*{box-sizing:border-box;font-family:Arial,sans-serif!important}body{margin:0;min-height:100vh;background:var(--bg);color:var(--text);font:13px Arial,sans-serif}
main.home{width:min(560px,calc(100% - 36px));margin:0 auto;padding:clamp(38px,10vh,82px) 0 28px}
.brand{display:flex;align-items:center;gap:9px;margin:0 0 23px}.mark{display:grid;place-items:center;width:27px;height:27px;border:1px solid #514a61;border-radius:4px;background:#302d37;color:#c1b2e8;font-size:15px;font-weight:bold}
.brandname{font-size:16px;font-weight:600}.brandname span{color:var(--muted);font-weight:400}
form#search{display:flex;align-items:center;gap:9px;height:42px;padding:0 9px;border:1px solid var(--line);border-radius:5px;background:var(--panel)}
#search svg{width:16px;height:16px;flex:none;fill:none;stroke:var(--muted);stroke-width:1.7;stroke-linecap:round}
#q{flex:1;min-width:0;height:100%;padding:0;border:0;outline:0;background:transparent;color:var(--text);font-size:14px}#q::placeholder{color:var(--muted)}
#search button,.primary{height:28px;padding:0 11px;border:1px solid #514766;border-radius:4px;background:#393443;color:#e7e0f6;font-size:12px;cursor:pointer}#search button:hover,.primary:hover{background:#474052}
section{margin-top:24px}h2{margin:0 0 7px;color:var(--muted);font-size:11px;font-weight:600;letter-spacing:.35px}
.site-list{border-top:1px solid var(--line)}.site-link{display:flex;align-items:center;gap:10px;width:100%;height:36px;padding:0 7px;border:0;border-bottom:1px solid var(--line);background:transparent;color:var(--text);text-align:left;cursor:pointer}
.site-link:hover,.row:hover{background:var(--panel)}.site-mark{display:grid;place-items:center;width:20px;height:20px;flex:none;border:1px solid var(--line);border-radius:3px;color:var(--accent);font-size:11px}
.site-title{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}.site-url{overflow:hidden;margin-left:auto;color:var(--muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}
.empty{padding:10px 7px;border-bottom:1px solid var(--line);color:var(--muted);font-size:12px}
main.page{width:min(760px,calc(100% - 36px));margin:0 auto;padding:30px 0 40px}.eyebrow{color:var(--accent);font-size:10px;font-weight:bold;letter-spacing:.6px}
h1{margin:6px 0 20px;font-size:21px;font-weight:600}.rowwrap{display:flex;gap:5px;border-top:1px solid var(--line)}
.row{display:flex;align-items:center;justify-content:space-between;gap:12px;width:100%;min-height:38px;padding:6px 8px;border:0;border-bottom:1px solid var(--line);background:transparent;color:var(--text);text-align:left;cursor:pointer}
.row span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}.row small{overflow:hidden;color:var(--muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}
.remove{width:30px;flex:none;border:0;border-bottom:1px solid var(--line);background:transparent;color:var(--muted);font-size:16px;cursor:pointer}.remove:hover{color:var(--text)}
.page label{display:block;margin:0 0 13px;color:var(--text);font-size:12px}.page label small{display:block;margin:3px 0;color:var(--muted);font-size:11px}
.page input,.page select{width:100%;height:32px;margin-top:5px;padding:0 8px;border:1px solid var(--line);border-radius:4px;background:var(--panel);color:var(--text);font-size:12px}
.actions{display:flex;flex-wrap:wrap;gap:7px;margin-top:19px}.actions button{height:30px;padding:0 10px;border-radius:4px;font-size:12px;cursor:pointer}
.secondary{border:1px solid var(--line);background:transparent;color:var(--text)}.secondary:hover{background:var(--panel)}
@media(max-width:520px){main.home{padding-top:30px}.status{max-width:70px}.site-url{max-width:38vw}.menu-panel{width:min(260px,calc(100vw - 16px))}}
</style>"#
}
