import { Sculpt, alloy, lit, mat, lamp, streak, finish } from './sculpt.js';
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
 ['mav','nugget','cold','bike','launcher','tug','jet','swan'],
];
export function hasSculpt(id){if(typeof id!=='string'||!/^\d{1,2}:[A-Za-z_][A-Za-z0-9_]{0,23}$/.test(id))return false;const [seat,body]=id.split(':');return Boolean(HOUSE_SCULPTS[Number(seat)]?.includes(body));}

const LEATHER='#6b4a33',FUR='#5a4636',WARM='#ffcf7a',FIRE='#ff9b45',KNEE=.32*.74;
// A ring of parts turning about a vertical axis: each is authored at angle 0 and turned into place about the axis.
function around(s,n,r,y,build){for(let i=0;i<n;i++){const a=i*2*Math.PI/n;build(i,[r,y,0],[0,-a,0]);}}
// A spoked wheel: iron tyre, wooden face, three spokes across it and a hub. Spokes and hub are close detail.
function wheel(s,at,r,{c=WOOD,rim=IRON,hub=GOLD}={}){
 const out=Math.sign(at[2]||1);
 s.part('cylinder',at,[r*2,r*2,.10],rim,'roll',at,[0,0,0],.4);s.part('cylinder',[at[0],at[1],at[2]+out*.012],[r*1.72,r*1.72,.10],c,'roll',at,[0,0,0],.4);
 const fine=s.parts.length;for(const a of [Math.PI/4,-Math.PI/4])s.box([at[0],at[1],at[2]+out*.07],[r*1.66,.05,.03],c,'roll',at,[0,0,a]);
 s.tag(fine,{detail:1});if(hub){s.part('cylinder',[at[0],at[1],at[2]+out*.085],[r*.5,r*.5,.06],hub,'roll',at).detail=2;}
}
// A cloak from the shoulders, in the faction's colour unless told otherwise; it sways as the body moves.
// It hangs from just behind the shoulders (back: the torso's depth there) and falls away from the body; an emblem
// rides on it.
function cape(s,x,{c=P,long=.92,wide=.62,clasp=GOLD,back=.27,emblem=null}={}){
 const first=s.parts.length,pivot=[x-back,1.33,0],turn=[0,Math.PI/2,-.2];
 s.poly([pivot[0],1.33-long/2,0],[-wide*.56,-long/2,wide*.56,-long/2,wide*.40,long/2,-wide*.40,long/2],.035,c,'sway',pivot,turn);
 if(emblem){s.box([pivot[0],1.0,-.03],[.07,.34,.02],emblem,'sway',pivot,turn);s.box([pivot[0],1.06,-.03],[.26,.07,.02],emblem,'sway',pivot,turn);}
 s.tag(first,{material:'cloth'});const fine=s.parts.length;for(const z of [-.2,.2])s.box([x-.02,1.31,z],[.07,.07,.07],clasp);s.tag(fine,{detail:1});
}
function human(s,{coat=P,skin='#d6a17e',legs=DARK,boots=LEATHER,hat='cap',trim=S,armor=false,robe=false,lean=0,frame='common',mounted=false,detail=true,pauldron=armor,cloak=null,emblem=null,hair=DARK,tache=false}={}){
 const start=s.parts.length;
 // Every frame owns its torso and head.  The shared tool sockets deliberately stay
 // put so a role can change equipment without turning each faction into the same pawn.
 for(const z of [-.16,.16]){
  const motion=z<0?'strideL':'strideR',pivot=[0,.65,z],knee=z<0?'kneeL':'kneeR';
  if(mounted){
   // Astride: thighs forward along the saddle, shins down the flank and feet in the stirrups. A rider's legs hold still.
   const w=z*2.3;s.taper([-.02,.64,z*1.2],[.30,.55,w],.10,legs);s.taper([.30,.55,w],[.26,.22,w*1.04],.08,legs);
   const boot=s.parts.length;s.capsule([.27,.26,w*1.04],[.16,.16,.17],boots);s.poly([.33,.14,w*1.04],[-.13,-.05,.17,-.05,.24,.04,.05,.08,-.15,.05],.17,boots);mat(s,boot,'leather');
   continue;
  }
  // Separate thigh, knee and calf give every walking unit a readable gait from the
  // strategy camera without adding a skinned mesh or a per-frame rig.
  s.capsule([-.03,.51,z],[.19,.33,.19],legs,motion,pivot);
  s.capsule([.02,.33,z],[.16,.13,.16],legs,knee,pivot);
  s.capsule([.05,.21,z],[.15,.27,.16],legs,knee,pivot);
  const boot=s.parts.length;s.capsule([.06,.15,z],[.17,.16,.18],boots,knee,pivot);
  s.poly([.15,.075,z],[-.18,-.06,.22,-.06,.32,.045,.08,.10,-.22,.06],.22,boots,knee,pivot);mat(s,boot,'leather');
  if(armor){const cop=s.parts.length;s.box([.09,.34,z],[.08,.11,.14],IRON,knee,pivot);s.box([.08,.22,z],[.06,.18,.13],IRON,knee,pivot);mat(s,cop,'iron',1);}
 }
 const legEnd=s.parts.length;
 let customHead=false;
 if(frame==='steppe'){
  // A deel: the long wrapped coat of the steppe, skirted over the boots and closed with a sash.
  s.lathe([lean,.38,0],[[0,.32,.29],[.13,.43,.39],[.53,.42,.37],[.78,.28,.26]],coat);s.capsule([lean-.10,1.16,0],[.62,.20,.54],FUR);
  const sash=s.parts.length;s.box([lean,.82,0],[.52,.08,.47],trim);s.poly([lean+.20,.98,.05],[-.05,.16,.05,.16,.05,-.16,-.05,-.16],.30,trim);mat(s,sash,'cloth',1);
 }else if(frame==='arjuna'){
  s.lathe([lean,.10,0],[[0,.25,.23],[.12,.32,.29],[.61,.22,.20],[1.05,.13,.12]],coat);s.box([lean,1.14,0],[.38,.040,.34],GOLD);
  s.orb([lean+.03,1.42,0],[.23,.28,.22],skin);s.lathe([lean,1.60,0],[[0,.17,.16],[.08,.24,.21],[.31,.035,.035]],GOLD);for(const z of [-.20,0,.20])s.poly([lean-.08,1.63,z],[-.12,0,.12,0,0,.20],.022,GOLD);customHead=true;
  // A gem in the crown burns in the faction's colour, and a jewelled collar lies over the chest.
  const collar=s.parts.length;s.lathe([lean,1.08,0],[[0,.24,.21],[.05,.27,.24],[.09,.18,.16]],GOLD);s.tag(collar,{material:'brass',detail:1});
  lamp(s,[lean+.17,1.66,0],.035,{halo:3.4});
 }else if(frame==='templar'){
  // Mail under a white surcoat, plate at the shoulders and a closed helm.
  s.lathe([lean,.39,0],[[0,.18,.16],[.28,.29,.25],[.57,.36,.30],[.76,.22,.20]],IVORY);
  const mail=s.parts.length;s.lathe([lean,.86,0],[[0,.26,.24],[.20,.30,.27],[.36,.22,.20]],IRON);s.tag(mail,{material:'iron'});
  s.capsule([lean+.03,1.43,0],[.37,.42,.34],IRON);s.box([lean+.25,1.42,0],[.025,.12,.20],DARK);customHead=true;
 }else if(frame==='atlantis'){
  s.lathe([lean,.22,0],[[0,.43,.37],[.12,.57,.49],[.48,.57,.49],[.69,.40,.35]],GOLD);s.box([lean,.80,0],[.74,.045,.62],DARK);
  for(const z of [-.39,.39]){s.orb([lean-.20,.88,z],[.24,.44,.20],IRON);s.rod([lean-.15,1.11,z],[lean+.02,1.20,z],.045,IRON);}
  s.orb([lean+.03,1.44,0],[.55,.52,.50],'#b89a63');
  // A diving-bell helm: a lit lens for a face, rivets round the collar and a gauge on the chest.
  lamp(s,[lean+.30,1.46,0],.11,{c:LIGHT,halo:2.6,stretch:[.7,1,1.05]});
  const rivets=s.parts.length;for(const z of [-.18,0,.18])s.orb([lean+.18,1.20,z],[.05,.05,.05],GOLD);s.part('cylinder',[lean+.30,.95,.16],[.13,.13,.04],IVORY,'',[lean+.30,.95,.16],[0,Math.PI/2,0]);s.tag(rivets,{material:'brass',detail:1});
  customHead=true;
 }else if(frame==='fleet'){
  // Lamellar over a long coat: overlapping plates laced in rows.
  s.lathe([lean,.24,0],[[0,.20,.18],[.10,.27,.23],[.78,.24,.21],[.95,.17,.15]],coat);s.box([lean-.25,.77,.28],[.15,.45,.16],WOOD);
  const plates=s.parts.length;for(const y of [.78,.92,1.06])s.lathe([lean,y,0],[[0,.27,.24],[.11,.25,.22]],armor?IRON:coat);s.tag(plates,{material:armor?'iron':'cloth',detail:1});
  s.orb([lean+.02,1.40,0],[.26,.28,.24],coat);s.lathe([lean,1.52,0],[[0,.43,.39],[.05,.52,.46],[.12,.39,.34],[.17,.12,.10]],'#c5a86c');customHead=true;
 }else if(frame==='industrial'){
  // A greatcoat with a cartridge belt and a crossed bandolier.
  s.lathe([lean,.15,0],[[0,.25,.22],[.10,.31,.27],[.86,.31,.27],[1.03,.23,.20]],coat);s.box([lean,1.07,0],[.70,.10,.52],IRON);s.box([lean-.27,.75,.30],[.20,.52,.16],DARK);
  const kit=s.parts.length;s.box([lean,.70,0],[.60,.07,.55],LEATHER);for(const z of [-.12,.12])s.box([lean+.27,.70,z],[.06,.09,.08],LEATHER);s.rod([lean+.24,1.02,-.20],[lean+.24,.68,.22],.035,LEATHER);mat(s,kit,'leather',1);
  s.lathe([lean,1.55,0],[[0,.26,.23],[.06,.30,.26],[.13,.25,.22]],DARK);customHead=true;
 }else if(frame==='nations'){
  s.lathe([lean,.18,0],[[0,.20,.18],[.15,.33,.28],[.53,.24,.21],[.85,.18,.16]],coat);s.lathe([lean-.20,.38,0],[[0,.14,.12],[.30,.28,.23],[.56,.14,.12]],coat);s.box([lean,.75,0],[.43,.045,.35],trim);
  // Facings and brass buttons down the coat.
  const facing=s.parts.length;s.poly([lean+.18,.98,0],[-.03,-.22,.03,-.22,.05,.20,-.05,.20],.30,trim);for(const y of [.86,.98,1.10])s.box([lean+.215,y,0],[.03,.04,.04],GOLD);s.tag(facing,{detail:1});
  s.capsule([lean+.03,1.39,0],[.22,.27,.21],skin);customHead=true;
  if(hat==='tricorn'){s.lathe([lean,1.55,0],[[0,.48,.28],[.05,.56,.34],[.11,.36,.22],[.17,.10,.07]],DARK);const fine=s.parts.length;s.orb([lean+.12,1.64,.25],[.09,.09,.04],P);s.tag(fine,{detail:1});}
  else headwear(s,hat,lean,trim);
 }else if(frame==='garden'){
  s.lathe([lean,.08,0],[[0,.31,.27],[.18,.25,.21],[.80,.18,.16],[1.05,.07,.06]],'#72523b');s.orb([lean,1.31,0],[.65,.42,.58],'#91aa75');
  for(const z of [-.38,.38])s.poly([lean,1.10,z],[-.30,0,.09,.10,.35,.03,.08,-.14],.028,'#d3a7a4','sway',[lean,.98,z]);customHead=true;
  // Glowing seed pods in the crown of leaves.
  for(const z of [-.18,.16])lamp(s,[lean+.08,1.50,z],.045,{c:'#ffe58a',halo:3.4,detail:1});
  // Eyes that glow out of the leaves.
  for(const z of [-.12,.12])lamp(s,[lean+.31,1.30,z],.035,{c:'#e9ff9a',halo:2.8,stretch:[.6,1,1]});
 }else if(frame==='pilot'){
  // A leather flight jacket over the flight suit: a knit waistband, a fleece collar, a brass zip, a name tag, and a
  // squadron patch and shoulder straps lit in the faction's colour. The head is bare under a ball cap or the pilot's own hair, and the
  // mirrored aviators catch the faction's colour too.
  s.capsule([lean,.70,0],[.36,.28,.34],legs);
  const jacket=s.parts.length;s.capsule([lean,1.0,0],[.54,.42,.48],coat);s.capsule([lean,.86,0],[.47,.16,.43],coat);mat(s,jacket,'leather');
  const knit=s.parts.length;s.box([lean,.79,0],[.47,.07,.43],trim);mat(s,knit,'cloth',1);
  const collar=s.parts.length;s.capsule([lean-.03,1.21,0],[.41,.11,.45],'#c9b79a');mat(s,collar,'organic');
  const fine=s.parts.length;s.box([lean+.255,1.0,0],[.015,.34,.03],GOLD);s.box([lean+.25,1.09,.13],[.02,.06,.12],IVORY);s.tag(fine,{detail:2});s.parts[fine].material='brass';
  const patch=s.parts.length;s.part('cylinder',[lean+.25,1.05,-.13],[.11,.11,.02],P,'',[lean+.25,1.05,-.13],[0,Math.PI/2,0]).detail=1;for(const z of [-.24,.24])s.box([lean-.02,1.19,z],[.17,.03,.07],P);lit(s,patch);
  s.capsule([lean+.03,1.40,0],[.29,.34,.28],skin);s.box([lean+.19,1.38,0],[.05,.06,.07],skin).detail=1;
  if(hat==='cap'){const cap=s.parts.length;s.orb([lean,1.51,0],[.32,.16,.31],P);s.box([lean+.20,1.49,0],[.20,.03,.24],P);mat(s,cap,'cloth');}
  else{const top=s.parts.length;s.orb([lean-.01,1.50,0],[.31,.19,.30],hair);mat(s,top,'organic');}
  if(tache){const lip=s.parts.length;s.box([lean+.195,1.33,0],[.035,.035,.13],hair);mat(s,lip,'organic',1);}
  aviators(s,lean);
  customHead=true;
 }else {
  s.capsule([lean,.99,0],[.52,.38,.46],coat,'',[lean,.92,0]);s.capsule([lean,.73,0],[.36,.34,.34],coat,'',[lean,.92,0]);
  if(robe)s.lathe([lean,.18,0],[[0,.32,.29],[.12,.38,.34],[.64,.25,.23]],coat);s.box([lean,.80,0],[.47,.052,.51],trim);
 }
 if(!customHead){s.capsule([lean+.03,1.40,0],[.29,.34,.28],skin);s.box([lean+.193,1.40,0],[.075,.09,.11],skin);s.box([lean+.174,1.46,0],[.035,.035,.24],DARK);}
 for(const z of [-.31,.31]){
  const m=z<0?'strideR':'strideL',pivot=[lean,1.15,z];
  if(frame==='garden'){s.taper([lean,1.12,z],[lean+.36,.72,z*1.25],.055,'#587450','sway',pivot);s.poly([lean+.39,.69,z*1.28],[-.12,0,.12,0,.20,.07,.04,.12,-.13,.06],.024,'#91aa75','sway',pivot);}else{
   // Shoulder, upper arm and forearm meet at an elbow, so an arm bends instead of hanging as one stick.
   const r=frame==='fleet'?.085:.11;
   s.capsule([lean,1.15,z],[.21,.21,.21],coat,m,pivot);s.taper([lean,1.12,z],[lean+.05,.97,z],r,coat,m,pivot);s.taper([lean+.05,.97,z],[.16,.86,z],r*.86,coat,m,pivot);
   s.capsule([.18,.84,z],[.14,.15,.15],skin,m,pivot);
   if(pauldron){const pad=s.parts.length;s.capsule([lean-.01,1.20,z*1.08],[.30,.17,.26],IRON,m,pivot);s.box([lean+.04,1.06,z*1.10],[.20,.06,.16],IRON,m,pivot);mat(s,pad,'iron');}
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
 if(detail){
  const ornament=s.parts.length;
  s.box([lean,.88,0],[.57,.055,.49],trim);s.box([lean+.22,.89,-.01],[.065,.10,.08],GOLD);
  s.rod([lean-.20,1.20,-.25],[lean+.22,.82,-.25],.027,trim);s.box([lean+.235,1.41,0],[.055,.07,.08],skin);
  s.box([lean+.05,.82,.26],[.14,.12,.07],LEATHER);
  if(armor){for(const z of [-.28,.28])s.box([lean+.03,1.08,z],[.30,.18,.055],IRON);}
  s.tag(ornament,{material:armor?'iron':'cloth',detail:1});s.parts[ornament+4].material='leather';
 }
 if(cloak)cape(s,lean,{c:cloak,emblem,back:{steppe:.46,arjuna:.30,templar:.36,atlantis:.55,fleet:.27,industrial:.31,nations:.32}[frame]??.27});
 s.tag(start,{joint:'legs'});
 // Undo the broad tag immediately after the leg segment.  The rider's torso, head,
 // and equipment must remain independent from the walking leg joint.
 for(const p of s.parts.slice(legEnd))if(p.joint==='legs')delete p.joint;
 return start;
}
function headwear(s,hat,x=0,trim=S){
 if(hat==='none')return;
 const first=s.parts.length;
 if(hat==='hood'){s.orb([x-.01,1.46,-.04],[.40,.45,.39],P);s.box([x+.20,1.40,0],[.04,.20,.23],DARK);s.taper([x-.14,1.36,0],[x-.30,1.10,0],.11,P);}
 else if(hat==='greathelm'){
  s.capsule([x+.01,1.48,0],[.40,.42,.37],IRON);s.poly([x+.16,1.49,0],[-.06,-.14,.06,-.14,.09,.14,-.09,.14],.39,DARK);s.box([x+.215,1.42,0],[.035,.18,.035],GOLD);
  // An eye slit, and a crest plume in the faction's colour.
  const fine=s.parts.length;s.box([x+.20,1.52,0],[.03,.035,.30],DARK);s.poly([x-.04,1.74,0],[-.20,-.02,.10,-.02,.18,.08,.04,.20,-.14,.14],.05,P,'sway',[x,1.70,0]);s.tag(fine,{detail:1});s.parts[fine+1].material='cloth';
 }
 else if(hat==='conical'){
  s.capsule([x,1.53,0],[.36,.18,.35],IRON);s.poly([x,1.58,0],[-.25,0,.25,0,.03,.37,-.05,.37],.39,IRON);s.box([x,1.52,0],[.46,.065,.43],trim);
  // A fur brim, a neck guard of lames and a plume at the spike.
  const fine=s.parts.length;s.capsule([x,1.50,0],[.48,.09,.46],FUR);s.lathe([x-.04,1.26,0],[[0,.20,.24],[.18,.22,.26]],IRON);s.rod([x,1.90,0],[x,2.04,0],.025,GOLD);s.poly([x-.03,2.06,0],[-.10,-.02,.06,-.02,.02,.16],.05,P,'sway',[x,2.0,0]);
  s.tag(fine,{detail:1});s.parts[fine].material='organic';s.parts[fine+3].material='cloth';
 }
 else if(hat==='turban'){s.orb([x,1.58,0],[.43,.25,.41],IVORY);s.box([x+.01,1.61,0],[.39,.055,.43],trim);s.poly([x-.12,1.62,0],[-.10,0,.12,0,0,.22],.10,GOLD);s.poly([x-.02,1.72,0],[-.04,0,.04,0,-.08,.22],.03,IVORY,'sway',[x,1.70,0]);lamp(s,[x+.21,1.60,0],.035,{halo:3.4});}
 else if(hat==='tricorn'){s.poly([x,1.57,0],[-.32,0,.32,0,.10,.20,-.20,.20],.52,DARK);s.box([x,1.56,0],[.66,.045,.57],trim);const fine=s.parts.length;s.orb([x+.06,1.66,.25],[.10,.10,.04],P);s.tag(fine,{detail:1});}
 else if(hat==='fur'){s.orb([x,1.56,0],[.43,.20,.43],FUR);s.poly([x,1.62,0],[-.18,0,.18,0,0,.23],.29,P);const fine=s.parts.length;for(const z of [-.20,.20])s.box([x-.02,1.42,z],[.16,.20,.05],FUR);s.tag(fine,{detail:1});s.tag(first,{material:'organic'});s.parts[first+1].material='cloth';}
 else if(hat==='straw'){s.poly([x,1.52,0],[-.40,0,.40,0,0,.28],.64,'#c5a86c');s.box([x,1.55,0],[.34,.04,.34],trim);}
 else if(hat==='crown'){s.box([x,1.58,0],[.38,.12,.35],GOLD);for(const z of [-.15,.15])s.poly([x,1.64,z],[-.2,0,.2,0,.17,.20,0,.08,-.17,.20],.05,GOLD);lamp(s,[x+.20,1.60,0],.035,{halo:3.4});}
 else if(hat==='beret'){s.orb([x,1.56,0],[.42,.16,.40],P);s.box([x+.21,1.55,0],[.19,.045,.32],DARK);const fine=s.parts.length;s.orb([x-.02,1.65,0],[.09,.08,.09],trim);s.poly([x-.10,1.62,.17],[-.03,0,.03,0,-.10,.30],.02,IVORY,'sway',[x-.10,1.62,.17]);s.tag(fine,{detail:1});}
 else {
  s.orb([x,1.56,0],[.39,.20,.37],hat==='helmet'?IRON:DARK);s.box([x+.21,1.55,0],[.19,.045,.32],hat==='helmet'?IRON:DARK);
  const fine=s.parts.length;if(hat==='helmet')s.lathe([x,1.49,0],[[0,.25,.24],[.03,.27,.26]],IRON);else s.box([x+.17,1.60,0],[.04,.06,.10],P);s.tag(fine,{detail:1});
 }
}
function tool(s,kind,{x=.25,y=.9,z=.37}={}){
 const first=s.parts.length;
 if(['bow','crossbow','rifle','musket'].includes(kind)){s.muzzle=[x+(kind==='bow'?.44:kind==='crossbow'?.45:.79),y+(kind==='rifle'||kind==='musket'?.035:0),z];s.muzzleJoint='weapon';s.muzzlePivot=[x,y,z];}
 if(kind==='bow'){
  // A recurved bow with a leather grip and an arrow on the string; the quiver rides at the hip.
  s.poly([x+.25,y,z],[0,-.43,.14,-.27,.19,0,.14,.28,0,.44,.06,.20,.08,0,.04,-.20],.065,WOOD);
  s.rod([x+.25,y-.43,z],[x+.25,y+.44,z],.022,IVORY);
  const fine=s.parts.length;s.box([x+.43,y,z],[.06,.12,.08],LEATHER);s.rod([x+.06,y,z],[x+.62,y,z],.018,WOOD);s.poly([x+.64,y,z],[-.03,-.03,-.03,.03,.05,0],.02,IRON);s.tag(fine,{detail:1});s.parts[fine].material='leather';
  s.box([-.30,1.13,-z],[.16,.53,.18],LEATHER);const fletch=s.parts.length;for(const dz of [-.04,.04])s.poly([-.30,1.44,-z+dz],[-.04,0,.04,0,0,.12],.02,P);s.tag(fletch,{detail:1});
 }
 else if(kind==='crossbow'){s.box([x+.1,y,z],[.70,.09,.10],WOOD);s.rod([x+.35,y,z-.32],[x+.35,y,z+.32],.07,IRON);s.box([x-.12,y-.10,z],[.16,.20,.07],WOOD);s.box([x+.18,y-.10,z],[.055,.18,.045],IRON);
  const fine=s.parts.length;s.rod([x+.35,y,z-.30],[x+.06,y,z],.015,IVORY);s.rod([x+.06,y,z],[x+.35,y,z+.30],.015,IVORY);s.box([x+.45,y-.06,z],[.03,.12,.12],IRON);s.tag(fine,{detail:1});}
 else if(kind==='rifle'||kind==='musket'){
  s.box([x,y,z],[.43,.13,.12],WOOD);s.box([x+.44,y+.035,z],[.70,.055,.06],IRON);s.box([x-.17,y-.13,z],[.10,.25,.075],WOOD);s.box([x+.18,y-.08,z],[.055,.20,.035],IRON);if(kind==='rifle')s.box([x+.17,y+.13,z],[.26,.055,.07],DARK);
  // Barrel bands, a brass lock and, on a musket, a bayonet.
  const fine=s.parts.length;for(const dx of [.30,.58])s.box([x+dx,y+.035,z],[.03,.08,.08],kind==='musket'?GOLD:IRON);s.box([x+.08,y+.05,z+.05],[.10,.06,.03],GOLD);if(kind==='musket')s.box([x+.94,y+.06,z],[.30,.02,.03],IRON);s.tag(fine,{detail:1});
  for(const p of s.parts.slice(first))p.motion='recoil';
 }
 else if(kind==='spear'||kind==='staff'){
  s.rod([x,y-.62,z],[x+.24,y+.90,z],.055,WOOD);s.poly([x+.24,y+.90,z],[-.07,-.02,.07,-.02,0,.28],.065,IRON);s.box([x+.02,y+.05,z],[.13,.045,.10],GOLD);
  const fine=s.parts.length;s.box([x+.23,y+.86,z],[.05,.07,.05],GOLD);s.poly([x+.21,y+.82,z],[-.03,0,.03,0,.05,-.18,-.05,-.16],.04,P,'sway',[x+.22,y+.86,z]);s.tag(fine,{detail:1});s.parts[fine+1].material='cloth';
 }
 else if(kind==='sword'){
  // Blade with a ridge, a crossguard, a wrapped grip and a pommel, all on the strike joint.
  const tilt=[0,0,-.55];
  s.poly([x,y+.50,z],[-.035,-.36,.035,-.36,.03,.28,0,.38,-.03,.28],.04,IRON,'strike',[x,y,z],tilt);s.box([x,y+.11,z],[.30,.05,.07],GOLD,'strike',[x,y,z],tilt);
  const fine=s.parts.length;s.box([x,y,z],[.05,.17,.05],LEATHER,'strike',[x,y,z],tilt);s.orb([x,y-.10,z],[.065,.065,.065],GOLD,'strike',[x,y,z],tilt);s.tag(fine,{detail:1});s.parts[fine].material='leather';
 }
 else if(kind==='pick'){s.rod([x,y-.45,z],[x+.27,y+.26,z],.055,WOOD,'strike',[x,y,z]);s.poly([x+.26,y+.27,z],[-.26,-.10,-.02,.02,.26,-.06,.02,.08],.07,IRON,'strike',[x,y,z]);}
 else if(kind==='hammer'){s.rod([x,y-.22,z],[x+.16,y+.40,z],.07,WOOD,'strike',[x,y,z]);s.box([x+.16,y+.38,z],[.32,.20,.23],IRON,'strike',[x,y,z]);const fine=s.parts.length;s.box([x+.16,y+.38,z],[.34,.05,.25],DARK,'strike',[x,y,z]);s.tag(fine,{detail:1});}
 else if(kind==='mace'){s.rod([x,y-.20,z],[x+.16,y+.46,z],.075,WOOD,'strike',[x,y,z]);s.orb([x+.16,y+.48,z],[.31,.30,.31],GOLD,'strike',[x,y,z]);const fine=s.parts.length;for(const size of [[.40,.08,.08],[.08,.40,.08],[.08,.08,.40]])s.box([x+.16,y+.48,z],size,GOLD,'strike',[x,y,z]);s.tag(fine,{detail:1});}
 else if(kind==='book'){s.box([x+.10,y,z],[.30,.08,.39],WOOD);s.box([x+.10,y+.04,z],[.25,.045,.33],IVORY);const fine=s.parts.length;s.box([x+.25,y,z],[.03,.09,.10],GOLD);s.box([x+.05,y+.02,z+.12],[.02,.03,.20],RED);s.tag(fine,{detail:1});}
 else if(kind==='lantern'){
  // A lantern that actually lights: a warm flame behind its glass and a glow round it.
  s.rod([x,y,z],[x,y+.5,z],.045,DARK);s.box([x,y-.12,z],[.22,.30,.22],GOLD,'sway',[x,y+.3,z]);s.box([x,y+.05,z],[.14,.05,.14],DARK,'sway',[x,y+.3,z]);
  lamp(s,[x,y-.12,z],.08,{c:WARM,m:'sway',pivot:[x,y+.3,z],halo:3.6,stretch:[1,1.3,1]});
 }
 const ranged=['bow','crossbow','rifle','musket'].includes(kind);
 // Every piece of a weapon turns as one rigid assembly. Its socket uses the
 // same pivot, so the muzzle and visible barrel cannot separate while aiming.
 const keep=s.parts.slice(first).map(p=>p.detail);
 s.tag(first,{detail:0,aim:ranged?'weapon':undefined,pivot:ranged?[x,y,z]:undefined});
 // Close detail stays close detail after the assembly is tagged.
 s.parts.slice(first).forEach((p,i)=>{if(keep[i])p.detail=keep[i];});
}
function shield(s,c=P,cross=false){
 s.poly([.10,.92,-.46],[-.27,.25,.27,.25,.23,-.13,0,-.35,-.23,-.13],.10,c);
 // An iron rim behind the face and a boss at its centre.
 const fine=s.parts.length;s.poly([.10,.92,-.44],[-.30,.28,.30,.28,.26,-.14,0,-.39,-.26,-.14],.06,IRON);s.orb([.10,.98,-.52],[.10,.10,.06],GOLD);s.tag(fine,{detail:1});
 if(cross){s.box([.11,.94,-.52],[.065,.40,.02],'#ae303d');s.box([.11,1,-.53],[.30,.065,.02],'#ae303d');}
}
function banner(s,c=P,x=-.20,height=2.1){
 s.rod([x,.65,-.35],[x,height,-.35],.035,WOOD);
 // A swallow-tailed flag on a crossbar, with a gilt finial.
 s.poly([x+.30,height-.22,-.35],[-.27,.18,.27,.18,.27,-.18,.12,-.06,0,-.20,-.27,-.18],.04,c,'sway',[x,height,-.35]);
 const fine=s.parts.length;s.rod([x,height-.03,-.35],[x+.58,height-.03,-.35],.025,GOLD);s.poly([x,height+.02,-.35],[-.04,0,.04,0,0,.14],.04,GOLD);s.tag(fine,{detail:1});
}
// A horse in proportion: barrel, chest and quarters, an arched neck and a long head, legs that bend at the knee as they
// stride, a mane and a swinging tail. A chariot team (compact) keeps rigid legs. Camels and giraffes share the frame.
function horse(s,{coat='#926044',mane=DARK,compact=false,tack=true,camel=false,giraffe=false,cloth=P}={}){
 const start=s.parts.length,h=camel?.86:giraffe?.98:.70,neck=giraffe?[[.46,h+.14],[.70,h+1.20]]:camel?[[.48,h+.10],[.88,h+.30]]:[[.48,h+.16],[.78,h+.58]];
 s.capsule([-.04,h+.02,0],[1.10,.46,.44],coat);s.capsule([.40,h+.05,0],[.46,.50,.42],coat);if(!compact)s.capsule([-.46,h+.06,0],[.50,.50,.46],coat);
 s.taper([neck[0][0],neck[0][1],0],[neck[1][0],neck[1][1],0],giraffe?.13:.16,coat);
 const head=camel?[1.00,h+.62]:[neck[1][0]+.02,neck[1][1]+.08];
 if(camel)s.taper([.88,h+.30,0],[.98,h+.62,0],.12,coat);
 s.taper([head[0],head[1],0],[head[0]+.24,head[1]-.24,0],.095,coat);s.capsule([head[0]+.26,head[1]-.26,0],[.15,.13,.14],coat);
 if(!compact)for(const z of [-.06,.06])s.poly([head[0],head[1]+.14,z],[-.04,0,.04,0,0,.11],.03,coat);
 if(camel)for(const x of [-.30,.12])s.capsule([x,h+.34,0],[.42,.42,.36],coat);
 if(giraffe){s.rod([head[0]-.02,head[1]+.12,-.06],[head[0]-.04,head[1]+.28,-.06],.04,DARK);s.rod([head[0]-.02,head[1]+.12,.06],[head[0]-.04,head[1]+.28,.06],.04,DARK);}
 const hair=s.parts.length;
 if(!camel){const [a,b]=neck,dx=b[0]-a[0],dy=b[1]-a[1];s.poly([(a[0]+b[0])/2-.05,(a[1]+b[1])/2+.05,0],[-dx/2-.08,-dy/2,dx/2-.06,dy/2,dx/2-.01,dy/2-.06,-dx/2-.02,-dy/2-.08],.07,mane);}
 s.taper([-.70,h+.14,0],[-.86,h-.32,0],giraffe?.04:.07,mane,'sway',[-.70,h+.14,0]);
 s.tag(hair,{material:'organic'});
 for(const x of [-.44,.38])for(const z of [-.17,.17]){
  const L=x*z>0,m=L?'strideL':'strideR',pivot=[x,h-.04,z];
  if(compact){s.taper([x,h-.04,z],[x+.03,.10,z],.07,coat,m,pivot);s.box([x+.04,.05,z],[.13,.10,.12],DARK,m,pivot);continue;}
  s.capsule([x,h-.16,z],[.17,.30,.18],coat,m,pivot);
  const k=L?'kneeL':'kneeR',lower=s.parts.length,knee=h-.04-KNEE;
  // The knee sits KNEE below the hip, the depth the walk cycle bends at; the cannon runs from it to the hoof.
  s.taper([x+.01,knee+.02,z],[x+.04,.11,z],camel?.045:.05,coat,k,pivot);s.box([x+.05,.05,z],[.13,.10,.12],DARK,k,pivot);
  for(const p of s.parts.slice(lower))p.kneeDrop=KNEE;
 }
 s.tag(start,{material:'organic'});
 if(giraffe){const spots=s.parts.length;for(const [x,y] of [[-.40,h+.20],[-.10,h+.24],[.20,h+.22],[.52,h+.48],[.60,h+.80]])for(const z of [-.23,.23])s.box([x,y,z*(y>h+.4?.55:1)],[.12,.12,.02],'#705335');s.tag(spots,{material:'organic',detail:1});}
 if(!tack)return start;
 // Saddle cloth in the faction's colour over a leather saddle, girth, bridle, reins and stirrups.
 const top=h+(camel?.58:.24);
 s.box([-.06,top,0],[.66,.05,.62],cloth);s.tag(s.parts.length-1,{material:'cloth'});
 const t=s.parts.length;s.capsule([-.08,top+.06,0],[.42,.12,.34],LEATHER);s.box([.12,top+.11,0],[.06,.10,.20],LEATHER);s.box([-.28,top+.12,0],[.06,.12,.26],LEATHER);
 s.box([-.02,h+.01,0],[.07,.48,.47],LEATHER);
 const fine=s.parts.length;
 for(const z of [-1,1]){s.rod([head[0]+.24,head[1]-.20,z*.075],[head[0]+.02,head[1]+.06,z*.075],.02,LEATHER);s.rod([head[0]+.22,head[1]-.22,z*.08],[.20,top+.08,z*.16],.018,LEATHER);s.rod([-.04,top,z*.30],[-.04,h-.12,z*.31],.015,LEATHER);s.box([-.04,h-.14,z*.31],[.10,.025,.06],IRON);}
 mat(s,t,'leather');s.tag(fine,{detail:2});s.parts[s.parts.length-1].material='iron';s.parts[s.parts.length-5].material='iron';
 return start;
}
// Caparison: a cloth skirt to the knees in the faction's colour, with a trim; a chanfron plate guards the face.
function barding(s,{c=P,trim=S,plate=false,lamellar=false}={}){
 const h=.70,first=s.parts.length;
 for(const z of [-.26,.26]){
  if(lamellar){for(const x of [-.30,.22])s.box([x,h-.02,z],[.46,.34,.03],IRON);}
  else{s.poly([-.04,h-.06,z],[-.70,.24,.62,.24,.66,-.10,.50,-.22,.30,-.12,.10,-.22,-.10,-.12,-.30,-.22,-.50,-.12,-.70,-.22],.03,c);s.box([-.04,h+.17,z*1.02],[1.34,.05,.02],trim);}
 }
 s.tag(first,{material:lamellar?'iron':'cloth'});
 if(plate||lamellar){const p=s.parts.length;s.box([.96,h+.58,0],[.24,.06,.13],IRON,'',[.96,h+.58,0],[0,0,-.8]);s.tag(p,{material:'iron'});}
}
function elephant(s,{coat='#7e898d'}={}){
 const start=s.parts.length,h=.92;
 s.capsule([-.08,h+.18,0],[1.55,1.02,.92],coat);s.orb([-.55,h+.12,0],[.80,.90,.84],coat);
 s.orb([.70,h+.40,0],[.66,.70,.64],coat);
 // Ears that fan, a trunk in three swinging segments, tusks.
 for(const z of [-1,1])s.poly([.56,h+.44,z*.36],[-.10,.34,.22,.30,.24,-.30,-.04,-.40,-.20,-.10],.06,coat,'sway',[.62,h+.60,z*.30]);
 s.taper([.96,h+.36,0],[1.10,h-.02,0],.17,coat,'sway',[.96,h+.36,0]);s.taper([1.10,h-.02,0],[1.12,h-.42,0],.13,coat,'sway',[.96,h+.36,0]);s.taper([1.12,h-.42,0],[1.20,h-.62,0],.09,coat,'sway',[.96,h+.36,0]);
 for(const z of [-.22,.22])s.taper([.96,h+.18,z],[1.30,h+.08,z*1.2],.065,IVORY);
 for(const z of [-.2,.2])s.orb([.96,h+.56,z],[.06,.06,.04],DARK);
 for(const x of [-.58,.46])for(const z of [-.28,.28]){const m=x*z>0?'strideL':'strideR',pivot=[x,h-.02,z];s.capsule([x,h/2-.02,z],[.30,h+.02,.31],coat,m,pivot);const toes=s.parts.length;s.box([x+.13,.05,z],[.05,.07,.18],IVORY,m,pivot);s.tag(toes,{detail:1});}
 s.rod([-1.12,h+.30,0],[-1.26,h-.20,0],.05,DARK,'sway',[-1.12,h+.30,0]);
 s.tag(start,{material:'organic'});
 return start;
}
function mounted(s,{coat,camel=false,rider={},weapon='spear',standard=false,armor=null,mane=DARK}={}){
 horse(s,{coat,camel,mane});if(armor)barding(s,armor);
 const start=human(s,{...rider,mounted:true});s.transform(start,.74,[-.17,camel?.90:.60,0]);
 tool(s,weapon,{x:.05,y:camel?1.52:1.20,z:.28});if(standard)banner(s,P,-.34,camel?2.42:2.12);
}
function cart(s,{tank=false,truck=false,cannon=false,rockets=false,culture=0}={}){
 const c=tank?'#60704c':culture===3?'#947855':culture===10?NAVY:WOOD;
 const turret=(pivot,build,muzzle)=>{const first=s.parts.length;build();s.tag(first,{joint:'turret',pivot});if(muzzle){s.muzzle=muzzle;s.muzzleJoint='turret';s.muzzlePivot=[...pivot];}};
 if(tank){
  // Tracks on five road wheels with a drive sprocket, a sloped hull with fenders, headlamps and exhausts; a cast
  // turret with a mantlet, a long gun, a commander's cupola and the faction's star.
  const start=s.parts.length;
  for(const z of [-.45,.45]){const out=Math.sign(z);
   s.box([0,.27,z],[1.70,.40,.26],DARK);
   for(const x of [-.60,-.30,0,.30,.60])s.part('cylinder',[x,.22,z+out*.14],[.24,.24,.04],IRON,'roll',[x,.22,z],[0,0,0],.4);
   s.part('cylinder',[.80,.34,z+out*.13],[.22,.22,.05],GOLD,'roll',[.80,.34,z],[0,0,0],.4);
   const links=s.parts.length;for(const x of [-.65,-.43,-.21,.01,.23,.45,.67])s.box([x,.49,z],[.14,.04,.27],IRON,'roll',[0,.27,z]);s.tag(links,{detail:1});
   s.box([0,.52,z*1.03],[1.82,.04,.30],c);
  }
  s.poly([0,.44,0],[-.86,0,.86,0,.60,.38,-.66,.38],.70,c);
  const hullFine=s.parts.length;for(const z of [-.16,.16])s.part('cylinder',[-.86,.66,z],[.10,.10,.18],DARK,'',[-.86,.66,z],[0,Math.PI/2,0]);s.box([.30,.83,.25],[.40,.04,.10],DARK);s.tag(hullFine,{detail:1});
  s.tag(start,{material:'iron'});
  for(const z of [-.27,.27])lamp(s,[.84,.62,z],.045,{c:WARM,halo:3.4});
  turret([-.12,.89,0],()=>{
   s.lathe([-.10,.80,0],[[0,.40,.34],[.10,.44,.37],[.26,.34,.29],[.32,.18,.16]],c);s.box([.30,.94,0],[.16,.20,.24],IRON);
   s.tube([.31,.99,0],[1.41,.99,0],.06,IRON,'recoil',[-.12,.89,0]);s.box([1.40,.99,0],[.10,.15,.15],DARK,'recoil',[-.12,.89,0]);
   s.part('cylinder',[-.24,1.15,.10],[.22,.22,.10],IRON,'',[-.24,1.15,.10],[Math.PI/2,0,0]);
   const fine=s.parts.length;s.box([-.24,1.21,.10],[.18,.03,.18],DARK);s.rod([-.40,1.10,-.18],[-.46,1.60,-.18],.015,DARK);s.tag(fine,{detail:1});
   s.poly([-.06,.93,.36],[0,.10,.025,.03,.10,.03,.04,-.012,.06,-.09,0,-.045,-.06,-.09,-.04,-.012,-.10,.03,-.025,.03],.03,P);
   s.tag(s.parts.length-1,{material:'cloth'});
  },[1.47,.99,0]);
  return;
 }
 if(truck||rockets){
  // A cab with glazed windows and headlamps, a grille and fenders over a flat bed.
  s.box([0,.38,0],[1.52,.12,.74],DARK);
  const body=culture===8?'#8c8a5b':culture===10?'#a9b0b5':'#68754d';s.box([.46,.70,0],[.52,.54,.66],body);s.box([.82,.54,0],[.26,.30,.62],body);
  s.box([.735,.82,0],[.02,.20,.54],'#98c7cf');for(const z of [-.335,.335])s.box([.46,.82,z],[.30,.18,.01],'#98c7cf');
  const fine=s.parts.length;s.box([.955,.54,0],[.02,.22,.40],IRON);for(const z of [-.36,.36])s.box([.62,.44,z],[.62,.05,.12],body);s.tag(fine,{detail:1});
  s.box([-.34,.50,0],[.86,.14,.74],c);s.tag(s.parts.length-1,{material:'wood'});
  for(const z of [-.24,.24])lamp(s,[.97,.60,z],.045,{c:WARM,halo:3.4});
  for(const x of [-.50,.52])for(const z of [-.42,.42])wheel(s,[x,.22,z],.21,{c:DARK,rim:DARK,hub:IRON});
 }else{
  s.box([0,.30,0],[1.48,.16,.80],c);
  // Side rails and iron corner straps over planks.
  const fine=s.parts.length;for(const z of [-.38,.38])s.box([0,.42,z],[1.44,.06,.05],c);for(const x of [-.70,.70])for(const z of [-.38,.38])s.box([x,.32,z],[.08,.18,.06],IRON);s.tag(fine,{detail:1});
  for(const x of [-.48,.48])for(const z of [-.47,.47])wheel(s,[x,.22,z],.22);
 }
 if(rockets)turret([-.39,.70,0],()=>{for(const z of [-.23,0,.23]){s.rod([-.73,.72,z],[.14,1.05,z],.10,IRON);s.rod([-.75,.86,z],[.12,1.19,z],.10,IRON);}s.box([-.39,.70,0],[.12,.43,.72],DARK);const tips=s.parts.length;for(const z of [-.23,0,.23])for(const y of [1.05,1.19])s.orb([.16,y,z],[.07,.07,.07],RED);s.tag(tips,{detail:1});},[.22,1.20,0]);
 if(truck&&culture===8)turret([-.3,1.02,0],()=>{s.rod([-.3,.5,0],[-.3,1.02,0],.07,IRON);s.box([.01,1.04,0],[.95,.075,.09],DARK);s.part('cylinder',s.about([-.12,1.12,0],[-.3,1.02,0],[Math.PI/2,0,0]),[.27,.27,.05],DARK,'recoil',[-.3,1.02,0],[Math.PI/2,0,0]);const fine=s.parts.length;s.box([-.30,1.20,0],[.30,.22,.04],IRON);s.tag(fine,{detail:1});},[.36,1.12,0]);
 if(cannon)turret([-.30,.55,0],()=>{s.tube([-.52,.72,0],[.58,.72,0],.195,DARK,'recoil',[-.30,.55,0]).amplitude=.08;s.box([-.30,.55,0],[.37,.16,.63],GOLD);const fine=s.parts.length;for(const dx of [-.30,.30])s.tube([dx,.72,0],[.06+dx,.72,0],.225,IRON,'recoil',[-.30,.55,0]);s.tube([.58,.72,0],[.66,.72,0],.235,IRON,'recoil',[-.30,.55,0]);s.tag(fine,{detail:1});},[.58,.72,0]);
}
function emu(s,{forager=false,gunner=false}={}){
 const coat=forager?'#9c8b69':'#786c55';
 s.orb([-.10,.78,0],[.85,.78,.61],coat);
 // Shaggy plumage hangs off the back in layers; the legs bend at the knee.
 const shag=s.parts.length;for(const [x,y] of [[-.42,.72],[-.18,.62],[-.48,.95]])s.orb([x,y,0],[.40,.30,.66],forager?'#8c7c5c':'#695f4d');s.tag(shag,{detail:1});
 for(const z of [-.18,.18]){const m=z<0?'strideL':'strideR',k=z<0?'kneeL':'kneeR',pivot=[0,.60,z];s.taper([0,.60,z],[.13,.30,z],.06,'#514a38',m,pivot);s.orb([.13,.29,z],[.09,.10,.09],'#514a38',k,pivot);s.taper([.13,.28,z],[.03,.05,z],.043,'#514a38',k,pivot);s.poly([.17,.04,z],[-.17,-.04,.20,-.04,.27,.04,.06,.08,-.20,.05],.15,DARK,k,pivot);}
 s.taper([.13,1.05,0],[.30,1.67,0],.14,coat,'bob');s.orb([.35,1.70,0],[.31,.28,.25],DARK,'bob');s.poly([.56,1.65,0],[-.10,-.04,.11,-.04,.16,.04,.02,.07,-.14,.04],.13,'#bca77c','bob');
 for(const z of [-.12,.12])s.orb([.43,1.75,z],[.06,.06,.03],'#efb94b','bob');
 for(const z of [-.30,.30])s.orb([-.22,.87,z],[.56,.48,.12],coat,'sway',[-.11,1.06,z]);
 for(const z of [-1,1])s.poly([-.22,.92,z*.13],[-.39,0,.20,.06,.42,.20,-.12,.16],.025,z<0?'#a69570':'#695f4d','sway',[-.11,1.06,z]);
 // A webbing harness in the faction's colour with a brass badge.
 s.box([-.07,1.02,0],[.28,.11,.67],P);const fine=s.parts.length;s.rod([.18,1.10,-.22],[.18,.70,.24],.03,P);s.orb([.22,.92,0],[.08,.08,.05],GOLD);s.tag(fine,{detail:1});
 s.tag(0,{material:'organic'});for(const p of s.parts)if(p.color===P)p.material='cloth';
 if(forager){s.box([-.37,.72,.38],[.30,.35,.20],WOOD);tool(s,'pick',{x:.28,y:.75,z:.35});}
 if(gunner){s.box([.16,.94,.39],[.87,.10,.11],WOOD);s.box([.79,.96,.39],[.56,.06,.065],IRON);s.rod([-.2,.9,0],[.30,.94,.39],.07,DARK);s.part('cylinder',[.34,1.04,.39],[.23,.23,.05],DARK,'recoil',[.34,1.04,.39],[Math.PI/2,0,0]);}
}
function kublai(s,body){
 if(['archer','keshig','horseman','camel'].includes(body)){
  const keshig=body==='keshig',camel=body==='camel';
  mounted(s,{coat:keshig?'#513c30':camel?'#b39a65':body==='horseman'?'#7a5236':'#986646',camel,mane:camel?'#8a7048':FUR,armor:keshig?{lamellar:true}:null,
   rider:{coat:keshig?IRON:P,hat:keshig?'conical':'fur',armor:keshig,frame:'steppe',cloak:keshig?P:null},weapon:body==='archer'?'bow':body==='horseman'?'sword':camel?'staff':'spear',standard:keshig});
  // Saddlebags on the camel; a bow case and a quiver on the archer's horse.
  if(camel){const bags=s.parts.length;for(const z of [-.36,.36])s.box([-.10,1.10,z],[.42,.40,.16],'#a27b50');s.rod([-.10,1.32,-.36],[-.10,1.32,.36],.03,LEATHER);mat(s,bags,'leather');}
  else if(body==='archer'){const kit=s.parts.length;s.box([-.52,.82,.28],[.30,.36,.10],LEATHER);s.box([.02,.80,-.28],[.42,.20,.08],LEATHER);mat(s,kit,'leather',1);}
  return;
 }
 if(body==='trebuchet'){
  cart(s);for(const z of [-.29,.29]){s.rod([-.51,.38,z],[0,1.39,z],.11,WOOD);s.rod([.55,.38,z],[0,1.39,z],.11,WOOD);}
  s.rod([0,1.39,-.36],[0,1.39,.36],.07,IRON);
  // The beam swings on its axle: a counterweight box bound in iron at one end, a sling and its stone at the other.
  s.rod([-.40,1.03,0],[.65,1.85,0],.11,WOOD,'strike',[0,1.39,0]);s.box([-.40,.96,0],[.36,.43,.37],'#8b8b7d','strike',[0,1.39,0]);
  const bands=s.parts.length;for(const dy of [-.12,.12])s.box([-.40,.96+dy,0],[.38,.04,.39],IRON,'strike',[0,1.39,0]);s.tag(bands,{detail:1});
  s.rod([.65,1.85,0],[.94,1.25,0],.025,IVORY,'strike',[0,1.39,0]);s.orb([.95,1.20,0],[.14,.13,.14],'#9d927c','strike',[0,1.39,0]);
  const winch=s.parts.length;s.part('cylinder',[-.62,.55,0],[.26,.26,.50],WOOD,'',[-.62,.55,0]);s.rod([-.62,.55,0],[-.40,.96,0],.015,IVORY);s.tag(winch,{detail:1});
  banner(s,P,-.54,1.3);return;
 }
 human(s,{coat:body==='shaman'?'#354b69':body==='artisan'?'#8b7860':P,hat:'fur',robe:body==='shaman',frame:'steppe'});
 if(body==='artisan'){tool(s,'pick');const apron=s.parts.length;s.poly([.22,.62,0],[-.03,-.26,.03,-.26,.04,.24,-.04,.24],.40,LEATHER);s.box([-.30,.85,-.32],[.26,.30,.18],LEATHER);mat(s,apron,'leather',1);}
 if(body==='falconer'){
  // A gauntlet, and a falcon with folded wings, a hooked beak and a barred tail.
  s.rod([.07,1.14,.31],[.52,1.23,.31],.14,LEATHER);s.orb([.52,1.36,.31],[.22,.28,.18],'#654d37');s.orb([.58,1.52,.31],[.13,.12,.12],'#654d37');
  s.poly([.66,1.50,.31],[-.03,.02,.05,0,.02,-.05],.04,GOLD);
  for(const z of [-.08,.08])s.poly([.46,1.36,.31+z],[-.08,.12,.10,.10,-.02,-.20,-.16,-.18],.02,'#8a6a4a');
  s.poly([.38,1.22,.31],[-.02,.06,.06,.04,-.10,-.14],.03,IVORY);s.box([.68,1.46,.31],[.1,.05,.07],GOLD);
 }
 if(body==='shaman'){
  // A skin drum, a staff hung with ribbons, and a spirit flame that burns above it.
  s.part('cylinder',[.37,.98,.44],[.43,.43,.15],IVORY);s.part('cylinder',[.37,.98,.42],[.48,.48,.07],WOOD);tool(s,'staff',{x:-.25,y:.9,z:-.37});
  const ribbons=s.parts.length;for(const dz of [-.05,.05])s.poly([-.03,1.70,-.37+dz],[-.02,0,.02,0,.06,-.30,.02,-.30],.02,dz<0?P:S,'sway',[-.02,1.72,-.37]);mat(s,ribbons,'cloth',1);
  lamp(s,[-.01,1.98,-.37],.07,{c:'#9fe8ff',halo:3.8,stretch:[1,1.5,1]});
 }
}
function arjuna(s,body){
 if(body==='chariot'){
  horse(s,{coat:'#e9e1c9',compact:true,mane:IVORY,tack:false});const start=s.parts.length;horse(s,{coat:'#d5cdbb',compact:true,mane:IVORY,tack:false});s.transform(start,.82,[.15,0,-.47]);
  const yoke=s.parts.length;s.box([.46,1.02,-.20],[.14,.10,.86],GOLD);for(const z of [-.04,-.44])s.box([.46,.92,z],[.20,.24,.30],P);s.tag(yoke,{detail:1});s.parts[yoke+1].material='cloth';s.parts[yoke+2].material='cloth';
  const a=s.parts.length;human(s,{coat:P,hat:'crown',trim:GOLD,skin:'#b77f59',detail:false});s.transform(a,.67,[-.95,.54,0]);
  // A gilded car with a carved front, wheels shod in gold and a parasol over the archer.
  s.box([-.94,.43,0],[.78,.17,.86],GOLD);for(const z of [-.48,.48]){wheel(s,[-.98,.30,z],.30,{c:GOLD,rim:'#a88a4a',hub:null});s.poly([-1.15,.51,z],[-.18,0,.52,0,.43,.45,-.18,.34],.07,P);}
  const car=s.parts.length;s.poly([-.56,.70,0],[-.06,-.18,.06,-.18,.10,.18,0,.30,-.10,.18],.70,GOLD);s.rod([-1.20,.50,0],[-1.20,2.0,0],.03,GOLD);s.lathe([-1.20,1.98,0],[[0,.42,.42],[.10,.36,.36],[.22,.04,.04]],P);s.tag(car,{detail:1});s.parts[car+2].material='cloth';
  s.rod([-.80,.48,0],[.50,.60,0],.055,GOLD);tool(s,'bow',{x:-.6,y:1.40,z:.35});
  lamp(s,[-1.20,2.22,0],.04,{c:GOLD,halo:3.6});return;
 }
 if(body==='elephant'){
  elephant(s);
  // A caparison in the faction's colour with gold borders, a gilded headplate, gold-capped tusks and a canopied howdah.
  const dress=s.parts.length;for(const z of [-.47,.47]){s.poly([-.10,1.10,z],[-.62,.40,.62,.40,.56,-.34,.30,-.44,0,-.34,-.30,-.44,-.56,-.34],.03,P);s.box([-.10,1.50,z*1.02],[1.24,.06,.02],GOLD);}mat(s,dress,'cloth');s.parts[dress+1].material='brass';s.parts[dress+3].material='brass';
  s.box([-.05,1.40,0],[.98,.08,.94],P);s.box([-.10,1.62,0],[.65,.35,.66],GOLD);
  const howdah=s.parts.length;for(const x of [-.38,.18])for(const z of [-.28,.28])s.rod([x,1.78,z],[x,2.20,z],.03,GOLD);s.poly([-.10,2.22,0],[-.40,0,.40,0,.20,.20,0,.26,-.20,.20],.66,P);s.tag(howdah,{detail:1});s.parts[howdah+4].material='cloth';
  s.box([-.10,1.80,0],[.56,.04,.56],P);
  const head=s.parts.length;s.poly([.98,1.40,0],[-.10,.16,.10,.16,.06,-.20,-.06,-.20],.40,GOLD);for(const z of [-.24,.24])s.orb([1.31,1.00,z*1.2],[.06,.06,.06],GOLD);s.tag(head,{material:'brass'});
  lamp(s,[1.07,1.46,0],.05,{halo:3.4});
  const a=s.parts.length;human(s,{hat:'turban',coat:P,trim:GOLD,skin:'#b77f59'});s.transform(a,.42,[-.08,1.64,0]);return;
 }
 human(s,{coat:body==='sage'?'#dfb76d':body==='healer'?'#dfd4bb':body==='servant'?'#b5a586':P,skin:body==='giant'?'#625377':'#b77f59',hat:body==='servant'?'turban':body==='sage'?'none':'crown',trim:GOLD,robe:['sage','healer','servant'].includes(body),frame:'arjuna',cloak:body==='mace'?S:null});
 if(body==='archer'){tool(s,'bow');s.rod([.26,1.04,-.46],[.26,1.04,.46],.045,GOLD);lamp(s,[.89,.90,.37],.035,{c:GOLD,halo:4,detail:1});}
 if(body==='mace'){tool(s,'mace');shield(s,GOLD);}
 if(body==='sage'){
  tool(s,'staff');s.rod([-.22,.84,-.40],[-.22,1.82,-.40],.035,GOLD);s.box([.02,1.76,-.40],[.42,.27,.025],IVORY,'sway',[-.22,1.82,-.40]);
  // A halo of light behind the head.
  streak(s,[-.14,1.48,0],[.05,.66,.66],0,{c:GOLD});
 }
 if(body==='healer'){s.orb([.42,.85,.37],[.40,.46,.38],'#a8683f');s.box([.42,1.12,.37],[.22,.05,.20],GOLD);for(const dz of [-.06,.06])s.poly([.42,1.20,.37+dz],[-.03,0,.03,0,0,.16],.03,'#7fbf6a');lamp(s,[.42,1.16,.37],.06,{c:'#a6ffc0',halo:3.6});}
 if(body==='servant'){tool(s,'pick');s.box([-.33,.75,-.35],[.40,.52,.28],WOOD);}
 if(body==='giant'){
  s.transform(0,1.22,[0,0,0]);
  // A stone club, and bands of gold on the arms.
  s.rod([.20,1.10,.40],[.52,1.48,.40],.10,WOOD,'strike',[.12,1.36,.3]);s.orb([.57,1.51,.36],[.65,.60,.55],'#777c83','strike',[.12,1.36,.3]);
  const bands=s.parts.length;for(const z of [-.38,.38])s.box([.10,1.22,z],[.20,.06,.20],GOLD);s.tag(bands,{detail:1});
 }
}
function templar(s,body){
 if(body==='ballista'){
  cart(s);s.box([0,.62,0],[1.17,.19,.28],WOOD);s.rod([.35,.69,-.65],[.35,.69,.65],.13,DARK);s.rod([-.55,.73,0],[.88,.73,0],.055,IRON);s.rod([.35,.69,-.65],[-.48,.73,0],.022,IVORY);s.rod([-.48,.73,0],[.35,.69,.65],.022,IVORY);s.poly([.91,.73,0],[-.13,-.06,-.13,.06,.15,0],.06,GOLD);
  // Torsion skeins of twisted rope at each arm, and a winch at the tail.
  const fine=s.parts.length;for(const z of [-.18,.18])s.part('cylinder',[.35,.69,z],[.16,.16,.20],'#c8b98f','',[.35,.69,z],[Math.PI/2,0,0]);s.part('cylinder',[-.52,.62,0],[.20,.20,.34],WOOD);s.box([-.20,.74,0],[.40,.03,.32],RED);s.tag(fine,{detail:1});
  return;
 }
 if(['marshal','turcopole'].includes(body)){
  const marshal=body==='marshal';
  mounted(s,{coat:marshal?'#c7c7ba':'#946c4c',armor:marshal?{c:IVORY,trim:RED,plate:true}:null,rider:{coat:marshal?IVORY:'#9a7e5d',hat:marshal?'greathelm':'conical',armor:true,frame:'templar',cloak:marshal?IVORY:null},weapon:marshal?'spear':'bow',standard:marshal});
  // The cross on the caparison and the rider's mantle.
  if(marshal){const cross=s.parts.length;for(const z of [-.285,.285]){s.box([-.10,.82,z],[.07,.30,.02],RED);s.box([-.10,.86,z],[.24,.07,.02],RED);}mat(s,cross,'cloth');}
  else s.box([-.05,.85,-.32],[.48,.34,.03],'#b7323f');
  return;
 }
 const healer=body==='hospitaller',worker=body==='acolyte',c=healer?'#27333c':body==='sergeant'?'#7f7767':IVORY;
 human(s,{coat:c,hat:worker?'hood':body==='crusader'?'greathelm':'helmet',trim:healer?IVORY:S,armor:!worker,robe:worker||healer,frame:'templar',cloak:body==='crusader'?IVORY:healer?'#27333c':null,emblem:body==='crusader'?RED:healer?IVORY:null});
 if(!worker){s.box([.25,1.04,0],[.025,.32,.055],healer?IVORY:'#ad303d');s.box([.27,1.09,0],[.025,.055,.24],healer?IVORY:'#ad303d');}
 if(body==='crossbowman'){tool(s,'crossbow');const pavise=s.parts.length;s.box([-.36,.95,0],[.06,.70,.46],IVORY);s.box([-.395,1.00,0],[.02,.40,.07],RED);s.box([-.395,1.06,0],[.02,.07,.28],RED);s.tag(pavise,{detail:1});}
 else if(healer){tool(s,'book');s.box([-.27,.84,.26],[.25,.29,.22],WOOD);tool(s,'lantern',{x:.25,y:.95,z:-.37});}
 else if(worker)tool(s,'hammer');
 else{tool(s,body==='sergeant'?'spear':'sword');shield(s,body==='sergeant'?WOOD:IVORY,body==='crusader');}
}
function atlantis(s,body){
 if(body==='engine'){
  cart(s,{cannon:true,culture:3});
  // A steam engine on the gun carriage: a riveted boiler, a stack with a crown, a firebox that glows, gauges and pipes.
  s.part('cylinder',[-.30,.68,0],[.57,.57,.57],'#be9b58');s.rod([-.4,.8,0],[-.4,1.48,0],.14,DARK);s.lathe([-.4,1.40,0],[[0,.10,.10],[.08,.16,.16],[.12,.12,.12]],IRON);s.box([-.6,.70,-.42],[.24,.38,.03],P);
  const fine=s.parts.length;for(const z of [-.20,.20])s.part('cylinder',[-.30,.68,z],[.60,.60,.03],IRON);s.part('cylinder',[-.10,.96,.18],[.12,.12,.04],IVORY,'',[-.10,.96,.18],[0,Math.PI/2,0]);s.rod([-.10,.90,.25],[.20,.62,.30],.03,GOLD);s.tag(fine,{detail:1});
  lamp(s,[-.58,.55,0],.09,{c:FIRE,halo:2.6,stretch:[.6,.8,1.6]});
  return;
 }
 if(body==='ship'){vessel(s,'steamer');return;}
 if(body==='flyer'){
  // An ornithopter: a wooden hull, canvas wings on ribs, a turning screw, a pilot in goggles and a burner glowing behind.
  s.orb([0,.75,0],[.80,.50,.46],WOOD,'bob');
  for(const z of [-1,1]){s.poly([0,.82,z*.22],[-.51,0,.44,0,.1,z*.95,-.4,z*.82],.045,IVORY,'sway',[0,.82,z*.22],[Math.PI/2,0,0]);const rib=s.parts.length;s.rod([.30,.84,z*.22],[.06,.84,z*.95],.025,WOOD,'sway',[0,.82,z*.22]);s.rod([-.10,.84,z*.22],[-.34,.84,z*.80],.025,WOOD,'sway',[0,.82,z*.22]);s.tag(rib,{detail:1});}
  s.rod([.55,.75,-.34],[.55,.75,.34],.065,GOLD,'spin',[.55,.75,0]);s.part('cylinder',[.48,.75,0],[.16,.16,.10],GOLD,'',[.48,.75,0],[0,Math.PI/2,0]);
  s.orb([-.15,1,0],[.32,.25,.28],P,'bob');s.orb([-.06,1.06,0],[.10,.06,.20],'#9cccd3','bob');
  lamp(s,[-.42,.74,0],.06,{c:FIRE,m:'bob',halo:3});return;
 }
 if(body==='automaton'){
  human(s,{coat:'#b89a63',skin:IRON,hat:'helmet',armor:true,frame:'atlantis'});
  // Clockwork laid open: a turning gear at the chest, a glowing core and lit eyes.
  s.part('cylinder',[.25,1.05,0],[.24,.24,.06],GOLD,'spin',[.25,1.05,0],[0,Math.PI/2,0]);
  const teeth=s.parts.length;for(const a of [0,Math.PI/4,Math.PI/2,3*Math.PI/4])s.box([.25,1.05,0],[.02,.30,.04],GOLD,'spin',[.25,1.05,0],[a,0,0]);s.tag(teeth,{detail:1});
  lamp(s,[.24,1.22,0],.05,{c:LIGHT,halo:3.4});tool(s,'spear');return;
 }
 human(s,{coat:body==='mystery'?'#414c56':body==='pioneer'?'#957b58':'#466d7d',hat:body==='mystery'?'hood':'tricorn',trim:GOLD,robe:body==='fellow',frame:'atlantis',cloak:body==='mystery'?'#2c343c':null});
 if(body==='pioneer'){tool(s,'hammer');s.box([-.38,.76,-.35],[.45,.56,.31],WOOD);const kit=s.parts.length;s.part('cylinder',[-.38,1.10,-.35],[.18,.18,.40],IVORY,'',[-.38,1.10,-.35]);s.tag(kit,{detail:1});}
 if(body==='gunner'){tool(s,'musket');s.rod([.30,1.06,-.46],[.30,1.06,.46],.055,IRON);s.box([-.22,.85,-.37],[.32,.42,.20],WOOD);}
 if(body==='fellow'){tool(s,'book');s.box([.38,1.18,.37],[.42,.10,.50],IVORY);lamp(s,[.34,1.40,.37],.10,{c:'#b6e9e2',halo:2.6,stretch:[1,1.3,1]});}
 if(body==='mystery'){tool(s,'lantern');s.rod([-.20,.82,-.36],[-.20,1.78,-.36],.04,IRON);s.box([.03,1.68,-.36],[.46,.27,.025],DARK,'sway',[-.20,1.78,-.36]);}
}
function fleet(s,body){
 if(body==='junk'){vessel(s,'junk');return;}
 if(body==='qilin'){
  // The fleet brought a giraffe home: a real animal, rather than a generic fantasy unicorn, in a silk saddle cloth.
  horse(s,{coat:'#bb9957',giraffe:true,mane:'#705335',cloth:P});
  const bells=s.parts.length;for(const z of [-.12,.12])s.orb([.62,1.30,z],[.06,.06,.06],GOLD);s.tag(bells,{detail:1});return;
 }
 if(body==='rockets'){
  cart(s);s.box([-.1,.77,0],[.86,.55,.72],'#a34c36');for(const z of [-.23,0,.23])for(const y of [.65,.85,1.05])s.rod([-.55,y,z],[.70,y+.25,z],.07,WOOD);s.box([.50,.79,0],[.035,.54,.68],DARK);
  // Fire-arrows ready in their racks: each fuse glows at the mouth.
  const caps=s.parts.length;for(const x of [-.50,.30])s.box([x,1.06,0],[.06,.08,.76],GOLD);s.tag(caps,{detail:1});
  for(const z of [-.23,.23])lamp(s,[.72,1.31,z],.035,{c:FIRE,halo:3.4});
  return;
 }
 const worker=body==='porter';human(s,{coat:worker?'#a58b65':body==='chronicler'?'#374e65':P,skin:'#c9a078',hat:worker?'straw':body==='chronicler'?'cap':'conical',trim:'#7b3338',armor:body==='marine',robe:body==='chronicler',frame:'fleet'});
 if(worker){tool(s,'pick');s.rod([-.43,1.1,-.4],[.45,1.1,-.4],.055,WOOD);for(const x of [-.4,.4])s.box([x,.72,-.4],[.27,.35,.28],'#aa865b');const ropes=s.parts.length;for(const x of [-.4,.4])s.rod([x,.90,-.4],[x,1.1,-.4],.015,IVORY);s.tag(ropes,{detail:1});}
 if(body==='marine'){tool(s,'spear');shield(s,'#9d513d');}
 if(body==='crossbow'){tool(s,'crossbow');const mag=s.parts.length;s.box([.38,1.00,.37],[.36,.10,.07],WOOD);s.rod([.16,.92,.37],[.30,1.05,.37],.025,WOOD);s.tag(mag,{detail:0,aim:'weapon',pivot:[.25,.9,.37]});}
 if(body==='signal'){
  // A paper lantern, lit red, on a carrying pole; the banner behind.
  s.rod([.20,.86,.37],[.58,1.60,.37],.03,WOOD);s.rod([.58,1.60,.37],[.58,1.40,.37],.012,DARK);
  lamp(s,[.58,1.28,.37],.13,{c:'#ff7a4a',m:'sway',pivot:[.58,1.60,.37],halo:2.4,stretch:[1,1.25,1]});
  const caps=s.parts.length;for(const dy of [-.17,.17])s.part('cylinder',s.about([.58,1.28+dy,.37],[.58,1.60,.37],[Math.PI/2,0,0]),[.16,.16,.03],GOLD,'sway',[.58,1.60,.37],[Math.PI/2,0,0]);s.tag(caps,{detail:1});
  banner(s,S,-.25,1.9);
 }
 if(body==='chronicler'){tool(s,'book');s.box([-.23,1,-.4],[.21,.42,.18],WOOD);const brush=s.parts.length;s.rod([.30,.95,-.36],[.42,1.20,-.36],.02,WOOD);s.orb([.43,1.22,-.36],[.04,.06,.04],DARK);s.tag(brush,{detail:1});}
}
function manifesto(s,body){
 if(body==='tank'){cart(s,{tank:true});return;}
 if(body==='katyusha'){cart(s,{truck:true,rockets:true});return;}
 if(body==='cavalry'){mounted(s,{coat:'#6c4c35',rider:{coat:'#756e4b',hat:'cap',trim:P,frame:'industrial',cloak:'#5d5a3e'},weapon:'sword',standard:true});return;}
 human(s,{coat:body==='stakhanovite'?'#637581':body==='mole'?'#514c43':'#727650',hat:body==='mole'?'hood':body==='soldier'?'helmet':'cap',trim:body==='mole'?DARK:P,robe:body==='mole',frame:'industrial',cloak:body==='sniper'?'#6d7253':null});
 if(body==='stakhanovite'){tool(s,'pick');lamp(s,[.24,1.57,0],.05,{c:WARM,halo:3.8});s.box([-.3,.88,-.32],[.28,.4,.22],WOOD);}
 else if(body==='agitator'){
  banner(s,P,-.29,1.95);tool(s,'book');s.box([.12,1.1,.32],[.06,.20,.045],GOLD);
  // A loudhailer raised to the crowd.
  const horn=s.parts.length;s.lathe(s.about([.28,1.0,-.33],[0,1.15,-.31],[0,0,-Math.PI/2+.3]),[[0,.05,.05],[.30,.16,.16]],IRON,'strideR',[0,1.15,-.31],[0,0,-Math.PI/2+.3]);s.tag(horn,{material:'iron'});
 }
 else if(body==='mole'){tool(s,'book');lamp(s,[.24,1.50,0],.045,{c:WARM,halo:3.8});}
 else {
  tool(s,'rifle');s.box([-.18,.90,-.35],[.27,.34,.17],WOOD);
  if(body==='sniper'){s.box([-.24,1.04,0],[.12,.47,.56],'#9a9d7a');const scope=s.parts.length;s.tube([.27,1.07,.37],[.57,1.07,.37],.03,DARK,'recoil',[.25,.9,.37]);s.tag(scope,{aim:'weapon',pivot:[.25,.9,.37]});lamp(s,[.58,1.07,.37],.022,{c:LIGHT,m:'recoil',pivot:[.25,.9,.37],halo:3,detail:1});for(const p of s.parts.slice(-2))p.aim='weapon';}
  if(body==='soldier'){const kit=s.parts.length;s.orb([-.30,.70,.20],[.12,.16,.10],'#4e5340');s.part('cylinder',[-.28,1.05,0],[.30,.30,.10],'#565a44','',[-.28,1.05,0],[0,Math.PI/2,0]);s.tag(kit,{detail:1});}
 }
}
// Null Pointer Society: dark polished alloy under faction-coloured light, in the manner of a science-fiction RTS.
// Alloy, emissive and halo are viewer materials (gpu/unit-surfaces.js): alloy armour mirrors the sky and carries an
// edge light in the faction's colour; emissive parts light themselves and flare as the unit strikes or is struck; a
// halo is the soft glow around a light. Halos are drawn last (finish), so they glow over the body they sit on.
const NGUN='#4c5b66',NPLATE='#35434d',NSUIT='#1b2329',NVOID='#0f161a',NHOT='#eafffb';
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
  const lens=s.parts.length;s.part('cylinder',s.about([.22,1.02,0],[0,1.02,0],[0,Math.PI/2,0]),[.30,.30,.10],NPLATE,'',[0,1.02,0],[0,Math.PI/2,0]);alloy(s,lens);
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
 if(body==='carronade'){
  cart(s,{cannon:true});
  // Shot piled by the trail, a rammer along the carriage and a linstock whose slow match glows.
  const fine=s.parts.length;for(const [x,z] of [[-.62,-.10],[-.62,.10],[-.54,0]])s.orb([x,.44,z],[.13,.13,.13],DARK);s.orb([-.59,.54,.0],[.13,.13,.13],DARK);s.rod([-.70,.42,.34],[.55,.42,.34],.035,WOOD);s.tag(fine,{detail:1});
  s.rod([-.74,.38,-.30],[-.56,1.02,-.30],.03,WOOD);lamp(s,[-.55,1.05,-.30],.035,{c:FIRE,halo:3.6});
  return;
 }
 if(body==='merchantman'){vessel(s,'indiaman');return;}
 if(body==='dragoon'){
  mounted(s,{coat:'#c4c8c4',mane:'#8d8f8a',rider:{coat:'#a54a43',hat:'tricorn',trim:IVORY,frame:'nations',cloak:'#2f3d4c'},weapon:'musket'});
  // A sabre in its scabbard and a pistol holster at the saddle.
  const kit=s.parts.length;s.rod([-.10,.92,-.33],[.42,.62,-.36],.035,IRON);s.box([-.12,.94,-.33],[.08,.06,.06],GOLD);s.box([.30,.98,.33],[.20,.12,.08],LEATHER);mat(s,kit,'iron',1);s.parts[kit+2].material='leather';
  return;
 }
 const coat=body==='highlander'?'#b65b4d':body==='pinmaker'?'#927858':body==='smuggler'?'#4d625c':body==='shopkeeper'?'#6a5a7a':'#597382';
 human(s,{coat,hat:body==='highlander'?'beret':body==='pinmaker'?'cap':'tricorn',trim:IVORY,frame:'nations'});
 if(body==='highlander'){
  // A belted plaid: the kilt pleated in tartan, a sporran, and the plaid thrown over the shoulder.
  const kilt=s.parts.length;s.lathe([0,.40,0],[[0,.35,.33],[.16,.31,.29],[.34,.25,.23]],'#496254');mat(s,kilt,'cloth');
  const fine=s.parts.length;s.lathe([0,.50,0],[[0,.33,.31],[.035,.33,.31]],'#b85e4b');s.lathe([0,.62,0],[[0,.29,.27],[.025,.29,.27]],IVORY);s.rod([.25,1.18,-.22],[.25,.76,.24],.075,'#496254');mat(s,fine,'cloth',1);
  const sporran=s.parts.length;s.box([.31,.62,0],[.05,.17,.14],LEATHER);s.orb([.33,.66,0],[.06,.06,.06],FUR);mat(s,sporran,'leather',1);
  tool(s,'musket');
 }else if(body==='exciseman'){
  tool(s,'musket');tool(s,'lantern',{x:.25,y:.9,z:-.37});
  // A brass gorget at the throat and a white cross-belt.
  const fine=s.parts.length;s.lathe([0,1.16,0],[[0,.20,.18],[.06,.23,.21]],GOLD);s.rod([.24,1.12,.22],[.24,.74,-.22],.04,IVORY);s.tag(fine,{detail:1});s.parts[fine].material='brass';
 }
 else if(body==='pinmaker'){
  tool(s,'hammer');s.box([-.36,.77,-.35],[.38,.55,.28],WOOD);s.box([-.25,.76,.30],[.04,.40,.07],GOLD);
  const apron=s.parts.length;s.poly([.22,.62,0],[-.03,-.30,.03,-.30,.04,.26,-.04,.26],.40,LEATHER);mat(s,apron,'leather',1);
 }
 else if(body==='shopkeeper'){
  tool(s,'book');s.box([-.27,.80,-.35],[.42,.40,.28],WOOD);
  // An apron, a fat purse at the belt and a quill in the ledger.
  const fine=s.parts.length;s.poly([.22,.62,0],[-.03,-.30,.03,-.30,.04,.26,-.04,.26],.38,IVORY);s.orb([.12,.70,-.29],[.13,.15,.11],GOLD);s.rod([.38,.96,.30],[.30,1.20,.44],.015,IVORY);s.tag(fine,{detail:1});
  s.parts[fine].material='cloth';s.parts[fine+1].material='brass';
 }
 else{
  // The smuggler: a cutlass, a dark lantern and a keg of brandy across the back.
  tool(s,'sword');tool(s,'lantern',{x:.25,y:.9,z:-.37});
  s.tube([-.34,.92,-.26],[-.34,.92,.26],.17,WOOD);const bands=s.parts.length;for(const z of [-.16,.16])s.tube([-.34,.92,z-.02],[-.34,.92,z+.02],.18,IRON);s.tag(bands,{detail:1});
 }
}
function garden(s,body){
 if(body==='eye'){
  // A floating eye: a lit iris under petal lids, roots trailing below and a spore light under it.
  s.orb([0,1,0],[.65,.42,.44],IVORY,'bob');const iris=s.parts.length;s.orb([.32,1,0],[.06,.29,.27],P,'bob');lit(s,iris);s.orb([.36,1,0],[.045,.16,.14],DARK,'bob');
  lamp(s,[.33,1,0],.07,{m:'bob',pivot:[0,1,0],halo:3.2,stretch:[.3,1,1]});
  for(const z of [-1,1]){s.poly([-.12,.95,z*.23],[-.4,0,0,.12,.25,0,0,-.14],.08,S,'sway',[-.12,.95,z*.23]);s.poly([-.16,1.15,z*.27],[-.28,0,.12,.08,.31,.02,.03,-.14],.025,'#d6a0a8','sway',[-.12,.95,z*.23]);}
  const roots=s.parts.length;for(const [x,z] of [[-.18,-.12],[-.08,.14],[-.28,.04]])s.taper([x,.72,z],[x-.10,.30,z*1.6],.035,'#587450','sway',[x,.72,z]);s.tag(roots,{detail:1});
  lamp(s,[-.12,.22,0],.05,{c:'#e9ff9a',m:'bob',pivot:[0,1,0],halo:3.6});
  s.tag(0,{material:'organic'});return;
 }
 if(body==='thorn'){
  s.orb([0,.22,0],[.75,.43,.66],'#6a8760');for(const z of [-.23,.23])s.taper([-.24,.27,z],[.15,.95,z],.12,'#587450');s.taper([.15,.95,0],[.73,1.28,0],.17,P,'strike',[.15,.95,0]);s.poly([.73,1.28,0],[0,0,-.09,.28,.12,.10,.30,.12],.17,IVORY);
  for(const z of [-.31,.31])s.poly([.15,.62,z],[-.30,0,.14,.08,.34,.22,.02,.16],.028,'#91aa75','sway',[.15,.95,z]);s.orb([-.18,.71,0],[.60,.50,.52],P);
  // Thorns down the striking stem, and a sting that glows at its tip.
  const thorns=s.parts.length;for(const t of [.3,.55,.8])s.poly([.15+.58*t,.95+.33*t+.09,0],[-.04,0,.04,0,-.02,.13],.05,IVORY,'strike',[.15,.95,0]);s.tag(thorns,{detail:1});
  lamp(s,[.98,1.40,0],.05,{m:'strike',pivot:[.15,.95,0],halo:3.4});
  const bulbs=s.parts.length;for(const z of [-.20,.20])s.orb([-.30,1.02,z],[.12,.12,.12],P);lit(s,bulbs);
  s.tag(0,{material:'organic'});return;
 }
 if(body==='tree'){
  for(const z of [-.22,.22]){s.rod([0,.7,z],[z*.4,.08,z],.21,'#72523b',z<0?'strideL':'strideR',[0,.7,z]);s.rod([0,1.1,z],[.45,.68,z*2],.14,WOOD,'strike',[0,1.1,z]);
   // Roots for toes, and twig fingers.
   const fine=s.parts.length;s.poly([z*.4+.10,.05,z],[-.18,-.04,.22,-.04,.30,.03,-.10,.06],.20,'#5a3f2c',z<0?'strideL':'strideR',[0,.7,z]);s.taper([.45,.68,z*2],[.62,.58,z*2.3],.04,WOOD,'strike',[0,1.1,z]);s.tag(fine,{detail:1});}
  s.poly([0,.56,0],[-.24,0,.24,0,.19,.81,-.16,.81],.47,WOOD);s.orb([-.04,1.53,0],[.98,.65,.86],P);s.orb([.32,1.45,.17],[.58,.44,.55],'#91aa75');s.box([.22,1.25,-.12],[.035,.06,.08],GOLD);s.box([.22,1.25,.12],[.035,.06,.08],GOLD);
  for(const p of s.parts)if(p.color===P||p.color==='#91aa75')p.material='organic';
  // Eyes in the bark, and fruit that lights the crown.
  for(const z of [-.10,.10])lamp(s,[.235,1.06,z],.035,{c:'#e9ff9a',halo:2.8,stretch:[.6,1,1]});
  for(const [x,y,z] of [[.30,1.70,-.30],[-.40,1.80,.28],[.48,1.30,.42],[-.20,1.30,-.48]])lamp(s,[x,y,z],.05,{c:'#ffc76b',halo:3.2});
  return;
 }
 if(body==='statue'){
  // A walking statue: weathered stone, moss on the shoulders and runes that glow in the faction's colour.
  const start=s.parts.length;human(s,{coat:IVORY,skin:IVORY,legs:IVORY,boots:IVORY,hat:'none',trim:IVORY,robe:true,detail:false});mat(s,start,'stone');
  for(const p of s.parts.slice(start))p.color='#c9c2ae';
  const moss=s.parts.length;for(const z of [-.24,.24])s.orb([-.05,1.24,z],[.24,.10,.20],'#6a8760');s.orb([-.04,1.56,0],[.24,.08,.24],'#6a8760');mat(s,moss,'organic',1);
  const runes=s.parts.length;s.box([.24,1.00,0],[.02,.22,.04],P);s.box([.24,1.04,0],[.02,.04,.16],P);s.box([.22,.70,0],[.02,.14,.04],P);lit(s,runes);
  s.box([-.25,1.03,.30],[.09,.44,.41],'#c9c2ae');s.tag(s.parts.length-1,{material:'stone'});
  return;
 }
 human(s,{coat:'#8b9a72',skin:'#bd9d7d',hat:'straw',trim:P,frame:'garden'});
 tool(s,'pick');
 // A bloom on the crown, lit from inside.
 const bloom=s.parts.length;around(s,4,.09,1.50,(i,at,rot)=>s.poly(at,[-.07,0,.07,0,0,.17],.03,'#d3a7a4','sway',[0,1.50,0],rot));s.tag(bloom,{material:'organic',detail:1});
 lamp(s,[0,1.58,0],.05,{c:'#ffe58a',m:'sway',pivot:[0,1.50,0],halo:3.4});
}
// Highway to the Danger Node: every soldier is a Top Gun pilot in a leather flight jacket and mirrored aviators, lit like
// a flight deck at dusk. Mav carries a pistol and his helmet, Cold is blond with a carbine, Swan has the moustache, a
// radio headset and a kneeboard, and the nugget (a pilot on a first tour) works the deck in a ball cap with a wrench and
// a lit marshalling wand. Mav rides the motorbike, the nugget drives the tow tractor, and the parked jet is the Tomcat on
// its gear.
const NAVY='#7d868d',SAGE='#6f7a5a',WHITE='#e8e6e0',BOOTS='#2b2622',AMBER='#ffb347';
const PILOTS={mav:{coat:'#5b4632',hair:'#3a2a1e'},cold:{coat:'#3f3329',hair:'#d8c08a'},swan:{coat:'#6b4a33',hair:'#4a3526',tache:true},nugget:{coat:'#7a5a3c',hat:'cap'}};
// Mirrored aviators: two dark lenses on a brass frame, with a glint of the faction's colour across them.
function aviators(s,x=0){
 const lens=s.parts.length;for(const z of [-.08,.08])s.box([x+.19,1.43,z],[.04,.09,.13],'#1b262d');mat(s,lens,'glass');
 streak(s,[x+.21,1.44,0],[.06,.12,.42],0,{detail:1});
 const frame=s.parts.length;s.box([x+.19,1.455,0],[.02,.015,.05],GOLD);for(const z of [-.15,.15])s.box([x+.08,1.45,z],[.22,.015,.015],GOLD);mat(s,frame,'brass',2);
}
function pilot(s,body,{mounted=false}={}){
 const look=PILOTS[body],start=human(s,{coat:look.coat,hair:look.hair,tache:!!look.tache,hat:look.hat||'none',trim:'#2f2a24',legs:SAGE,boots:BOOTS,frame:'pilot',detail:false,mounted});
 for(const p of s.parts.slice(start))if(p.color===look.coat)p.material='leather';
 return start;
}
// The flight helmet: white, with a gold visor and a stripe lit in the faction's colour.
function helmet(s,at,{m='',pivot=at}={}){
 const shell=s.parts.length;s.orb(at,[.30,.28,.26],WHITE,m,pivot);mat(s,shell,'iron');
 const visor=s.parts.length;s.box([at[0]+.13,at[1]+.01,at[2]],[.05,.10,.20],'#c9a227',m,pivot);mat(s,visor,'glass');
 const stripe=s.parts.length;s.box([at[0],at[1]+.135,at[2]],[.26,.025,.05],P,m,pivot);lit(s,stripe);
}
function pistol(s,{x=.25,y=.9,z=.37}={}){
 const first=s.parts.length;s.muzzle=[x+.36,y+.03,z];s.muzzleJoint='weapon';s.muzzlePivot=[x,y,z];
 s.box([x+.14,y+.03,z],[.30,.08,.07],DARK);s.box([x+.02,y-.08,z],[.08,.17,.06],DARK);mat(s,first,'iron');
 s.tag(first,{detail:0,aim:'weapon',pivot:[x,y,z]});
}
function danger(s,body){
 if(body==='jet'){
  // The Tomcat on its gear with its wings spread, and the boarding ladder down.
  s.parts.push(...airSculpt(10,'fighter').parts.map(p=>p.motion==='sweep'?{...p,motion:''}:p));
  const gear=s.parts.length;for(const [x,z] of [[.80,0],[-.40,-.26],[-.40,.26]]){s.rod([x,.14,z],[x,.58,z],.035,IRON);s.part('cylinder',[x,.12,z],[.22,.22,.08],DARK);}mat(s,gear,'iron');
  const ladder=s.parts.length;s.rod([.56,.04,-.26],[.44,.80,-.17],.03,'#e0c04a');s.tag(ladder,{detail:1});
  return;
 }
 if(body==='tug'){
  // A yellow tow tractor with headlamps, a lit stripe and a turning amber beacon.
  const hull=s.parts.length;s.box([0,.32,0],[1.30,.26,.82],'#d9b23a');s.box([-.32,.62,0],[.34,.34,.40],DARK);s.box([-.32,.82,0],[.42,.05,.48],'#d9b23a');mat(s,hull,'iron');
  for(const x of [-.40,.40])for(const z of [-.44,.44])s.wheel([x,.20,z],.19,DARK);
  s.rod([.66,.34,0],[1.08,.30,0],.07,IRON,'strike',[.66,.34,0]);
  const stripe=s.parts.length;for(const z of [-.415,.415])s.box([0,.40,z],[1.10,.05,.02],P);lit(s,stripe);
  for(const z of [-.24,.24])lamp(s,[.66,.38,z],.04,{c:WARM,halo:3.2});
  s.rod([-.32,.85,0],[-.32,.98,0],.03,DARK);lamp(s,[-.26,1.0,0],.045,{c:AMBER,m:'spin',pivot:[-.32,1.0,0],halo:3.4});
  const driver=pilot(s,'nugget',{mounted:true});s.transform(driver,.55,[-.32,.44,0]);return;
 }
 if(body==='launcher'){
  cart(s,{rockets:true,culture:10});s.box([.42,.98,0],[.30,.04,.40],WHITE);
  const stripe=s.parts.length;for(const z of [-.375,.375])s.box([-.34,.50,z],[.80,.04,.02],P);lit(s,stripe);
  lamp(s,[.48,1.05,0],.04,{c:AMBER,m:'spin',pivot:[.42,1.05,0],halo:3.4});return;
 }
 if(body==='bike'){
  // Mav on his motorbike: red and black, a headlamp, a tail light and a lit stripe down the fairing.
  for(const x of [-.55,.55])s.wheel([x,.30,0],.28,DARK);
  const frame=s.parts.length;s.box([0,.48,0],[.95,.16,.16],RED);s.orb([.12,.66,0],[.45,.20,.26],RED);s.box([-.30,.62,0],[.40,.07,.24],DARK);mat(s,frame,'iron');
  s.taper([.55,.30,0],[.42,.86,0],.045,IRON);s.rod([.42,.86,-.26],[.42,.86,.26],.04,IRON);
  const stripe=s.parts.length;for(const z of [-.13,.13])s.box([.12,.66,z],[.40,.03,.02],P);lit(s,stripe);
  lamp(s,[.62,.72,0],.05,{c:WARM,halo:3.4});lamp(s,[-.52,.64,0],.035,{c:'#ff4a3a',halo:3});
  const rider=pilot(s,'mav',{mounted:true});s.transform(rider,.74,[-.14,.40,0]);
  return;
 }
 pilot(s,body);
 const off=[0,1.15,-.31];
 if(body==='mav')pistol(s);
 else if(body==='cold')tool(s,'rifle');
 else if(body==='swan'){
  // The backseater's radio headset and kneeboard.
  const set=s.parts.length;s.rod([.02,1.60,-.16],[.02,1.60,.16],.03,DARK);for(const z of [-.155,.155])s.capsule([.03,1.42,z],[.11,.13,.06],DARK);mat(s,set,'iron');
  const mic=s.parts.length;s.rod([.05,1.38,.16],[.20,1.31,.08],.015,DARK);mat(s,mic,'iron',1);
  const board=s.parts.length;s.box([.24,.86,-.37],[.20,.26,.03],IVORY,'strideR',off);s.box([.24,.99,-.37],[.06,.03,.04],GOLD,'strideR',off);s.tag(board,{detail:1});
  pistol(s);
 }else{
  // A wrench in one hand and a lit marshalling wand in the other.
  const wrench=s.parts.length,grip=[.25,.9,.37];s.rod([.25,.72,.37],[.33,1.12,.37],.045,IRON,'strike',grip);
  s.poly([.34,1.16,.37],[-.08,-.04,.08,-.04,.08,.10,.03,.10,.03,.02,-.03,.02,-.03,.10,-.08,.10],.05,IRON,'strike',grip);mat(s,wrench,'iron');
  s.rod([.22,.80,-.37],[.22,.90,-.37],.04,DARK,'strideR',off);lamp(s,[.22,1.08,-.37],.04,{m:'strideR',pivot:off,halo:2.2,stretch:[1,8,1]});
 }
}
function vessel(s,style,role='hull'){s.parts.push(...navalSculpt(style,role).parts);}
function landRig(seat,body){
 // Names recur between factions: Arjuna's archer is on foot while Kublai's is
 // mounted.  Bind the locomotion class to the authored chassis, never a name alone.
 if((seat===0&&['archer','keshig','horseman','camel'].includes(body))||(seat===1&&['chariot','elephant'].includes(body))||(seat===2&&['turcopole','marshal'].includes(body))||(seat===5&&body==='cavalry')||(seat===7&&body==='dragoon'))return 'rider';
 if(seat===10&&['bike','launcher','tug','jet'].includes(body))return 'vehicle';
 if(['trebuchet','ballista','engine','tank','katyusha','rockets','collector','carronade','truck'].includes(body))return 'vehicle';
 if(['ship','junk','merchantman'].includes(body))return 'ship';
 if(['qilin','tree','thorn','bug'].includes(body))return 'beast';
 if(['watch','bomb','eye','flyer'].includes(body))return 'hover';
 return 'biped';
}
// A house mark is deliberately structural, not a colour decal.  It sits above the
// common role body so a monochrome army still separates into eleven cultures at a glance.
function houseSilhouette(s,seat,body){
 const back=-.31;
 if(seat===0){ // Horse-tail standard and curved composite bow case.
  s.rod([back,.82,.27],[back,1.95,.27],.034,WOOD);s.poly([back,1.95,.27],[-.06,0,.06,0,0,.20],.03,GOLD);s.box([back,1.80,.27],[.14,.05,.14],GOLD);
  for(const [dx,dz] of [[-.05,-.03],[.05,-.03],[0,.05]])s.taper([back+dx,1.78,.27+dz],[back+dx*2.2,1.38,.27+dz*2.2],.03,'#2f2620','sway',[back,1.80,.27]);
 }else if(seat===1){ // Temple crown over a lotus collar.
  for(const z of [-.23,0,.23])s.poly([-.08,1.67,z],[-.12,0,.12,0,0,.26],.06,GOLD);
 }else if(seat===2){ // Gothic backplate and a tall cross finial.
  s.rod([back,.85,.30],[back,1.95,.30],.034,WOOD);s.box([back,1.92,.30],[.045,.24,.035],RED);s.box([back,1.94,.30],[.18,.045,.035],RED);
  const pennon=s.parts.length;s.poly([back-.17,1.62,.30],[-.17,.14,.17,.14,.17,-.14,0,-.05,-.17,-.14],.03,IVORY,'sway',[back,1.76,.30]);s.box([back-.17,1.63,.31],[.04,.20,.03],RED,'sway',[back,1.76,.30]);s.box([back-.17,1.66,.31],[.18,.04,.03],RED,'sway',[back,1.76,.30]);mat(s,pennon,'cloth');
 }else if(seat===3){ // Brass instrument frame: visibly Renaissance even on workers.
  s.part('cylinder',[back,.98,.30],[.42,.42,.055],GOLD,'spin',[back,.98,.30],[Math.PI/2,0,0]);lamp(s,[back,1.02,.30],.07,{c:LIGHT,halo:3});
 }else if(seat===4){ // Ming back banner with a curved roof cap.
  s.rod([back,.72,.29],[back,1.71,.29],.03,WOOD);s.poly([back,1.55,.29],[0,0,.34,.07,.26,.32,0,.32],.03,'#a34c36','sway',[back,1.71,.29]);s.box([back,1.72,.29],[.38,.045,.08],GOLD);
 }else if(seat===5){ // Industrial radiator and red star mast.
  for(const z of [-.08,.08])s.box([back,.96,.30+z],[.31,.46,.025],IRON);s.poly([back,1.55,.30],[0,.16,.04,.04,.15,.04,.06,-.03,.09,-.14,0,-.07,-.09,-.14,-.06,-.03,-.15,.04,-.04,.04],.025,P);lit(s,s.parts.length-1);
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
 }else if(seat===10){ // The flight helmet: in Mav's hand, slung at every other pilot's hip. Vehicles carry their own lights.
  if(body==='mav')helmet(s,[.12,.86,-.44],{m:'strideR',pivot:[0,1.15,-.31]});
  else if(PILOTS[body])helmet(s,[-.10,.70,-.36]);
 }else{ // Botanical glass petals: a rounded organic outline replaces a square pack.
  const petals=s.parts.length;for(const z of [-.26,0,.26])s.orb([back,.99,z],[.22,.08,.28],z?'#d3a7a4':IVORY,'sway',[back,.83,0]);s.rod([back,.78,0],[back,1.20,0],.025,'#587450');mat(s,petals,'organic');s.parts[petals+1].material='emissive';
 }
}
export function houseSculpt(id,flags=0,airRole='fighter'){
 if(!hasSculpt(id))return null;
 const [a,body]=id.split(':'),seat=Number(a),s=new Sculpt();
 if(flags&1){const model=airSculpt(seat,airRole);if(flags&2)model.parts.push({shape:'box',at:[-.33,.53,-.36],size:[.29,.30,.24],pivot:[-.33,.53,-.36],motion:'',color:'#b89458',on:[],material:'brass',detail:1});return {...model,rig:'air'};}
 [kublai,arjuna,templar,atlantis,fleet,manifesto,nullSociety,nations,(s,b)=>b==='truck'?cart(s,{truck:true,culture:8}):emu(s,{forager:b==='forager',gunner:b==='gunner'}),garden,danger][seat](s,body);
 houseSilhouette(s,seat,body);
 if(flags&2)s.box([-.33,.83,-.36],[.29,.30,.24],'#b89458');
 finish(s);
 return {parts:s.parts,sculpt:id,muzzle:s.muzzle,muzzleJoint:s.muzzleJoint,muzzlePivot:s.muzzlePivot,rig:landRig(seat,body)};
}
