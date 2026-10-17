// Faction-owned weapon appearance. Game event counters decide when weapons fire.
import {airRole} from './airframes.js';
const STYLES=['ger','peacock','cog','steamer','junk','cruiser','pointer','indiaman','emu','paper'];
const COLORS=['#ffba69','#71d6ff','#f2d899','#7deaff','#ff975e','#ffda82','#72ffe0','#ffd998','#fff0ac','#e399bb'];
const KINDS=['arrow','chakra','bolt','aether','rocket','tracer','pulse','ball','tracer','thorn'];
const GROUND=[
 {default:'arrow',shaman:'aether',trebuchet:'stone'},
 {default:'chakra',archer:'arrow',chariot:'arrow',elephant:'arrow',giant:'stone',sage:'aether',healer:'aether'},
 {default:'bolt',acolyte:'holy',hospitaller:'holy',ballista:'bolt'},
 {default:'aether',gunner:'musket',pioneer:'musket',engine:'ball',ship:'ball'},
 {default:'arrow',rockets:'rocket',junk:'rocket',crossbow:'bolt',signal:'aether'},
 {default:'tracer',tank:'ball',katyusha:'rocket'},
 {default:'pulse',bomb:'glitch',exception:'glitch',bug:'glitch'},
 {default:'musket',carronade:'ball',merchantman:'ball',pinmaker:'needle',shopkeeper:'coin'},
 {default:'boomerang',gunner:'tracer',truck:'tracer'},
 {default:'spore',thorn:'thorn',tree:'seed',statue:'seed'},
];
const CACHE=new WeakMap();
function seatOf(look,u){
 const style=look?.units?.[u?.type]||look?.unit,id=look?.models3d?.[style]?.sculpt;
 return /^\d:/.test(id||'')?Number(id[0]):-1;
}
function recipe(seat,kind){
 const color=COLORS[seat]||'#ffd998',p={weapon:[],mount:[0,-9],muzzle:[10,0],color,core:'#fff8df',glow:3,beam:false,material:'kinetic',
  projectile:[],trail:{color,length:.18,width:1.2,particles:0,wave:0},impact:{color,rings:1,sparks:5}};
 const line=(a,b,c,d,w,col)=>['line',a,b,c,d,w,col],poly=(pts,col)=>['poly',pts,col];
 if(kind==='arrow'||kind==='bolt')p.projectile=[line(-6,0,3,0,.8,'#b99b69'),poly([5,0,1,-1.6,1,1.6],'#ece5cb'),line(-6,-1,-4,1,1,'primary')];
 else if(kind==='chakra')p.projectile=[['circle',0,0,3.2,color],['circle',0,0,1.8,'#4e685c']];
 else if(kind==='tracer'){p.projectile=[line(-5,0,4,0,1.5,'#ffe4a1')];p.trail.length=.10;p.glow=2;}
 else if(kind==='pulse'){p.projectile=[poly([5,0,-2,-2,-4,0,-2,2],color),line(-7,0,-4,0,1,'#e4fff6')];p.trail.particles=2;p.material='energy';}
 else if(kind==='aether'){p.projectile=[['ellipse',0,0,3,1.5,color],['circle',1,0,1,'#ffffff']];p.trail.particles=3;p.material='energy';}
 else if(kind==='holy'){p.projectile=[line(-4,0,4,0,2,'#fff3bd'),line(1,-3,1,3,1.5,color)];p.trail.particles=2;p.material='energy';}
 else if(kind==='glitch'){p.projectile=[['rect',-4,-2,4,3,color],['rect',1,0,3,2,'#e5fff7'],['rect',-1,-4,2,2,'#9ec9ff']];p.trail.wave=1.3;p.trail.particles=3;p.material='energy';}
 else if(kind==='stone'){p.projectile=[poly([-4,-1,-2,-4,2,-3,4,1,1,3,-3,2],'#9d927c'),poly([-4,-1,-2,-4,2,-3,0,0],'#d0c4a6')];p.trail.color='#b8a585';p.glow=0;p.material='explosive';p.impact={color:'#c8b98f',rings:1,sparks:7};}
 else if(kind==='musket'){p.projectile=[line(-4,0,1,0,1,'#e9c28c'),['circle',1,0,1.3,'#cfc7ae']];p.trail.color='#c9bc9d';p.trail.length=.10;p.glow=1;}
 else if(kind==='spore'){p.projectile=[['circle',0,0,2.5,'#dfa6cc'],['circle',-3,2,1.1,'#b3d78f'],['circle',-4,-2,.9,'#e6c2ad']];p.trail.particles=4;p.trail.wave=1.5;p.impact.color='#c9b6d6';}
 else if(kind==='seed'){p.projectile=[['ellipse',0,0,3.5,1.8,'#a3bd6d'],poly([-1,0,-5,-4,-4,0,-6,2],'#e3afc0')];p.trail.color='#a5c47c';}
 else if(kind==='boomerang'){p.projectile=[poly([-4,-4,1,-1,4,4,1,3,-1,0,-5,-2],'#c59155')];p.trail.length=.1;p.glow=0;}
 else if(kind==='coin'){p.projectile=[['circle',0,0,3,'#e7bd59'],['circle',0,0,1.7,'#9b7034']];p.glow=1;}
 else if(kind==='needle'){p.projectile=[line(-5,0,5,0,.8,'#e6e6dc'),['circle',-4,0,1,'#bcb1a4']];p.glow=0;p.trail.length=.08;}
 else if(kind==='thorn')p.projectile=[poly([6,0,-3,-2,-1,0,-3,2],'#d6a0a8'),line(-4,0,1,0,1,'#627d40')];
 else if(kind==='rocket'){p.projectile=[['rect',-3,-1.3,6,2.6,'#ab4536'],poly([5,0,2,-1.3,2,1.3],'#e8c891'),poly([-3,0,-8,-2,-7,0,-8,2],'#ffc45c')];p.trail.color='#bcb7a5';p.trail.particles=3;p.trail.length=.28;p.material='explosive';}
 else if(kind==='bomb'){p.projectile=seat===9?[['ellipse',0,0,4,2.2,'#748b4f'],poly([-2,0,-5,-3,-4,0,-5,3],'#d9a4c4')]:[['ellipse',0,0,4,2.2,seat===3?'#bc9a59':'#4d5350'],line(-3,-3,-3,3,1.2,color)];p.trail.length=.08;p.trail.width=.8;p.material='explosive';p.impact={color,rings:2,sparks:8};}
 else if(kind==='torpedo'){p.projectile=[['ellipse',0,0,5,1.5,'#c9d6cf'],poly([5,0,2,-1.5,2,1.5],color),line(-4,-2,-4,2,.8,'#e2eeea')];p.trail.color='#deeff5';p.glow=0;}
 else if(kind==='slash'){p.projectile=[poly([0,-5,3,-1,4,2,0,5,1,0],'#fff2d2')];p.trail.length=.09;p.impact.rings=0;}
 else p.projectile=[['circle',0,0,2.1,'#4a4943'],['circle',-.6,-.6,.8,'#c9c6b4']];
 // Physical ammunition stays readable as a shape, without an energy-beam halo or long neon tail.
 if(['arrow','bolt','chakra','ball','bomb','thorn','stone','seed','boomerang','coin','needle'].includes(kind)){
  p.glow=0;p.trail.length=.035;p.trail.width=.55;p.trail.color=['arrow','bolt'].includes(kind)?'#b99b69':kind==='thorn'||kind==='seed'?'#8fa774':'#b9b8a8';
 }
 if(kind==='ball')p.material='explosive';
 if(kind==='spore'){p.glow=1;p.trail.length=.09;p.trail.width=.6;}
 if(kind==='rocket')p.glow=1;
 return p;
}
function retained(look,key,make){let entries=CACHE.get(look);if(!entries){entries=new Map();CACHE.set(look,entries);}if(!entries.has(key))entries.set(key,make());return entries.get(key);}

/** Shape and timing only: firing is always gated by an authoritative shot counter in rts-effects.js. */
export function defaultUnitWeaponEffect(look,u){
 const seat=seatOf(look,u);
 if(u.build?.[6]&&seat>=0){const role=airRole(u.build);return retained(look,'air:'+role,()=>{
  const kind=role==='striker'?(seat===1?'chakra':seat===4?'rocket':'bomb'):(seat===0||seat===2?'slash':KINDS[seat]);
  const flight=kind==='bomb'?520:kind==='slash'?120:260;
  return {kind,profile:recipe(seat,kind),flight,duration:flight+380,arc:kind==='bomb'?-12:kind==='rocket'?10:0,native:true};
 });}
 if(u.rts?.sea?.ship){const profile=navalWeaponProfile(look,'gun');return {kind:'shell',profile,flight:360,duration:780,arc:9,native:true};}
 if(seat>=0){const style=look?.units?.[u.type]||look?.unit,body=look.models3d[style].sculpt.split(':')[1];
  return retained(look,'ground:'+body,()=>{const kind=GROUND[seat][body]||GROUND[seat].default;
   const arc=['arrow','bolt','chakra','stone','spore','seed','boomerang','coin'].includes(kind)?12:kind==='rocket'?7:0;
   const flight=['tracer','musket','needle','pulse','glitch'].includes(kind)?170:kind==='stone'?430:290;
   return {kind,profile:recipe(seat,kind),flight,duration:flight+330,arc,native:true,ground:true};
  });}
 return null;
}
export function navalWeaponProfile(look,kind='gun'){
 if(!look)return recipe(-1,kind==='torpedo'?'torpedo':'ball');
 return retained(look,'sea:'+kind,()=>{const seat=STYLES.indexOf(look.ship);return recipe(seat,kind==='torpedo'?'torpedo':seat===4?'rocket':seat===6?'pulse':seat===9?'thorn':'ball');});
}
