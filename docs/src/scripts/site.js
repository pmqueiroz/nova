(()=>{
const $=(s,r=document)=>r.querySelector(s),$$=(s,r=document)=>[...r.querySelectorAll(s)];
const root=document.documentElement;
const qp=new URLSearchParams(location.search);
const saved=(()=>{try{return localStorage.getItem('nova-site-theme')}catch(e){}})();
root.dataset.theme=qp.get('theme')||saved||'dark';
document.addEventListener('click',e=>{
  const t=e.target.closest('[data-theme-toggle]');if(!t)return;
  root.dataset.theme=root.dataset.theme==='light'?'dark':'light';
  try{localStorage.setItem('nova-site-theme',root.dataset.theme)}catch(e){}
});
const plat=((navigator.userAgentData&&navigator.userAgentData.platform)||navigator.platform||navigator.userAgent).toLowerCase();
let os=qp.get('os')||(/mac|iphone|ipad/.test(plat)?'mac':/win/.test(plat)?'win':/linux|x11|cros/.test(plat)?'linux':'mac');
const setOS=o=>{
  os=o;
  $$('[data-os]').forEach(b=>b.setAttribute('aria-selected',b.dataset.os===o));
  $$('[data-os-panel]').forEach(p=>p.hidden=p.dataset.osPanel!==o);
  $$('[data-mod]').forEach(k=>k.textContent=o==='mac'?'⌘':'Ctrl');
  $$('[data-alt]').forEach(k=>k.textContent=o==='mac'?'⌥':'Alt');
};
document.addEventListener('click',e=>{const b=e.target.closest('[data-os]');if(b)setOS(b.dataset.os)});
const okIcon='<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M3 8.5 6.5 12 13 4.5"/></svg>';
document.addEventListener('click',async e=>{
  const b=e.target.closest('[data-copy]');if(!b)return;
  const scope=b.closest('[data-os-panel]')||b.closest('.code')||b.closest('.cmdline')||b.parentElement;
  const src=b.dataset.copy?$(b.dataset.copy):scope.querySelector('pre,.lines');
  let text=src.dataset.copyText||[...src.querySelectorAll('div:not(.note)')].map(d=>d.textContent).join('\n')||src.textContent;
  try{await navigator.clipboard.writeText(text.trim())}catch(err){}
  const html=b.innerHTML;b.classList.add('is-done');b.innerHTML=okIcon+(b.dataset.label!==undefined?'':'<span>Copied</span>');
  setTimeout(()=>{b.classList.remove('is-done');b.innerHTML=html},1600);
});
document.addEventListener('click',e=>{
  const m=e.target.closest('.menu-btn');const nav=$('.mobile-nav');
  if(m&&nav){nav.classList.toggle('open');m.setAttribute('aria-expanded',nav.classList.contains('open'));return}
  if(nav&&e.target.closest('.mobile-nav a'))nav.classList.remove('open');
});
const esc=s=>s.replace(/&/g,'&amp;').replace(/</g,'&lt;');
const hlToml=src=>src.split('\n').map(l=>{
  if(/^\s*#/.test(l))return`<span class="tk-c">${esc(l)}</span>`;
  if(/^\s*\[.*\]\s*$/.test(l))return`<span class="tk-h">${esc(l)}</span>`;
  const m=l.match(/^(\s*)([\w.-]+)(\s*=\s*)(.*)$/);
  if(!m)return esc(l);
  const v=m[4].replace(/("[^"]*")|\b(true|false)\b|(\b\d+(\.\d+)?\b)/g,(x,s,b,n)=>s?`<span class="tk-s">${esc(s)}</span>`:b?`<span class="tk-n">${b}</span>`:`<span class="tk-n">${n}</span>`);
  return`${m[1]}<span class="tk-k">${m[2]}</span><span class="tk-p">${m[3]}</span>${v}`;
}).join('\n');
const hlSh=src=>src.split('\n').map(l=>{
  if(/^\s*#/.test(l))return`<span class="tk-c">${esc(l)}</span>`;
  let first=true;
  return l.split(/(\s+|"[^"]*"|\|)/).map(t=>{
    if(!t)return'';if(/^\s+$/.test(t))return t;
    if(t==='|'){first=true;return'<span class="tk-p">|</span>'}
    if(t.startsWith('"'))return`<span class="tk-s">${esc(t)}</span>`;
    if(first){first=false;if(t==='sudo'){first=true}return`<span class="tk-cmd">${esc(t)}</span>`}
    if(/^-/.test(t))return`<span class="tk-f">${esc(t)}</span>`;
    if(t==='&&'){first=true;return'<span class="tk-p">&amp;&amp;</span>'}
    return esc(t);
  }).join('');
}).join('\n');
$$('pre code[data-lang]').forEach(c=>{
  const raw=c.textContent;c.parentElement.dataset.copyText=raw;
  c.innerHTML=c.dataset.lang==='toml'?hlToml(raw):/sh|powershell|bash/.test(c.dataset.lang)?hlSh(raw):esc(raw);
});
const scrim=$('#search');
if(scrim){
  const I=window.NOVA_SEARCH||[];
  const input=$('.sheet-input',scrim),list=$('.sheet-list',scrim);
  let sel=0,items=[];
  const pageIcon='<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4"><path d="M4 2h5l3 3v9H4z"/><path d="M9 2v3h3"/></svg>';
  const hashIcon='<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"><path d="M6 2 5 14M11 2l-1 12M2.5 5.5h11M2 10.5h11"/></svg>';
  const hl=(s,q)=>{if(!q)return esc(s);const i=s.toLowerCase().indexOf(q);return i<0?esc(s):esc(s.slice(0,i))+'<mark>'+esc(s.slice(i,i+q.length))+'</mark>'+esc(s.slice(i+q.length))};
  const render=()=>{
    const q=input.value.trim().toLowerCase();
    items=[];
    I.forEach(p=>{
      const pm=!q||p.title.toLowerCase().includes(q)||p.group.toLowerCase().includes(q);
      if(pm)items.push({t:p.title,sub:p.group,href:p.href,page:1});
      if(q)p.h.forEach(([h,a])=>{if(h.toLowerCase().includes(q))items.push({t:h,sub:p.title,href:p.href.split('#')[0]+'#'+a})});
    });
    items=items.slice(0,9);sel=Math.min(sel,Math.max(items.length-1,0));
    list.innerHTML=items.length?items.map((it,i)=>`<a class="sheet-item${i===sel?' is-sel':''}" href="${it.href}" data-i="${i}">${it.page?pageIcon:hashIcon}<span class="lbl"><span>${hl(it.t,q)}</span><small>${esc(it.sub)}</small></span>${i===sel?'<span class="hint">↵</span>':''}</a>`).join(''):`<div class="sheet-empty">No results for “${esc(input.value)}”</div>`;
  };
  const open=()=>{scrim.hidden=false;input.value='';sel=0;render();setTimeout(()=>input.focus(),10)};
  const close=()=>{scrim.hidden=true};
  document.addEventListener('click',e=>{if(e.target.closest('[data-search-open]'))open();else if(e.target===scrim||e.target.closest('.sheet-item'))close()});
  list.addEventListener('mousemove',e=>{const a=e.target.closest('[data-i]');if(a&&+a.dataset.i!==sel){sel=+a.dataset.i;render()}});
  input.addEventListener('input',()=>{sel=0;render()});
  document.addEventListener('keydown',e=>{
    if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();scrim.hidden?open():close();return}
    if(e.key==='/'&&scrim.hidden&&!/INPUT|TEXTAREA/.test(document.activeElement.tagName)){e.preventDefault();open();return}
    if(scrim.hidden)return;
    if(e.key==='Escape')close();
    else if(e.key==='ArrowDown'){e.preventDefault();sel=(sel+1)%Math.max(items.length,1);render()}
    else if(e.key==='ArrowUp'){e.preventDefault();sel=(sel-1+items.length)%Math.max(items.length,1);render()}
    else if(e.key==='Enter'&&items[sel]){location.href=items[sel].href;close()}
  });
}
const side=$('.docs-side');
document.addEventListener('click',e=>{
  if(e.target.closest('[data-side-toggle]')&&side){side.classList.toggle('open');document.body.classList.toggle('side-open')}
  else if(side&&side.classList.contains('open')&&(e.target.closest('.docs-side a')||e.target.classList.contains('side-scrim'))){side.classList.remove('open');document.body.classList.remove('side-open')}
});
const toc=$$('.toc a');
if(toc.length&&'IntersectionObserver' in window){
  const map=new Map(toc.map(a=>[a.getAttribute('href').slice(1),a]));
  const vis=new Set();
  const io=new IntersectionObserver(es=>{
    es.forEach(en=>en.isIntersecting?vis.add(en.target.id):vis.delete(en.target.id));
    const first=[...map.keys()].find(id=>vis.has(id));
    if(first){toc.forEach(a=>a.classList.remove('is-active'));map.get(first).classList.add('is-active')}
  },{rootMargin:'-80px 0px -60% 0px'});
  map.forEach((a,id)=>{const el=document.getElementById(id);if(el)io.observe(el)});
}
setOS(os);
window.NovaSite={setOS};
})();
