(()=>{const el=document.getElementById('typed');if(!el)return;const cmds=['nova --version','cargo test','git status'];
if(matchMedia('(prefers-reduced-motion: reduce)').matches){el.textContent=cmds[0];return}
let c=0,i=0,del=false;const tick=()=>{const s=cmds[c];
if(!del){el.textContent=s.slice(0,++i);if(i===s.length){del=true;return setTimeout(tick,2400)}}
else{el.textContent=s.slice(0,--i);if(i===0){del=false;c=(c+1)%cmds.length;return setTimeout(tick,400)}}
setTimeout(tick,del?30:70+Math.random()*60)};setTimeout(tick,1400)})();
