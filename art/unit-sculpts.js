import { Sculpt } from './sculpt.js';
import { airSculpt } from './air-sculpts.js';
import { navalSculpt } from './naval-sculpts.js';
export { navalSculpt } from './naval-sculpts.js';
// House-authored RTS miniatures. Generated once per look/role, retained by GPUUnits.
// +x is forward, +y is up, z separates the left/right limbs. No textures or per-frame JS rigs.
const P='primary',S='secondary',A='accent',IRON='#788b95',DARK='#26333a',WOOD='#785239',GOLD='#dab866',IVORY='#e6dcc2',LIGHT='#a3fff0',RED='#b93b3c';
export const HOUSE_SCULPTS=[
 ['archer','keshig','horseman','artisan','falconer','shaman','trebuchet','camel'],
 ['chariot','archer','servant','elephant','mace','sage','giant','healer'],
 ['crusader','acolyte','sergeant','crossbowman','turcopole','marshal','ballista','hospitaller'],
 ['engine','pioneer','fellow','automaton','gunner','ship','mystery','flyer'],
 ['marine','signal','rockets','junk','porter','qilin','crossbow','chronicler'],
 ['tank','katyusha','agitator','stakhanovite','soldier','cavalry','sniper','mole'],
 ['crash','watch','bomb','collector','zero','pointer','exception','bug'],
 ['highlander','dragoon','carronade','pinmaker','shopkeeper','merchantman','exciseman','smuggler'],
 ['truck','emu','forager','gunner'],['gardener','tree','eye','thorn','statue'],
];
export function hasSculpt(id){if(typeof id!=='string'||!/^\d:[A-Za-z_][A-Za-z0-9_]{0,23}$/.test(id))return false;const [seat,body]=id.split(':');return HOUSE_SCULPTS[Number(seat)].includes(body);}

function human(s,{coat=P,skin='#d6a17e',legs=DARK,hat='cap',trim=S,armor=false,robe=false,lean=0,frame='common',mounted=false,detail=true}={}){
 const start=s.parts.length;
 // Every frame owns its torso and head.  The shared tool sockets deliberately stay
 // put so a role can change equipment without turning each faction into the same pawn.
 if(frame==='null'){
  for(const z of [-.31,.31]){const m=z<0?'strideL':'strideR';s.taper([-.14,.78,z],[.18,.10,z],.060,LIGHT,m,[-.14,.78,z]);s.box([.26,.08,z],[.42,.10,.18],IRON,m,[-.14,.78,z]);s.orb([-.02,.45,z],[.15,.15,.15],LIGHT,m,[-.14,.78,z]);}
  const legEnd=s.parts.length;s.tag(start,{joint:'legs'});
  s.lathe([lean,.62,0],[[0,.13,.14],[.10,.34,.28],[.52,.25,.22],[.67,.10,.10]],DARK);
  s.part('cylinder',[lean+.03,1.38,0],[.48,.48,.22],IRON,'',[lean+.03,1.38,0],[Math.PI/2,0,0]);
  s.box([lean+.20,1.39,0],[.05,.08,.30],LIGHT);for(const z of [-.36,.36])s.taper([lean,1.04,z],[lean+.42,.83,z],.052,IRON,z<0?'strideR':'strideL',[lean,1.04,z]);
  return start;
 }
 for(const z of [-.16,.16]){
  const motion=z<0?'strideL':'strideR',pivot=[0,.65,z];
  // Separate thigh, knee and calf give every walking unit a readable gait from the
  // strategy camera without adding a skinned mesh or a per-frame rig.
  s.capsule([-.04,.51,z],[.20,.34,.20],legs,motion,pivot);
  const knee=z<0?'kneeL':'kneeR';s.orb([.02,.33,z],[.17,.14,.17],legs,knee,pivot);
  s.capsule([.06,.20,z],[.16,.29,.17],legs,knee,pivot);
  s.poly([.15,.075,z],[-.18,-.06,.20,-.06,.30,.045,.08,.10,-.22,.06],.23,DARK,knee,pivot);
 }
 const legEnd=s.parts.length;
 let customHead=false;
 if(frame==='steppe'){
  s.lathe([lean,.38,0],[[0,.34,.31],[.13,.48,.43],[.53,.48,.41],[.78,.29,.27]],coat);s.orb([lean-.10,1.16,0],[.62,.20,.54],'#493e34');
 }else if(frame==='arjuna'){
  s.lathe([lean,.10,0],[[0,.25,.23],[.12,.32,.29],[.61,.22,.20],[1.05,.13,.12]],coat);s.box([lean,1.14,0],[.38,.040,.34],GOLD);
  s.orb([lean+.03,1.42,0],[.23,.28,.22],skin);s.lathe([lean,1.60,0],[[0,.17,.16],[.08,.24,.21],[.31,.035,.035]],GOLD);for(const z of [-.20,0,.20])s.poly([lean-.08,1.63,z],[-.12,0,.12,0,0,.20],.022,GOLD);customHead=true;
 }else if(frame==='templar'){
  s.lathe([lean,.39,0],[[0,.18,.16],[.28,.29,.25],[.57,.36,.30],[.76,.22,.20]],IVORY);for(const z of [-.27,.27])s.orb([lean,1.10,z],[.25,.17,.20],IRON);
  s.capsule([lean+.03,1.43,0],[.37,.42,.34],IRON);s.box([lean+.25,1.42,0],[.025,.12,.20],DARK);customHead=true;
 }else if(frame==='atlantis'){
  s.lathe([lean,.22,0],[[0,.43,.37],[.12,.57,.49],[.48,.57,.49],[.69,.40,.35]],GOLD);s.box([lean,.80,0],[.74,.045,.62],DARK);
  for(const z of [-.39,.39]){s.orb([lean-.20,.88,z],[.24,.44,.20],IRON);s.rod([lean-.15,1.11,z],[lean+.02,1.20,z],.045,IRON);}s.orb([lean+.03,1.44,0],[.55,.52,.50],'#b89a63');s.orb([lean+.39,1.46,0],[.16,.23,.25],LIGHT);customHead=true;
 }else if(frame==='fleet'){
  s.lathe([lean,.24,0],[[0,.20,.18],[.10,.27,.23],[.78,.24,.21],[.95,.17,.15]],coat);s.box([lean-.25,.77,.28],[.15,.45,.16],WOOD);
  s.orb([lean+.02,1.40,0],[.26,.28,.24],coat);s.lathe([lean,1.52,0],[[0,.43,.39],[.05,.52,.46],[.12,.39,.34],[.17,.12,.10]],'#c5a86c');customHead=true;
 }else if(frame==='industrial'){
  s.lathe([lean,.15,0],[[0,.25,.22],[.10,.31,.27],[.86,.31,.27],[1.03,.23,.20]],coat);s.box([lean,1.07,0],[.70,.10,.52],IRON);s.box([lean-.27,.75,.30],[.20,.52,.16],DARK);s.lathe([lean,1.55,0],[[0,.26,.23],[.06,.30,.26],[.13,.25,.22]],DARK);customHead=true;
 }else if(frame==='nations'){
  s.lathe([lean,.18,0],[[0,.20,.18],[.15,.33,.28],[.53,.24,.21],[.85,.18,.16]],coat);s.lathe([lean-.20,.38,0],[[0,.14,.12],[.30,.28,.23],[.56,.14,.12]],coat);s.box([lean,.75,0],[.43,.045,.35],trim);
  s.capsule([lean+.03,1.39,0],[.22,.27,.21],skin);s.lathe([lean,1.55,0],[[0,.48,.28],[.05,.56,.34],[.11,.36,.22],[.17,.10,.07]],DARK);customHead=true;
 }else if(frame==='garden'){
  s.lathe([lean,.08,0],[[0,.31,.27],[.18,.25,.21],[.80,.18,.16],[1.05,.07,.06]],'#72523b');s.orb([lean,1.31,0],[.65,.42,.58],'#91aa75');
  for(const z of [-.38,.38])s.poly([lean,1.10,z],[-.30,0,.09,.10,.35,.03,.08,-.14],.028,'#d3a7a4','sway',[lean,.98,z]);customHead=true;
 }else {
  s.capsule([lean,.99,0],[.52,.38,.46],coat,'',[lean,.92,0]);s.capsule([lean,.73,0],[.36,.34,.34],coat,'',[lean,.92,0]);
  if(robe)s.lathe([lean,.18,0],[[0,.32,.29],[.12,.38,.34],[.64,.25,.23]],coat);s.box([lean,.80,0],[.47,.052,.51],trim);
 }
 if(!customHead){s.capsule([lean+.03,1.40,0],[.29,.34,.28],skin);s.box([lean+.193,1.40,0],[.075,.09,.11],skin);s.box([lean+.174,1.46,0],[.035,.035,.24],DARK);}
 for(const z of [-.31,.31]){
  const m=z<0?'strideR':'strideL',pivot=[lean,1.15,z];
  if(frame==='garden'){s.taper([lean,1.12,z],[lean+.36,.72,z*1.25],.055,'#587450','sway',pivot);s.poly([lean+.39,.69,z*1.28],[-.12,0,.12,0,.20,.07,.04,.12,-.13,.06],.024,'#91aa75','sway',pivot);}else{
   s.taper([lean,1.15,z],[.17,.84,z],frame==='fleet'?.09:.135,coat,m,pivot);s.orb([.18,.84,z],[.15,.16,.16],skin,m,pivot);
   // A cuff and two fingers give a weapon hand a readable grip at inspection
   // scale; they are omitted from the overview LOD with the other close detail.
   const hand=s.parts.length;s.box([.14,.88,z],[.12,.07,.19],armor?IRON:trim,m,pivot);
   for(const dz of [-.045,.045])s.box([.28,.84,z+dz],[.10,.035,.025],skin,m,pivot);
   s.tag(hand,{material:armor?'iron':'cloth',detail:2});
  }
 }
 if(!customHead)headwear(s,hat,lean,trim);
 // Belts, collar plates, and a visible nose make the shared frame read as a
 // clothed person at close zoom. They are mid-detail so distant armies stay clean.
 if(detail&&frame!=='null'){
  const ornament=s.parts.length;
  s.box([lean,.88,0],[.57,.055,.49],trim);s.box([lean+.22,.89,-.01],[.065,.10,.08],GOLD);
  s.rod([lean-.20,1.20,-.25],[lean+.22,.82,-.25],.027,trim);s.box([lean+.235,1.41,0],[.055,.07,.08],skin);
  if(armor){for(const z of [-.28,.28])s.box([lean+.03,1.08,z],[.30,.18,.055],IRON);}
  s.tag(ornament,{material:armor?'iron':'cloth',detail:1});
 }
 if(!mounted)s.tag(start,{joint:'legs'}); else s.tag(start,{joint:'legs'});
 // Undo the broad tag immediately after the leg segment.  The rider's torso, head,
 // and equipment must remain independent from the walking leg joint.
 for(const p of s.parts.slice(legEnd))if(p.joint==='legs')delete p.joint;
 return start;
}
function headwear(s,hat,x=0,trim=S){
 if(hat==='none')return;
 if(hat==='hood'){s.orb([x,1.46,-.04],[.39,.44,.38],P);s.box([x+.20,1.40,0],[.04,.20,.23],DARK);}
 else if(hat==='greathelm'){s.capsule([x+.01,1.48,0],[.40,.42,.37],IRON);s.poly([x+.16,1.49,0],[-.06,-.14,.06,-.14,.09,.14,-.09,.14],.39,DARK);s.box([x+.215,1.42,0],[.035,.18,.035],GOLD);}
 else if(hat==='conical'){s.capsule([x,1.53,0],[.36,.18,.35],IRON);s.poly([x,1.58,0],[-.25,0,.25,0,.03,.37,-.05,.37],.39,IRON);s.box([x,1.52,0],[.46,.065,.43],trim);}
 else if(hat==='turban'){s.orb([x,1.58,0],[.43,.25,.41],IVORY);s.box([x+.01,1.61,0],[.39,.055,.43],trim);s.poly([x-.12,1.62,0],[-.10,0,.12,0,0,.22],.10,GOLD);}
 else if(hat==='tricorn'){s.poly([x,1.57,0],[-.32,0,.32,0,.10,.20,-.20,.20],.52,DARK);s.box([x,1.56,0],[.66,.045,.57],trim);}
 else if(hat==='fur'){s.orb([x,1.56,0],[.43,.20,.43],'#493e34');s.poly([x,1.62,0],[-.18,0,.18,0,0,.23],.29,P);}
 else if(hat==='straw'){s.poly([x,1.52,0],[-.40,0,.40,0,0,.28],.64,'#c5a86c');}
 else if(hat==='crown'){s.box([x,1.58,0],[.38,.12,.35],GOLD);for(const z of [-.15,.15])s.poly([x,1.64,z],[-.2,0,.2,0,.17,.20,0,.08,-.17,.20],.05,GOLD);}
 else {s.orb([x,1.56,0],[.39,.20,.37],hat==='helmet'?IRON:hat==='beret'?P:DARK);s.box([x+.21,1.55,0],[.19,.045,.32],hat==='helmet'?IRON:DARK);}
}
function tool(s,kind,{x=.25,y=.9,z=.37}={}){
 const first=s.parts.length;
 if(['bow','crossbow','rifle','musket'].includes(kind)){s.muzzle=[x+(kind==='bow'?.44:kind==='crossbow'?.45:.79),y+(kind==='rifle'||kind==='musket'?.035:0),z];s.muzzleJoint='weapon';s.muzzlePivot=[x,y,z];}
 if(kind==='bow'){
  s.poly([x+.25,y,z],[0,-.43,.14,-.27,.19,0,.14,.28,0,.44,.06,.20,.08,0,.04,-.20],.065,WOOD);
  s.rod([x+.25,y-.43,z],[x+.25,y+.44,z],.022,IVORY);
  s.box([-.30,1.13,-z],[.16,.53,.18],'#594332');
 }
 else if(kind==='crossbow'){s.box([x+.1,y,z],[.70,.09,.10],WOOD);s.rod([x+.35,y,z-.32],[x+.35,y,z+.32],.07,IRON);s.box([x-.12,y-.10,z],[.16,.20,.07],WOOD);s.box([x+.18,y-.10,z],[.055,.18,.045],IRON);}
 else if(kind==='rifle'||kind==='musket'){s.box([x,y,z],[.43,.13,.12],WOOD);s.box([x+.44,y+.035,z],[.70,.055,.06],IRON);s.box([x-.17,y-.13,z],[.10,.25,.075],WOOD);s.box([x+.18,y-.08,z],[.055,.20,.035],IRON);if(kind==='rifle')s.box([x+.17,y+.13,z],[.26,.055,.07],DARK);for(const p of s.parts.slice(first))p.motion='recoil';}
 else if(kind==='spear'||kind==='staff'){s.rod([x,y-.62,z],[x+.24,y+.90,z],.055,WOOD);s.poly([x+.24,y+.90,z],[-.07,-.02,.07,-.02,0,.28],.065,IRON);s.box([x+.02,y+.05,z],[.13,.045,.10],GOLD);}
 else if(kind==='sword'){s.box([x+.32,y+.20,z],[.065,.70,.045],IRON,'strike',[x,y,z],[0,0,-.4]);s.box([x+.19,y-.10,z],[.29,.055,.065],GOLD,'strike',[x,y,z]);}
 else if(kind==='pick'){s.rod([x,y-.45,z],[x+.27,y+.26,z],.055,WOOD,'strike',[x,y,z]);s.rod([x+.01,y+.20,z],[x+.51,y+.35,z],.075,IRON,'strike',[x,y,z]);}
 else if(kind==='hammer'){s.rod([x,y-.22,z],[x+.16,y+.40,z],.07,WOOD,'strike',[x,y,z]);s.box([x+.16,y+.38,z],[.32,.20,.23],IRON,'strike',[x,y,z]);}
 else if(kind==='mace'){s.rod([x,y-.20,z],[x+.16,y+.46,z],.075,WOOD,'strike',[x,y,z]);s.orb([x+.16,y+.48,z],[.31,.30,.31],GOLD,'strike',[x,y,z]);}
 else if(kind==='book'){s.box([x+.10,y,z],[.30,.08,.39],WOOD);s.box([x+.10,y+.04,z],[.25,.045,.33],IVORY);}
 else if(kind==='lantern'){s.rod([x,y,z],[x,y+.5,z],.045,DARK);s.box([x,y-.12,z],[.22,.30,.22],GOLD,'sway',[x,y+.3,z]);s.box([x+.12,y-.10,z],[.025,.18,.13],'#fff1b4','sway',[x,y+.3,z]);}
 const ranged=['bow','crossbow','rifle','musket'].includes(kind);
 // Every piece of a weapon turns as one rigid assembly. Its socket uses the
 // same pivot, so the muzzle and visible barrel cannot separate while aiming.
 s.tag(first,{detail:0,aim:ranged?'weapon':undefined,pivot:ranged?[x,y,z]:undefined});
}
function shield(s,c=P,cross=false){
 s.poly([.10,.92,-.46],[-.27,.25,.27,.25,.23,-.13,0,-.35,-.23,-.13],.10,c);
 if(cross){s.box([.11,.94,-.52],[.065,.40,.02],'#ae303d');s.box([.11,1,-.53],[.30,.065,.02],'#ae303d');}
}
function banner(s,c=P,x=-.20,height=2.1){s.rod([x,.65,-.35],[x,height,-.35],.035,WOOD);s.box([x+.27,height-.19,-.35],[.55,.34,.04],c,'sway',[x,height,-.35]);}
function animal(s,{coat='#926044',camel=false,elephant=false,compact=false}={}){
 const start=s.parts.length;
 const h=elephant?.9:.65,L=elephant?1.45:1.25;
 (compact?s.capsule.bind(s):s.orb.bind(s))([-.05,h,0],[L,elephant?.90:.55,elephant?.80:.52],coat);
 for(const x of [-.45,.43])for(const z of [-.23,.23]){
  const motion=(x*z>0)?'strideL':'strideR',pivot=[x,h-.1,z];
  s.capsule([x,h/2-.05,z],[elephant?.22:.14,h-.10,elephant?.24:.15],coat,motion,pivot);
  if(!compact)s.orb([x+.03,.14,z],[elephant?.22:.16,.16,elephant?.23:.17],coat,motion,pivot);
  s.poly([x+.13,.055,z],[-.15,-.05,.22,-.05,.30,.05,.06,.10,-.19,.06],elephant?.27:.15,elephant?coat:DARK,motion,pivot);
 }
 s.taper([.45,h+.10,0],[.65,h+(elephant?.16:.65),0],elephant?.5:.24,coat);
 (compact?s.capsule.bind(s):s.orb.bind(s))([.80,h+(elephant?.12:.61),0],[elephant?.64:.46,elephant?.62:.28,elephant?.65:.30],coat);
 s.rod([-.62,h+.1,0],[-.82,h-.25,0],.08,DARK,'sway',[-.62,h+.1,0]);
 if(camel){s.orb([-.24,h+.34,0],[.5,.45,.4],coat);s.orb([.19,h+.32,0],[.43,.42,.4],coat);}
 if(elephant){s.orb([.65,h+.15,.36],[.12,.59,.35],coat);s.orb([.65,h+.15,-.36],[.12,.59,.35],coat);s.taper([1.02,h+.1,0],[1.12,.18,0],.19,coat);for(const z of [-.23,.23])s.taper([1,h+.35,z],[1.35,h+.28,z],.07,IVORY);}
 else {s.box([.79,h+.66,0],[.33,.06,.36],WOOD);s.box([.42,h+.28,0],[.55,.065,.57],P);}
 // Saddle blanket, cinch, bridle and reins make cavalry legible as cavalry.
 s.tag(start,{material:'organic'});const tack=s.parts.length;s.box([-.02,h+.48,0],[.69,.06,.61],P);s.box([-.02,h+.31,0],[.075,.35,.64],WOOD);s.rod([.80,h+.55,-.18],[.18,h+.48,-.25],.022,WOOD);s.rod([.80,h+.55,.18],[.18,h+.48,.25],.022,WOOD);s.tag(tack,{material:'leather',detail:1});
}
function mounted(s,{coat,camel=false,rider={},weapon='spear',standard=false}={}){
 animal(s,{coat,camel});
 const start=human(s,{...rider,mounted:true});s.transform(start,.74,[-.17,.72,0]);
 tool(s,weapon,{x:.05,y:1.30,z:.28});if(standard)banner(s,P,-.34,2.20);
}
function cart(s,{tank=false,truck=false,cannon=false,rockets=false,culture=0}={}){
 const c=tank?'#60704c':culture===3?'#947855':WOOD;
 const turret=(pivot,build,muzzle)=>{const first=s.parts.length;build();s.tag(first,{joint:'turret',pivot});if(muzzle){s.muzzle=muzzle;s.muzzleJoint='turret';s.muzzlePivot=[...pivot];}};
 if(tank){
  const start=s.parts.length;for(const z of [-.45,.45]){s.box([0,.28,z],[1.65,.43,.28],DARK);for(const x of [-.60,-.20,.20,.60]){s.wheel([x,.27,z+(z<0?-.15:.15)],.15,IRON);s.part('cylinder',[x,.27,z+(z<0?-.31:.31)],[.095,.095,.035],GOLD,'roll',[0,.27,z],[Math.PI/2,0,0]);}for(const x of [-.61,-.40,-.19,.02,.23,.44,.65])s.box([x,.14,z+(z<0?-.31:.31)],[.11,.09,.06],IRON,'roll',[0,.27,z]);s.rod([-.64,.33,z],[.64,.33,z],.045,IRON,'roll',[0,.27,z]);}
  s.poly([0,.42,0],[-.83,0,.83,0,.57,.38,-.64,.38],.85,c);s.tag(start,{material:'iron'});
  turret([-.12,.89,0],()=>{const detail=s.parts.length;s.poly([-.1,.79,0],[-.39,0,.44,0,.27,.28,-.25,.28],.64,c);s.box([.67,.99,0],[1.05,.09,.11],IRON,'recoil',[-.12,.89,0]);s.box([-.12,1.10,0],[.30,.035,.29],DARK);s.box([.04,.89,-.33],[.16,.12,.02],P);s.box([-.13,.91,.34],[.24,.14,.04],DARK);s.part('cylinder',[-.24,1.11,0],[.20,.20,.035],IRON,'',[-.12,.89,0],[Math.PI/2,0,0]);s.box([-.25,1.15,0],[.13,.035,.04],GOLD);s.tag(detail,{material:'iron',detail:1});},[1.19,1.03,0]);return;
 }
 s.box([0,.30,0],[1.48,.16,.80],c);
 for(const x of [-.48,.48])for(const z of [-.47,.47])s.wheel([x,.22,z],.22,DARK);
 if(truck||rockets){s.box([.42,.67,0],[.56,.56,.65],culture===8?'#8c8a5b':'#68754d');s.box([.72,.74,0],[.025,.22,.55],'#98c7cf');s.box([-.30,.48,0],[.81,.17,.72],c);}
 if(rockets)turret([-.39,.70,0],()=>{for(const z of [-.23,0,.23]){s.rod([-.73,.72,z],[.14,1.05,z],.10,IRON);s.rod([-.75,.86,z],[.12,1.19,z],.10,IRON);}s.box([-.39,.70,0],[.12,.43,.72],DARK);},[.22,1.20,0]);
 if(truck&&culture===8)turret([-.3,1.02,0],()=>{s.rod([-.3,.5,0],[-.3,1.02,0],.07,IRON);s.box([.01,1.04,0],[.95,.075,.09],DARK);s.part('cylinder',[-.12,1.12,0],[.27,.27,.05],DARK,'recoil',[-.3,1.02,0],[Math.PI/2,0,0]);},[.36,1.12,0]);
 if(cannon)turret([-.30,.55,0],()=>{s.part('cylinder',[.03,.72,0],[.39,.39,1.10],DARK,'recoil',[-.30,.55,0],[0,Math.PI/2,0],.08);s.box([-.30,.55,0],[.37,.16,.63],GOLD);},[.58,.72,0]);
}
function emu(s,{forager=false,gunner=false}={}){
 const coat=forager?'#9c8b69':'#786c55';
 s.orb([-.10,.78,0],[.85,.78,.61],coat);
 for(const z of [-.18,.18]){const m=z<0?'strideL':'strideR',pivot=[0,.60,z];s.taper([0,.60,z],[.13,.21,z],.052,'#514a38',m,pivot);s.orb([.13,.21,z],[.09,.11,.09],'#514a38',m,pivot);s.taper([.13,.21,z],[.03,.05,z],.043,'#514a38',m,pivot);s.poly([.17,.04,z],[-.17,-.04,.20,-.04,.27,.04,.06,.08,-.20,.05],.15,DARK,m,pivot);}
 s.taper([.13,1.05,0],[.30,1.67,0],.14,coat,'bob');s.orb([.35,1.70,0],[.31,.28,.25],DARK,'bob');s.poly([.56,1.65,0],[-.10,-.04,.11,-.04,.16,.04,.02,.07,-.14,.04],.13,'#bca77c','bob');s.orb([.43,1.75,.13],[.055,.055,.025],'#efb94b');
 for(const z of [-.30,.30])s.orb([-.22,.87,z],[.56,.48,.12],coat,'sway',[-.11,1.06,z]);
 for(const z of [-1,1])s.poly([-.22,.92,z*.13],[-.39,0,.20,.06,.42,.20,-.12,.16],.025,z<0?'#a69570':'#695f4d','sway',[-.11,1.06,z]);
 s.box([-.07,1.02,0],[.28,.11,.67],P);
 if(forager){s.box([-.37,.72,.38],[.30,.35,.20],WOOD);tool(s,'pick',{x:.28,y:.75,z:.35});}
 if(gunner){s.box([.16,.94,.39],[.87,.10,.11],WOOD);s.box([.79,.96,.39],[.56,.06,.065],IRON);s.rod([-.2,.9,0],[.30,.94,.39],.07,DARK);s.part('cylinder',[.34,1.04,.39],[.23,.23,.05],DARK,'recoil',[.34,1.04,.39],[Math.PI/2,0,0]);}
}
function kublai(s,body){
 if(['archer','keshig','horseman','camel'].includes(body)){
  mounted(s,{coat:body==='keshig'?'#513c30':body==='camel'?'#b39a65':'#986646',camel:body==='camel',rider:{coat:body==='keshig'?IRON:P,hat:body==='keshig'?'conical':'fur',armor:body==='keshig',frame:'steppe'},weapon:body==='archer'?'bow':body==='horseman'?'sword':body==='camel'?'staff':'spear',standard:body==='keshig'});
  if(body==='camel'){for(const z of [-.40,.40])s.box([-.25,.80,z],[.45,.42,.22],'#a27b50');}return;
 }
 if(body==='trebuchet'){
  cart(s);for(const z of [-.29,.29]){s.rod([-.51,.38,z],[0,1.39,z],.11,WOOD);s.rod([.55,.38,z],[0,1.39,z],.11,WOOD);}
  s.rod([-.40,1.03,0],[.65,1.85,0],.11,WOOD,'strike',[0,1.39,0]);s.box([-.40,.96,0],[.36,.43,.37],'#8b8b7d','strike',[0,1.39,0]);s.rod([.65,1.85,0],[.94,1.25,0],.025,IVORY,'strike',[0,1.39,0]);banner(s,P,-.54,1.3);return;
 }
 human(s,{coat:body==='shaman'?'#354b69':body==='artisan'?'#8b7860':P,hat:'fur',robe:body==='shaman',frame:'steppe'});
 if(body==='artisan')tool(s,'pick');
 if(body==='falconer'){s.rod([.07,1.14,.31],[.52,1.23,.31],.14,WOOD);s.orb([.52,1.36,.31],[.25,.30,.20],'#654d37');s.poly([.48,1.40,.31],[-.19,0,.22,0,.11,.13],.15,IVORY);s.box([.68,1.46,.31],[.1,.05,.07],GOLD);}
 if(body==='shaman'){s.part('cylinder',[.37,.98,.44],[.43,.43,.15],IVORY);s.part('cylinder',[.37,.98,.42],[.48,.48,.07],WOOD);tool(s,'staff',{x:-.25,y:.9,z:-.37});}
}
function arjuna(s,body){
 if(body==='chariot'){
  animal(s,{coat:'#e9e1c9',compact:true});const start=s.parts.length;animal(s,{coat:'#d5cdbb',compact:true});s.transform(start,.82,[.15,0,-.47]);
  const a=s.parts.length;human(s,{coat:P,hat:'crown',trim:GOLD,skin:'#b77f59',detail:false});s.transform(a,.67,[-.95,.54,0]);
  s.box([-.94,.43,0],[.78,.17,.86],GOLD);for(const z of [-.48,.48]){s.wheel([-.98,.28,z],.30,GOLD);s.poly([-1.15,.51,z],[-.18,0,.52,0,.43,.45,-.18,.34],.07,P);}
  s.rod([-.80,.48,0],[.50,.60,0],.055,GOLD);tool(s,'bow',{x:-.6,y:1.40,z:.35});banner(s,S,-1.23,2.10);return;
 }
 if(body==='elephant'){
  animal(s,{coat:'#7e898d',elephant:true});s.box([-.05,1.40,0],[.98,.08,.94],P);s.box([-.10,1.62,0],[.65,.35,.66],GOLD);s.box([-.10,1.80,0],[.56,.04,.56],P);const a=s.parts.length;human(s,{hat:'turban',coat:P,trim:GOLD,skin:'#b77f59'});s.transform(a,.42,[-.08,1.64,0]);return;
 }
 human(s,{coat:body==='sage'?'#dfb76d':body==='healer'?'#dfd4bb':body==='servant'?'#b5a586':P,skin:body==='giant'?'#625377':'#b77f59',hat:body==='servant'?'turban':body==='sage'?'none':'crown',trim:GOLD,robe:['sage','healer','servant'].includes(body),frame:'arjuna'});
 if(body==='archer'){tool(s,'bow');s.rod([.26,1.04,-.46],[.26,1.04,.46],.045,GOLD);}
 if(body==='mace'){tool(s,'mace');shield(s,GOLD);}
 if(body==='sage'){tool(s,'staff');s.rod([-.22,.84,-.40],[-.22,1.82,-.40],.035,GOLD);s.box([.02,1.76,-.40],[.42,.27,.025],IVORY,'sway',[-.22,1.82,-.40]);}
 if(body==='healer'){s.orb([.42,.85,.37],[.40,.46,.38],'#a8683f');s.box([.42,1.12,.37],[.22,.05,.20],GOLD);}
 if(body==='servant'){tool(s,'pick');s.box([-.33,.75,-.35],[.40,.52,.28],WOOD);}
 if(body==='giant'){s.transform(0,1.22,[0,0,0]);s.orb([.57,1.51,.36],[.65,.60,.55],'#777c83','strike',[.12,1.36,.3]);}
}
function templar(s,body){
 if(body==='ballista'){
  cart(s);s.box([0,.62,0],[1.17,.19,.28],WOOD);s.rod([.35,.69,-.65],[.35,.69,.65],.13,DARK);s.rod([-.55,.73,0],[.88,.73,0],.055,IRON);s.rod([.35,.69,-.65],[-.48,.73,0],.022,IVORY);s.rod([-.48,.73,0],[.35,.69,.65],.022,IVORY);s.poly([.91,.73,0],[-.13,-.06,-.13,.06,.15,0],.06,GOLD);return;
 }
 if(['marshal','turcopole'].includes(body)){mounted(s,{coat:body==='marshal'?'#c7c7ba':'#946c4c',rider:{coat:body==='marshal'?IVORY:'#9a7e5d',hat:body==='marshal'?'greathelm':'conical',armor:true,frame:'templar'},weapon:body==='marshal'?'spear':'bow',standard:body==='marshal'});s.box([-.05,.85,-.32],[.48,.34,.03],'#b7323f');return;}
 const healer=body==='hospitaller',worker=body==='acolyte',c=healer?'#27333c':body==='sergeant'?'#7f7767':IVORY;
 human(s,{coat:c,hat:worker?'hood':body==='crusader'?'greathelm':'helmet',trim:healer?IVORY:S,armor:!worker,robe:worker||healer,frame:'templar'});
 if(!worker){s.box([.25,1.04,0],[.025,.32,.055],healer?IVORY:'#ad303d');s.box([.27,1.09,0],[.025,.055,.24],healer?IVORY:'#ad303d');}
 if(body==='crossbowman')tool(s,'crossbow');
 else if(healer){tool(s,'book');s.box([-.27,.84,.26],[.25,.29,.22],WOOD);}
 else if(worker)tool(s,'hammer');
 else{tool(s,body==='sergeant'?'spear':'sword');shield(s,body==='sergeant'?WOOD:IVORY,body==='crusader');}
}
function atlantis(s,body){
 if(body==='engine'){cart(s,{cannon:true,culture:3});s.part('cylinder',[-.30,.68,0],[.57,.57,.57],'#be9b58');s.rod([-.4,.8,0],[-.4,1.48,0],.14,DARK);s.box([-.4,1.45,0],[.23,.08,.23],IRON);s.box([-.6,.70,-.42],[.24,.38,.03],P);return;}
 if(body==='ship'){vessel(s,'steamer');return;}
 if(body==='flyer'){s.orb([0,.75,0],[.80,.50,.46],WOOD,'bob');for(const z of [-1,1])s.poly([0,.82,z*.22],[-.51,0,.44,0,.1,z*.95,-.4,z*.82],.045,IVORY,'sway',[0,.82,z*.22],[Math.PI/2,0,0]);s.rod([.55,.75,-.34],[.55,.75,.34],.065,GOLD,'spin',[.55,.75,0]);s.orb([-.15,1,0],[.32,.25,.28],P);return;}
 if(body==='automaton'){human(s,{coat:'#b89a63',skin:IRON,hat:'helmet',armor:true,frame:'atlantis'});s.part('cylinder',[.25,1.05,0],[.24,.24,.06],GOLD,'spin',[.25,1.05,0],[0,Math.PI/2,0]);s.box([.20,1.48,0],[.04,.05,.26],'#9fe8db');tool(s,'spear');return;}
 human(s,{coat:body==='mystery'?'#414c56':body==='pioneer'?'#957b58':'#466d7d',hat:body==='mystery'?'hood':'tricorn',trim:GOLD,robe:body==='fellow',frame:'atlantis'});
 if(body==='pioneer'){tool(s,'hammer');s.box([-.38,.76,-.35],[.45,.56,.31],WOOD);}
 if(body==='gunner'){tool(s,'musket');s.rod([.30,1.06,-.46],[.30,1.06,.46],.055,IRON);s.box([-.22,.85,-.37],[.32,.42,.20],WOOD);}
 if(body==='fellow'){tool(s,'book');s.box([.38,1.18,.37],[.42,.10,.50],IVORY);s.orb([.34,1.42,.37],[.22,.30,.22],'#b6e9e2');}
 if(body==='mystery'){tool(s,'lantern');s.rod([-.20,.82,-.36],[-.20,1.78,-.36],.04,IRON);s.box([.03,1.68,-.36],[.46,.27,.025],DARK,'sway',[-.20,1.78,-.36]);}
}
function fleet(s,body){
 if(body==='junk'){vessel(s,'junk');return;}
 if(body==='qilin'){
  // The fleet brought a giraffe home: a real animal, rather than a generic fantasy unicorn.
  animal(s,{coat:'#bb9957'});s.rod([.40,.9,0],[.64,1.85,0],.20,'#bb9957');s.orb([.73,1.94,0],[.35,.28,.24],'#bb9957');s.rod([.65,2.03,-.07],[.63,2.2,-.07],.045,DARK);s.rod([.65,2.03,.07],[.63,2.2,.07],.045,DARK);for(const x of [-.4,-.1,.2])for(const z of [-.27,.27])s.box([x,.8,z],[.12,.18,.025],'#705335');s.box([-.1,.96,0],[.65,.10,.62],P);return;
 }
 if(body==='rockets'){cart(s);s.box([-.1,.77,0],[.86,.55,.72],'#a34c36');for(const z of [-.23,0,.23])for(const y of [.65,.85,1.05])s.rod([-.55,y,z],[.70,y+.25,z],.07,WOOD);s.box([.50,.79,0],[.035,.54,.68],DARK);return;}
 const worker=body==='porter';human(s,{coat:worker?'#a58b65':body==='chronicler'?'#374e65':P,skin:'#c9a078',hat:worker?'straw':body==='chronicler'?'cap':'conical',trim:'#7b3338',armor:body==='marine',robe:body==='chronicler',frame:'fleet'});
 if(worker){tool(s,'pick');s.rod([-.43,1.1,-.4],[.45,1.1,-.4],.055,WOOD);for(const x of [-.4,.4])s.box([x,.72,-.4],[.27,.35,.28],'#aa865b');}
 if(body==='marine'){tool(s,'spear');shield(s,'#9d513d');}
 if(body==='crossbow')tool(s,'crossbow');
 if(body==='signal'){tool(s,'lantern');banner(s,S,-.25,1.9);}
 if(body==='chronicler'){tool(s,'book');s.box([-.23,1,-.4],[.21,.42,.18],WOOD);}
}
function manifesto(s,body){
 if(body==='tank'){cart(s,{tank:true});return;}
 if(body==='katyusha'){cart(s,{truck:true,rockets:true});return;}
 if(body==='cavalry'){mounted(s,{coat:'#6c4c35',rider:{coat:'#756e4b',hat:'cap',trim:P,frame:'industrial'},weapon:'sword',standard:true});return;}
 human(s,{coat:body==='stakhanovite'?'#637581':body==='mole'?'#514c43':'#727650',hat:body==='mole'?'hood':body==='soldier'?'helmet':'cap',trim:body==='mole'?DARK:P,robe:body==='mole',frame:'industrial'});
 if(body==='stakhanovite'){tool(s,'pick');s.box([.21,1.57,0],[.08,.08,.12],'#ffe2a1');s.box([-.3,.88,-.32],[.28,.4,.22],WOOD);}
 else if(body==='agitator'){banner(s,P,-.29,1.95);tool(s,'book');s.box([.12,1.1,.32],[.06,.20,.045],GOLD);}
 else if(body==='mole')tool(s,'book');
 else {tool(s,'rifle');s.box([-.18,.90,-.35],[.27,.34,.17],WOOD);if(body==='sniper')s.box([-.24,1.04,0],[.12,.47,.56],'#9a9d7a');}
}
// Null Pointer Society: dark polished alloy under faction-coloured light, in the manner of a science-fiction RTS.
// Alloy, emissive and halo are viewer materials (gpu/unit-surfaces.js): alloy armour mirrors the sky and carries an
// edge light in the faction's colour; emissive parts light themselves and flare as the unit strikes or is struck; a
// halo is the soft glow around a light. Halos are drawn last (nullFinish), so they glow over the body they sit on.
const NGUN='#4c5b66',NPLATE='#35434d',NSUIT='#1b2329',NVOID='#0f161a',NHOT='#eafffb';
function alloy(s,start){for(const p of s.parts.slice(start))if(!p.halo)p.material='alloy';}
function lit(s,start){for(const p of s.parts.slice(start))if(!p.halo)p.material='emissive';}
// A light: an emissive bead and the halo around it, on the same joint.
function lamp(s,at,r,{c=P,m='',pivot=at,halo=3.2,stretch=[1,1,1],detail=0}={}){
 const bead=s.orb(at,stretch.map(k=>r*k),c,m,pivot);bead.material='emissive';bead.detail=detail;
 if(halo){const h=s.orb(at,stretch.map(k=>r*k*halo),c,m,pivot);h.halo=true;h.detail=detail;}
 return bead;
}
// A glow along a blade, lance or coil: a stretched halo centred on `at`, turned by `angle` to lie along it. A part turns
// about its joint's pivot, so the centre is turned back first.
function streak(s,at,size,angle,{c=P,m='',pivot=at,detail=0}={}){
 const dx=at[0]-pivot[0],dy=at[1]-pivot[1],cos=Math.cos(-angle),sin=Math.sin(-angle);
 const h=s.part('ellipsoid',[pivot[0]+dx*cos-dy*sin,pivot[1]+dx*sin+dy*cos,at[2]],size,c,m,pivot,[0,0,angle]);h.halo=true;h.detail=detail;return h;
}
// A ring of parts turning about a vertical axis: each is authored at angle 0 and turned into place about the axis.
function around(s,n,r,y,build){for(let i=0;i<n;i++){const a=i*2*Math.PI/n;build(i,[r,y,0],[0,-a,0]);}}
function nullFinish(s){const solid=s.parts.filter(p=>!p.halo),halos=s.parts.filter(p=>p.halo);for(const h of halos)h.material='halo';s.parts=[...solid,...halos];}
// The armoured trooper every biped of the house shares: plated legs on the gait joints, a sculpted cuirass with a light
// at its core, broad faction-coloured pauldrons, a visored helm and gauntlets banded with light.
function nullFrame(s,{lean=.04,hood=false,slim=false}={}){
 const legs=s.parts.length,w=slim?.9:1;
 for(const z of [-.17,.17]){
  const m=z<0?'strideL':'strideR',k=z<0?'kneeL':'kneeR',pivot=[0,.66,z],out=z<0?-1:1;
  s.taper([0,.68,z],[.03,.38,z],.088*w,NSUIT,m,pivot);
  const plate=s.parts.length;s.box([.06,.54,z],[.15,.25,.17*w],NPLATE,m,pivot);s.orb([.07,.35,z],[.17,.15,.18],NGUN,k,pivot);
  s.box([.08,.20,z],[.10,.24,.14],NGUN,k,pivot);alloy(s,plate);
  s.taper([.03,.34,z],[-.01,.08,z],.07,NSUIT,k,pivot);
  const sole=s.parts.length;s.poly([.11,.045,z],[-.17,-.045,.21,-.045,.28,.025,.07,.085,-.18,.07],.19,NPLATE,k,pivot);alloy(s,sole);
  const band=s.parts.length;s.box([.09,.22,z+out*.075],[.11,.025,.02],P,k,pivot);lit(s,band);s.parts[band].detail=1;
 }
 s.tag(legs,{joint:'legs'});
 const x=lean;
 s.lathe([x,.60,0],[[0,.14,.15],[.12,.17,.19],[.30,.22,.25],[.52,.30,.31],[.70,.22,.24]],NSUIT);
 const armour=s.parts.length;
 s.poly([x+.14,1.06,0],[-.08,-.24,.10,-.17,.13,.12,.03,.22,-.10,.20],.46,NGUN);
 s.box([x+.16,.80,0],[.11,.13,.27],NPLATE);s.box([x,1.30,0],[.30,.075,.40],NPLATE);
 s.box([x-.18,1.03,0],[.12,.38,.40],NPLATE);
 alloy(s,armour);
 lamp(s,[x+.25,1.07,0],.075,{halo:3.6});
 const seams=s.parts.length;for(const z of [-.13,.13])s.box([x+.25,.98,z],[.02,.17,.025],P);lit(s,seams);for(const p of s.parts.slice(seams))p.detail=1;
 // Pauldrons: the broad shoulder line that reads from the strategy camera, in the faction's colour.
 for(const z of [-.35,.35]){
  const pad=s.parts.length;s.orb([x-.01,1.25,z],[.36,.25,.30],P);s.box([x+.01,1.15,z*1.07],[.30,.07,.12],NPLATE);s.box([x-.02,1.32,z*.95],[.26,.05,.20],NGUN);alloy(s,pad);
  const rim=s.parts.length;s.box([x+.15,1.21,z*1.02],[.025,.11,.18],P);lit(s,rim);s.parts[rim].detail=1;
 }
 if(hood){
  // A cowl of plate over a void face: one lit eye in the dark.
  const cowl=s.parts.length;s.orb([x-.03,1.50,0],[.40,.42,.38],NPLATE);s.box([x-.17,1.38,0],[.15,.30,.42],NPLATE);s.poly([x-.02,1.66,0],[-.20,0,.16,0,-.06,.13],.07,NGUN);alloy(s,cowl);
  const face=s.parts.length;s.orb([x+.13,1.47,0],[.16,.24,.25],NVOID);s.parts[face].material='void';
  lamp(s,[x+.205,1.49,.05],.045,{halo:3.6});
 }else{
  const helm=s.parts.length;s.orb([x+.03,1.49,0],[.36,.36,.32],NGUN);s.poly([x-.06,1.62,0],[-.21,0,.15,0,-.04,.17],.06,NPLATE);s.box([x+.15,1.39,0],[.11,.10,.25],NPLATE);alloy(s,helm);
  const visor=s.parts.length;s.box([x+.20,1.50,0],[.06,.075,.30],P);s.box([x+.13,1.50,0],[.12,.075,.34],P);lit(s,visor);
  streak(s,[x+.22,1.50,0],[.14,.16,.48],0);
 }
 for(const z of [-.33,.33]){
  const m=z<0?'strideR':'strideL',pivot=[x,1.17,z];
  s.taper([x,1.17,z],[.13,.88,z],.085,NSUIT,m,pivot);
  const gauntlet=s.parts.length;s.box([.17,.87,z],[.25,.14,.16],NGUN,m,pivot);s.orb([.30,.85,z],[.11,.11,.11],NPLATE,m,pivot);alloy(s,gauntlet);
  const band=s.parts.length;s.box([.18,.945,z],[.20,.025,.165],P,m,pivot);lit(s,band);s.parts[band].detail=1;
 }
 // The suit under the plates is alloy too, so a blow anywhere ripples across the shield rather than flashing white.
 for(const p of s.parts.slice(legs))if(!['emissive','void'].includes(p.material)&&!p.halo)p.material='alloy';
}
function nullSociety(s,body){
 if(body==='bug'){
  // Exploit: a low six-legged infiltrator with a void carapace, outlined in the faction's colour.
  const shell=s.parts.length;s.orb([-.30,.42,0],[.62,.34,.46],NVOID);s.orb([.05,.44,0],[.42,.28,.38],NVOID);for(const p of s.parts.slice(shell))p.material='void';
  const head=s.parts.length;s.orb([.34,.43,0],[.30,.22,.28],NGUN);s.poly([.10,.58,0],[-.26,0,.22,0,.10,.07,-.18,.08],.30,NPLATE);alloy(s,head);
  const stripe=s.parts.length;for(const x of [-.44,-.30,-.16])s.box([x,.585,0],[.035,.02,.30],P);lit(s,stripe);
  for(const z of [-.07,.07])lamp(s,[.47,.46,z],.035,{halo:3});lamp(s,[.44,.52,0],.025,{halo:0});
  for(const x of [-.25,0,.25])for(const side of [-1,1]){
   const m=x*side<0?'strideL':'strideR',pivot=[x,.42,side*.18],leg=s.parts.length;
   s.taper([x,.42,side*.18],[x+.04,.47,side*.38],.065,NGUN,m,pivot);s.orb([x+.04,.47,side*.38],[.08,.08,.08],NPLATE,m,pivot);s.taper([x+.04,.47,side*.38],[x-.06,.03,side*.52],.055,NPLATE,m,pivot);alloy(s,leg);
   for(const p of s.parts.slice(leg))p.amplitude=.3;
  }
  const jaw=s.parts.length;for(const z of [-.09,.09])s.taper([.42,.36,z],[.58,.30,z*.4],.025,NPLATE,'sway',[.42,.36,z]);alloy(s,jaw);
  return;
 }
 if(body==='bomb'){
  // Fork bomb: a hovering mine. A hot core shows through a caged shell; the fork's two prongs carry its fuse lights.
  lamp(s,[0,.78,0],.30,{halo:2.1});
  const cage=s.parts.length;
  around(s,6,.33,.78,(i,at,rot)=>s.part('ellipsoid',at,[.11,.62,.20],NGUN,'spin',[0,.78,0],rot));
  s.lathe([0,1.06,0],[[0,.24,.24],[.05,.20,.20],[.09,.10,.10]],NPLATE);
  s.lathe([0,.36,0],[[0,.08,.08],[.10,.20,.20],[.16,.24,.24]],NGUN);
  alloy(s,cage);
  for(const z of [-.13,.13]){const prong=s.parts.length;s.rod([0,1.08,0],[.05,1.38,z],.05,NPLATE);alloy(s,prong);lamp(s,[.05,1.40,z],.05,{halo:3});}
  lamp(s,[0,.30,0],.07,{halo:3.4,stretch:[1.4,.5,1.4]});
  return;
 }
 if(body==='watch'){
  // Watchpoint: a floating sensor. Its lens turns toward what it watches inside a slowly turning ring of plates.
  const core=s.parts.length;s.orb([0,1.02,0],[.50,.46,.50],NGUN);s.lathe([0,.70,0],[[0,.04,.04],[.14,.14,.14],[.22,.20,.20]],NPLATE);alloy(s,core);
  const lens=s.parts.length;s.part('cylinder',[.22,1.02,0],[.30,.30,.10],NPLATE,'',[0,1.02,0],[0,Math.PI/2,0]);alloy(s,lens);
  lamp(s,[.27,1.02,0],.10,{halo:3});
  for(const p of s.parts.slice(lens))p.aim='head',p.pivot=[0,1.02,0];
  around(s,10,.44,1.02,(i,at,rot)=>{const ring=s.parts.length;s.box(at,[.07,.07,.21],i%2?NPLATE:P,'spin',[0,1.02,0],rot);if(i%2)alloy(s,ring);else lit(s,ring);});
  for(const a of [0,2.09,4.19]){const fin=s.parts.length;s.taper([Math.cos(a)*.16,.74,Math.sin(a)*.16],[Math.cos(a)*.30,.28,Math.sin(a)*.30],.045,NGUN,'sway',[0,.74,0]);alloy(s,fin);}
  lamp(s,[0,.62,0],.06,{halo:3.6,stretch:[1.5,.5,1.5]});
  return;
 }
 if(body==='collector'){
  // Garbage collector: a tracked salvage drone with a lit hopper and a repair claw.
  for(const z of [-.42,.42]){const track=s.parts.length;s.box([0,.24,z],[1.30,.30,.24],NSUIT);s.box([0,.40,z],[1.18,.05,.27],NPLATE);alloy(s,track);for(const x of [-.42,0,.42])s.wheel([x,.24,z+(z<0?-.12:.12)],.13,NPLATE);}
  const hull=s.parts.length;s.poly([0,.40,0],[-.62,0,.66,0,.46,.30,-.50,.34],.62,NGUN);s.box([-.20,.82,0],[.62,.14,.62],NPLATE);s.box([-.20,.62,0],[.66,.34,.06],NGUN);alloy(s,hull);
  const load=s.parts.length;s.box([-.20,.86,0],[.46,.05,.44],NSUIT);for(const [dx,dz,k] of [[-.08,-.08,.13],[.07,.06,.11],[-.06,.10,.09]])s.box([-.20+dx,.92,dz],[k,k,k],P,'',[-.20+dx,.92,dz],[0,dx*9,0]);lit(s,load+1);s.parts[load].material='alloy';
  streak(s,[-.20,.95,0],[.62,.26,.58],0);
  const arm=s.parts.length;s.rod([.30,.78,0],[.70,1.18,0],.10,NPLATE,'strike',[.30,.78,0]);s.rod([.70,1.18,0],[1.0,.88,0],.08,NGUN,'strike',[.30,.78,0]);
  for(const z of [-.06,.06])s.taper([1.0,.88,z],[1.10,.76,z*2.2],.025,NPLATE,'strike',[.30,.78,0]);alloy(s,arm);
  lamp(s,[1.04,.82,0],.045,{m:'strike',pivot:[.30,.78,0],halo:3.4});
  const mast=s.parts.length;s.rod([-.48,.88,.22],[-.52,1.40,.22],.03,NPLATE);alloy(s,mast);lamp(s,[-.52,1.42,.22],.035,{halo:3.2});
  return;
 }
 const hooded=body==='exception';nullFrame(s,{lean:body==='crash'?.09:.04,hood:hooded,slim:body==='pointer'});
 if(body==='crash'){
  // Segfault: twin energy blades sprung from the gauntlets.
  for(const z of [-.40,.40]){
   const pivot=[.20,.90,z],blade=s.parts.length;s.poly([.26,.93,z],[0,-.05,.48,.04,.78,.26,.50,.20,0,.07],.03,P,'strike',pivot);lit(s,blade);
   const edge=s.parts.length;s.poly([.27,.93,z],[0,-.01,.46,.08,.70,.25,.46,.17,0,.03],.035,NHOT,'strike',pivot);lit(s,edge);
   streak(s,[.62,1.06,z],[.80,.22,.14],.36,{m:'strike',pivot});
  }
 }else if(body==='pointer'){
  // Pointer: a light lance with a lit head that doubles as a mining probe, and a beacon pack.
  const pivot=[.25,.90,.37],lance=s.parts.length;s.rod([.08,.36,.37],[.44,1.62,.37],.045,NPLATE,'strike',pivot);alloy(s,lance);
  const head=s.parts.length;s.poly([.44,1.62,.37],[-.05,-.04,.05,-.04,.03,.30,0,.36,-.03,.30],.04,P,'strike',pivot);lit(s,head);
  streak(s,[.47,1.75,.37],[.13,.38,.12],-.28,{m:'strike',pivot});
  const pack=s.parts.length;s.box([-.26,1.02,0],[.16,.34,.30],NGUN);s.rod([-.30,1.18,-.10],[-.34,1.62,-.10],.025,NPLATE);alloy(s,pack);lamp(s,[-.34,1.64,-.10],.035,{halo:3});
 }else if(body==='exception'){
  // Exception: a rail rifle banded with coils of light, from under a void cowl.
  const x=.25,y=.92,z=.37,first=s.parts.length;
  s.box([x+.02,y,z],[.42,.12,.11],NPLATE);s.box([x+.50,y+.02,z],[.80,.06,.07],NGUN);s.box([x+.50,y+.07,z],[.62,.025,.05],NPLATE);s.box([x-.18,y-.11,z],[.10,.22,.07],NPLATE);
  alloy(s,first);
  const coils=s.parts.length;for(const dx of [.30,.48,.66])s.box([x+dx,y+.02,z],[.035,.10,.11],P);lit(s,coils);
  lamp(s,[x+.92,y+.02,z],.035,{halo:3.6});streak(s,[x+.48,y+.02,z],[.60,.14,.16],0);
  for(const p of s.parts.slice(first))p.motion='recoil';
  s.muzzle=[x+.92,y+.02,z];s.muzzleJoint='weapon';s.muzzlePivot=[x,y,z];
  for(const p of s.parts.slice(first)){p.aim='weapon';p.pivot=[x,y,z];}
 }else{
  // Null: the heavy line. A tower shield marked with the empty set, and a gauntlet cannon.
  const shield=s.parts.length;s.poly([.24,.92,-.50],[-.20,-.42,.20,-.42,.28,-.10,.28,.30,0,.48,-.28,.30,-.28,-.10],.08,NGUN);alloy(s,shield);
  const ring=s.parts.length;s.part('cylinder',[.25,.98,-.545],[.30,.30,.02],P);lit(s,ring);
  const hole=s.parts.length;s.part('cylinder',[.25,.98,-.555],[.20,.20,.02],NGUN);alloy(s,hole);
  const slash=s.parts.length;s.box([.25,.98,-.565],[.05,.42,.02],P,'',[.25,.98,-.565],[0,0,-.55]);lit(s,slash);
  const cannon=s.parts.length;s.box([.36,.90,.38],[.40,.13,.13],NPLATE,'recoil');s.part('cylinder',[.58,.90,.38],[.10,.10,.06],NGUN,'recoil',[.58,.90,.38],[0,Math.PI/2,0]);alloy(s,cannon);
  lamp(s,[.62,.90,.38],.035,{m:'recoil',halo:3});
  s.transform(0,1.1,[0,0,0]);
 }
}
function nations(s,body){
 if(body==='carronade'){cart(s,{cannon:true});return;}
 if(body==='merchantman'){vessel(s,'indiaman');return;}
 if(body==='dragoon'){mounted(s,{coat:'#c4c8c4',rider:{coat:'#a54a43',hat:'tricorn',trim:IVORY,frame:'nations'},weapon:'musket'});return;}
 human(s,{coat:body==='highlander'?'#b65b4d':body==='pinmaker'?'#927858':body==='smuggler'?'#4d625c':'#597382',hat:body==='highlander'?'beret':body==='pinmaker'?'cap':'tricorn',trim:IVORY,robe:body==='smuggler',frame:'nations'});
 if(body==='highlander'){
  s.box([0,.58,0],[.49,.35,.56],'#496254');for(const z of [-.31,.31]){s.box([0,.6,z],[.5,.065,.03],'#b85e4b');s.box([.04,.6,z],[.065,.33,.035],IVORY);}tool(s,'musket');
 }else if(body==='exciseman'){tool(s,'musket');s.rod([.30,1.02,-.48],[.30,1.02,.48],.045,IRON);}
 else if(body==='pinmaker'){tool(s,'hammer');s.box([-.36,.77,-.35],[.38,.55,.28],WOOD);s.box([-.25,.76,.30],[.04,.40,.07],GOLD);}
 else if(body==='shopkeeper'){tool(s,'book');s.box([-.27,.80,-.35],[.42,.40,.28],WOOD);}
 else{tool(s,'sword');shield(s,WOOD);s.box([-.24,.83,.35],[.32,.35,.23],WOOD);}
}
function garden(s,body){
 if(body==='eye'){s.orb([0,1,0],[.65,.42,.44],IVORY,'bob');s.orb([.32,1,0],[.06,.29,.27],P,'bob');s.orb([.36,1,0],[.045,.16,.14],DARK,'bob');for(const z of [-1,1]){s.poly([-.12,.95,z*.23],[-.4,0,0,.12,.25,0,0,-.14],.08,S,'sway',[-.12,.95,z*.23]);s.poly([-.16,1.15,z*.27],[-.28,0,.12,.08,.31,.02,.03,-.14],.025,'#d6a0a8','sway',[-.12,.95,z*.23]);}return;}
 if(body==='thorn'){s.orb([0,.22,0],[.75,.43,.66],'#6a8760');for(const z of [-.23,.23])s.taper([-.24,.27,z],[.15,.95,z],.12,'#587450');s.taper([.15,.95,0],[.73,1.28,0],.17,P,'strike',[.15,.95,0]);s.poly([.73,1.28,0],[0,0,-.09,.28,.12,.10,.30,.12],.17,IVORY);for(const z of [-.31,.31])s.poly([.15,.62,z],[-.30,0,.14,.08,.34,.22,.02,.16],.028,'#91aa75','sway',[.15,.95,z]);s.orb([-.18,.71,0],[.60,.50,.52],P);return;}
 if(body==='tree'){
  for(const z of [-.22,.22]){s.rod([0,.7,z],[z*.4,.08,z],.21,'#72523b',z<0?'strideL':'strideR',[0,.7,z]);s.rod([0,1.1,z],[.45,.68,z*2],.14,WOOD,'strike',[0,1.1,z]);}
  s.poly([0,.56,0],[-.24,0,.24,0,.19,.81,-.16,.81],.47,WOOD);s.orb([-.04,1.53,0],[.98,.65,.86],P);s.orb([.32,1.45,.17],[.58,.44,.55],'#91aa75');s.box([.22,1.25,-.12],[.035,.06,.08],GOLD);s.box([.22,1.25,.12],[.035,.06,.08],GOLD);return;
 }
 human(s,{coat:body==='statue'?IVORY:'#8b9a72',skin:body==='statue'?IVORY:'#bd9d7d',hat:body==='statue'?'none':'straw',trim:P,robe:body==='statue',frame:'garden'});
 if(body==='gardener')tool(s,'pick');
 else {s.orb([.35,1.03,.35],[.16,.15,.16],'#d68c8c','sway',[.25,.9,.35]);s.rod([.25,.8,.35],[.35,1.03,.35],.025,'#526f4e');s.box([-.25,1.03,.30],[.09,.44,.41],IVORY);}
}
function vessel(s,style,role='hull'){s.parts.push(...navalSculpt(style,role).parts);}
function landRig(seat,body){
 // Names recur between factions: Arjuna's archer is on foot while Kublai's is
 // mounted.  Bind the locomotion class to the authored chassis, never a name alone.
 if((seat===0&&['archer','keshig','horseman','camel'].includes(body))||(seat===1&&['chariot','elephant'].includes(body))||(seat===2&&['turcopole','marshal'].includes(body))||(seat===5&&body==='cavalry')||(seat===7&&body==='dragoon'))return 'rider';
 if(['trebuchet','ballista','engine','tank','katyusha','rockets','collector','carronade','truck'].includes(body))return 'vehicle';
 if(['ship','junk','merchantman'].includes(body))return 'ship';
 if(['qilin','tree','thorn','bug'].includes(body))return 'beast';
 if(['watch','bomb','eye','flyer'].includes(body))return 'hover';
 return 'biped';
}
// A house mark is deliberately structural, not a colour decal.  It sits above the
// common role body so a monochrome army still separates into ten cultures at a glance.
function houseSilhouette(s,seat,body){
 const back=-.31;
 if(seat===0){ // Horse-tail standard and curved composite bow case.
  s.rod([back,.82,.27],[back,1.83,.27],.034,WOOD);for(const y of [1.48,1.61,1.74])s.taper([back,y,.27],[back-.20,y+.20,.27],.026,'#493e34','sway',[back,1.83,.27]);
 }else if(seat===1){ // Temple crown over a lotus collar.
  for(const z of [-.23,0,.23])s.poly([-.08,1.67,z],[-.12,0,.12,0,0,.26],.06,GOLD);
 }else if(seat===2){ // Gothic backplate and a tall cross finial.
  s.poly([back,.98,.30],[-.20,0,.20,0,.15,.55,-.15,.55],.07,IVORY);s.box([back,1.63,.30],[.045,.42,.025],RED);s.box([back,1.75,.30],[.25,.045,.028],RED);
 }else if(seat===3){ // Brass instrument frame: visibly Renaissance even on workers.
  s.part('cylinder',[back,.98,.30],[.42,.42,.055],GOLD,'spin',[back,.98,.30],[Math.PI/2,0,0]);s.orb([back,1.02,.30],[.10,.10,.10],'#9cccd3');
 }else if(seat===4){ // Ming back banner with a curved roof cap.
  s.rod([back,.72,.29],[back,1.71,.29],.03,WOOD);s.poly([back,1.55,.29],[0,0,.34,.07,.26,.32,0,.32],.03,'#a34c36','sway',[back,1.71,.29]);s.box([back,1.72,.29],[.38,.045,.08],GOLD);
 }else if(seat===5){ // Industrial radiator and red star mast.
  for(const z of [-.08,.08])s.box([back,.96,.30+z],[.31,.46,.025],IRON);s.poly([back,1.55,.30],[0,.16,.04,.04,.15,.04,.06,-.03,.09,-.14,0,-.07,-.09,-.14,-.06,-.03,-.15,.04,-.04,.04],.025,P);
 }else if(seat===6){ // Two lit shards hover behind the shoulders of every trooper; drones and vehicles carry their own lights.
  if(['crash','zero','pointer','exception'].includes(body))for(const z of [-.24,.24]){
   const shard=s.parts.length;s.poly([back-.04,1.24,z],[0,-.21,.075,0,0,.24,-.075,0],.05,NGUN,'bob',[back-.04,1.24,z]);alloy(s,shard);
   const core=s.parts.length;s.poly([back-.04,1.24,z*1.02],[0,-.14,.04,0,0,.17,-.04,0],.055,P,'bob',[back-.04,1.24,z]);lit(s,core);
   for(const p of s.parts.slice(shard))p.amplitude=1.6;
  }
 }else if(seat===7){ // Highlander plaid roll and a merchant's compass disk.
  s.orb([back,.93,.29],[.44,.24,.16],'#496254');s.box([back,.93,.29],[.47,.045,.025],'#b85e4b');s.part('cylinder',[back,1.35,.30],[.27,.27,.035],GOLD,'',[back,1.35,.30],[Math.PI/2,0,0]);
 }else if(seat===8){ // Emu feather fan and exposed workshop exhaust.
  s.rod([back,.78,.29],[back,1.43,.29],.045,WOOD);for(const z of [-.14,0,.14])s.poly([back,1.43,.29+z],[-.13,0,.13,0,0,.35],.025,'#c7b57e','sway',[back,.78,.29]);
 }else{ // Botanical glass petals: a rounded organic outline replaces a square pack.
  for(const z of [-.26,0,.26])s.orb([back,.99,z],[.22,.08,.28],z?'#d3a7a4':IVORY,'sway',[back,.83,0]);s.rod([back,.78,0],[back,1.20,0],.025,'#587450');
 }
}
export function houseSculpt(id,flags=0,airRole='fighter'){
 if(!hasSculpt(id))return null;
 const [a,body]=id.split(':'),seat=Number(a),s=new Sculpt();
 if(flags&1){const model=airSculpt(seat,airRole);if(flags&2)model.parts.push({shape:'box',at:[-.33,.53,-.36],size:[.29,.30,.24],pivot:[-.33,.53,-.36],motion:'',color:'#b89458',on:[],material:'brass',detail:1});return {...model,rig:'air'};}
 [kublai,arjuna,templar,atlantis,fleet,manifesto,nullSociety,nations,(s,b)=>b==='truck'?cart(s,{truck:true,culture:8}):emu(s,{forager:b==='forager',gunner:b==='gunner'}),garden][seat](s,body);
 houseSilhouette(s,seat,body);
 if(flags&2)s.box([-.33,.83,-.36],[.29,.30,.24],'#b89458');
 if(seat===6)nullFinish(s);
 return {parts:s.parts,sculpt:id,muzzle:s.muzzle,muzzleJoint:s.muzzleJoint,muzzlePivot:s.muzzlePivot,rig:landRig(seat,body)};
}
