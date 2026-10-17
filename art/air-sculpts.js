// Wings are a permanent body stat. The build selects a scout, fighter or strike silhouette.
import {Sculpt} from './sculpt.js';
const P='primary',S='secondary',WOOD='#785239',DARK='#26333a',IRON='#788b95',GOLD='#dab866',IVORY='#e6dcc2',RED='#b93b3c',LIGHT='#a3fff0';
import {AIR_ROLES} from './airframes.js';
export {AIR_ROLES,AIR_TITLES,AIR_BUILDS,airRole,airDefinition} from './airframes.js';

function wing(s,x,y,span,c,{feathers=false,flap=feathers,swept=false,thin=false}={}){
 for(const side of [-1,1]){
  const z=side*.16,m=flap?(side<0?'walkL':'walkR'):'',pivot=[x,y,z];
  // Six points taper to a rounded-looking tip; feathered houses read as wings, not cards.
  const outline=swept?[-.48,0,.43,0,.31,side*span*.44,-.27,side*span,-.72,side*span*.91,-.90,side*span*.62]:[-.43,0,.42,0,.30,side*span*.48,.04,side*span,-.26,side*span*1.04,-.47,side*span*.62];
  s.poly([x,y,z],outline,thin?.035:.08,c,m,pivot,[Math.PI/2,0,0]);
  s.parts.at(-1).amplitude=flap?.22:0;
  // A slim rounded leading edge catches the light and keeps every house's aircraft
  // from reading as a flat card. It is cheaper than a second high-detail wing mesh.
  const edge=s.taper([x+.30,y+.025,z],[x-.22,y+.025,z+side*span*.82],.026,c,m,pivot);edge.amplitude=flap?.22:0;
  if(feathers)for(let i=0;i<3;i++){
   const f=s.box([x-.35-i*.10,y-.02,z+side*(span*.58+i*.14)],[.50-i*.08,.06,.11],i%2?S:c,m,pivot);f.amplitude=flap?.22:0;
  }
 }
}
function tail(s,x,y,c,span=.43){s.box([x,y,0],[.30,.05,span*2],c);s.poly([x,y,0],[-.20,0,.20,0,-.10,.40],.045,c);}
function rotor(s,x,y,z,c=IRON,r=.55){s.rod([x,y-.22,z],[x,y,z],.055,DARK);s.box([x,y,z],[r*2,.035,.10],c,'spin',[x,y,z]);s.box([x,y,z],[.10,.035,r*2],c,'spin',[x,y,z]);}
function bird(s,{coat='#8b7252',span=.95,armor=false}={}){
 s.orb([0,.71,0],[1.06,.48,.43],coat);s.rod([.37,.75,0],[.59,1.02,0],.20,coat);s.orb([.68,1.04,0],[.35,.30,.32],coat);s.box([.90,1.02,0],[.25,.08,.15],GOLD);
 for(const z of [-.12,.12]){s.orb([.77,1.11,z],[.075,.065,.050],DARK);s.taper([-.02,.57,z],[.09,.32,z],.045,GOLD);s.box([.19,.30,z],[.24,.045,.10],GOLD);}
 wing(s,-.06,.85,span,coat,{feathers:true,swept:true});tail(s,-.64,.68,coat,.27);
 if(armor){s.box([.03,.97,0],[.43,.13,.49],P);s.orb([.68,1.10,0],[.38,.15,.36],IRON);}
}
function rider(s,{x=0,y=1.05,c=P,helmet=false}={}){
 s.capsule([x,y,0],[.22,.33,.24],c);s.orb([x+.04,y+.26,0],[.22,.23,.21],helmet?IRON:'#caa381');
 for(const z of [-.20,.20])s.taper([x,y-.07,z*.6],[x+.12,y-.28,z],.052,DARK);
 s.taper([x+.13,y+.10,.19],[x+.31,y+.67,.19],.034,WOOD);s.poly([x+.31,y+.67,.19],[-.06,0,.06,0,0,.19],.05,GOLD);
}
function gondola(s,c=WOOD,length=1.4){s.poly([0,.43,0],[-length/2,0,length/2,0,length*.37,-.27,-length*.40,-.27],.44,c);for(const z of [-.24,.24])s.box([0,.49,z],[length,.07,.05],GOLD);}
function balloon(s,{c=IVORY,length=1.65,beam=.75,y=1.35}={}){
 s.orb([-.08,y,0],[length,.72,beam],c,'bob');for(const z of [-beam*.30,beam*.30])s.rod([-.12,.46,z],[-.12,y-.27,z],.025,WOOD);
 tail(s,-length*.48,y,c,.40);
}
function cockpit(s,x,y,c='#91bac6'){s.orb([x,y,0],[.47,.31,.34],c);s.capsule([x-.12,y-.02,0],[.31,.22,.24],c);s.box([x,y+.12,0],[.05,.04,.35],IRON);}
function engine(s,x,y,z,c=IRON){s.part('cylinder',[x,y,z],[.29,.29,.55],c,'',[x,y,z],[0,Math.PI/2,0]);s.capsule([x+.12,y,z],[.43,.24,.24],c);s.part('cylinder',[x-.30,y,z],[.15,.15,.035],GOLD,'',[x-.30,y,z],[0,Math.PI/2,0]);}
function bomb(s,x,y,z,c=DARK){s.orb([x,y,z],[.35,.17,.17],c);s.box([x-.14,y,z],[.12,.035,.28],IRON);}
function cross(s,x,y,z){s.box([x,y,z],[.05,.26,.025],RED);s.box([x,y+.04,z],[.22,.05,.03],RED);}
function star(s,x,y,z){const pts=[];for(let i=0;i<10;i++){const a=i*Math.PI/5,r=i%2?.07:.16;pts.push(Math.cos(a)*r,Math.sin(a)*r);}s.poly([x,y,z],pts,.025,RED);}

function kublai(s,role){
 if(role!=='striker'){bird(s,{coat:role==='fighter'?'#645445':'#ad9b76',span:role==='fighter'?1.15:.91,armor:role==='fighter'});if(role==='fighter')rider(s,{helmet:true});else{s.box([.10,.85,0],[.32,.06,.46],P);s.orb([.70,1.18,0],[.27,.10,.27],P);}return;}
 gondola(s);balloon(s,{c:'#b6a27b',length:2.30,beam:.87,y:1.50});s.part('cylinder',[-.32,.64,0],[.52,.52,.34],IVORY,'',[-.32,.64,0],[Math.PI/2,0,0]);s.orb([-.32,.86,0],[.62,.26,.57],P);
 for(const z of [-.51,.51]){s.part('cylinder',[.29,.47,z],[.35,.35,.16],WOOD);s.part('cylinder',[.29,.47,z*1.17],[.28,.28,.025],IVORY);rotor(s,-.43,.94,z,WOOD,.39);bomb(s,.53,.21,z);}
}
function arjuna(s,role){
 if(role==='scout'){bird(s,{coat:'#c59d4e',span:1.02});s.orb([.68,1.10,0],[.38,.18,.36],P);s.box([.10,.98,0],[.30,.06,.45],GOLD);return;}
 s.poly([0,.56,0],[-.74,0,.77,0,.53,.25,-.60,.25],.61,GOLD);wing(s,-.12,.69,role==='striker'?1.27:1.0,P,{feathers:true,flap:false});tail(s,-.86,.63,GOLD);
 for(const z of [-.28,.28])s.orb([.48,.49,z],[.33,.24,.08],GOLD);
 s.box([-.13,.93,0],[.54,.37,.49],P);s.box([-.13,1.15,0],[.69,.08,.64],GOLD);s.poly([-.13,1.20,0],[-.38,0,.38,0,0,.37],.52,GOLD);
 if(role==='fighter'){rider(s,{x:.41,y:.91,c:P});s.rod([.62,.84,-.28],[.62,1.23,-.28],.04,GOLD);}
 else{for(const z of [-.30,.30]){s.rod([-.46,.86,z],[-.46,1.57,z],.04,GOLD);s.orb([.46,.37,z],[.22,.35,.22],LIGHT);}s.box([-.46,1.59,0],[.75,.09,.74],GOLD);s.orb([-.46,1.74,0],[.49,.25,.48],P);}
}
function templar(s,role){
 if(role!=='striker'){bird(s,{coat:IVORY,span:role==='fighter'?1.16:.88,armor:role==='fighter'});s.orb([-.35,.69,0],[.75,.42,.48],'#a48d62');for(const z of [-.25,.25])cross(s,.02,.92,z);if(role==='fighter')rider(s,{helmet:true,c:IVORY});return;}
 gondola(s,'#6b4935',1.72);for(const z of [-.53,.53]){balloon(s,{c:IVORY,length:2.12,beam:.40,y:1.55});const start=s.parts.length-5;for(const p of s.parts.slice(start)){p.at[2]+=z;p.pivot[2]+=z;}}
 s.box([-.12,.91,0],[.64,.67,.49],IVORY);s.poly([-.12,1.25,0],[-.47,0,.47,0,0,.43],.63,RED);for(const z of [-.27,.27])cross(s,-.12,.97,z);s.rod([.51,.58,-.39],[.51,.58,.39],.07,WOOD);s.box([.70,.62,0],[.67,.05,.06],GOLD);rotor(s,-.65,.73,0,IRON,.54);
}
function atlantis(s,role){
 if(role==='striker'){gondola(s,WOOD,1.70);balloon(s,{c:'#bba36a',length:2.45,beam:.94,y:1.49});for(const z of [-.65,.65]){engine(s,.18,.56,z,GOLD);rotor(s,.18,.93,z,WOOD,.45);}s.orb([.66,.58,0],[.46,.46,.46],'#9cccd3');s.box([-.42,.67,0],[.37,.28,.31],P);for(const z of [-.27,.27])bomb(s,.18,.20,z,GOLD);return;}
 s.orb([0,.68,0],[1.28,.32,.35],WOOD);wing(s,-.13,.76,role==='fighter'?1.22:1.00,IVORY,{thin:true,swept:role==='scout'});tail(s,-.79,.70,IVORY);cockpit(s,.22,.84);
 for(const z of [-.53,.53]){s.rod([-.26,.71,z],[-.26,.94,z],.05,GOLD);rotor(s,-.26,.96,z,WOOD,.43);}
 s.orb([.70,.66,0],[.18,.23,.23],GOLD);if(role==='fighter')for(const z of [-.20,.20])s.box([.55,.64,z],[.71,.06,.06],GOLD,'recoil');else s.part('cylinder',[.55,.51,0],[.23,.23,.10],GOLD,'',[.55,.51,0],[Math.PI/2,0,0]);
}
function fleet(s,role){
 if(role==='scout'){s.orb([0,1.05,0],[.91,1.16,.84],'#d9a16c','bob');for(const z of [-.28,.28]){s.rod([0,.35,z],[0,1.58,z],.025,WOOD);s.box([0,1.06,z],[.72,.035,.035],WOOD);}s.box([0,.37,0],[.37,.24,.37],P);s.orb([0,.56,0],[.18,.27,.18],'#fff1ad');s.rod([0,.23,0],[0,-.04,0],.025,GOLD);return;}
 if(role==='fighter'){s.poly([0,.71,0],[-.78,0,.66,0,.42,.18,-.60,.18],.30,P);wing(s,-.05,.85,1.25,'#d5b478',{swept:true,thin:true});tail(s,-.88,.75,P);for(const z of [-.52,.52]){s.rod([-.35,.87,z],[.05,.87,z],.035,WOOD);for(const dz of [-.06,.06])s.rod([.14,.62,z+dz],[.81,.62,z+dz],.05,IRON);}s.rod([-.50,.73,0],[-.80,.35,0],.025,GOLD);return;}
 gondola(s,'#8c4d3b',1.95);for(const z of [-.51,.51]){s.orb([-.12,1.32,z],[2.17,.56,.44],'#d5b478','bob');for(const x of [-.50,.42])s.rod([x,.46,z*.55],[x,1.12,z],.03,WOOD);}s.box([-.63,.73,0],[.49,.28,.43],P);s.box([-.63,.90,0],[.62,.07,.53],DARK);
 for(const z of [-.31,.31]){rotor(s,.54,.84,z,WOOD,.42);s.box([.43,.41,z],[.54,.18,.18],RED);for(const x of [.20,.36,.52])s.box([x,.36,z],[.41,.045,.045],IRON);}
}
function manifesto(s,role){
 const span=role==='striker'?1.43:role==='fighter'?1.16:.92;
 s.orb([0,.72,0],[role==='striker'?2.12:1.70,.34,.37],'#697850');wing(s,-.06,.74,span,'#697850',{swept:role==='fighter',thin:true});tail(s,-1,.75,IRON,.48);cockpit(s,.40,.88);
 for(const z of [-span*.65,span*.65]){engine(s,.02,.64,z,IRON);star(s,-.10,.79,z);}
 star(s,.15,.77,.20);s.poly([-.89,.78,0],[-.21,0,.17,0,-.14,.47],.06,P);
 if(role==='scout'){wing(s,-.08,1.10,.92,IVORY,{thin:true});for(const z of [-.64,.64])s.rod([-.10,.75,z],[-.10,1.10,z],.03,WOOD);s.orb([.14,.49,0],[.20,.17,.22],DARK);}
 if(role==='fighter')for(const z of [-.32,.32])s.box([.50,.69,z],[.77,.06,.06],DARK,'recoil');
 if(role==='striker')for(const z of [-.43,.43])for(const x of [-.39,.20])bomb(s,x,.41,z);
}
function nullSociety(s,role){
 if(role==='scout'){s.orb([0,.81,0],[.49,.49,.49],DARK,'bob');s.orb([.23,.81,0],[.06,.28,.28],LIGHT,'bob');for(const z of [-.54,.54]){s.rod([-.07,.80,0],[-.30,.83,z],.06,IRON);s.poly([-.3,.83,z],[-.26,0,.26,0,-.10,.29],.06,P,'spin',[-.30,.83,z],[Math.PI/2,0,0]);}return;}
 s.poly([0,.67,0],[-1,0,1.12,.05,-.27,.33,-.86,.22],.34,DARK);wing(s,-.13,.76,role==='striker'?1.42:1.11,DARK,{swept:true});tail(s,-.99,.79,P,.51);
 for(const z of [-.48,.48]){s.box([.50,.71,z],[1.05,.09,.10],P);s.box([1.04,.71,z],[.045,.045,.055],LIGHT);s.box([-.65,.84,z],[.44,.035,.06],LIGHT);}
 s.box([.43,.88,0],[.30,.05,.26],LIGHT);
 if(role==='striker')for(const z of [-.81,.81]){s.box([-.39,.59,z],[.85,.22,.25],DARK);s.box([-.39,.46,z],[.51,.04,.12],LIGHT);s.poly([-.61,.74,z],[-.24,0,.24,0,-.28,.49],.06,P);}
}
function nations(s,role){
 if(role==='scout'){gondola(s,WOOD,.60);balloon(s,{c:IVORY,length:1.19,beam:.96,y:1.20});s.rod([.29,.41,0],[.65,.44,0],.08,GOLD);s.orb([.69,.44,0],[.06,.21,.21],DARK);return;}
 if(role==='striker'){gondola(s,'#694334',1.73);balloon(s,{c:IVORY,length:2.50,beam:.93,y:1.49});for(const z of [-.66,.66])rotor(s,-.28,.75,z,WOOD,.48);for(const x of [-.46,.13,.61])s.box([x,.57,0],[.34,.26,.37],WOOD);for(const z of [-.22,.22])s.box([.06,.68,z],[1.57,.035,.025],GOLD);return;}
 // A tapered brass-and-canvas fuselage replaces the former rectangular plank.
 s.capsule([-.03,.69,0],[1.47,.29,.36],WOOD);s.taper([-.63,.69,0],[.67,.71,0],.15,WOOD);wing(s,-.10,.80,1.18,IVORY,{feathers:false,thin:true});tail(s,-.77,.73,IVORY,.43);cockpit(s,.24,.87);for(const z of [-.68,.68]){rotor(s,-.27,1.02,z,GOLD,.37);s.rod([-.27,.69,z],[-.27,.99,z],.035,GOLD);}s.taper([.35,.70,0],[.98,.70,0],.035,GOLD,'recoil');
}
function emu(s,role){
 if(role==='striker'){s.capsule([-.08,.69,0],[1.72,.43,.48],'#8c8a5b');s.taper([-.69,.69,0],[.72,.76,0],.20,'#8c8a5b');wing(s,-.09,.81,1.32,'#c7b57e',{thin:true});wing(s,-.09,1.19,1.32,IVORY,{thin:true});tail(s,-.94,.76,WOOD,.43);for(const z of [-1.02,1.02])s.rod([-.10,.82,z],[-.10,1.18,z],.035,WOOD);s.orb([.60,.81,0],[.28,.23,.26],DARK);rotor(s,.56,1.08,0,WOOD,.41);for(const z of [-.44,.44])s.capsule([-.26,.39,z],[.65,.28,.26],'#b69c5c');return;}
 s.orb([-.13,.67,0],[.77,.60,.55],'#8a7d61');s.rod([.14,.85,0],[.29,1.21,0],.15,'#8a7d61');s.orb([.36,1.28,0],[.30,.27,.25],DARK);s.box([.57,1.25,0],[.22,.06,.13],GOLD);s.box([.47,1.36,0],[.10,.07,.30],IRON);
 for(const z of [-.18,.18]){s.rod([0,.45,z],[.15,.25,z],.055,WOOD);s.box([.24,.23,z],[.31,.045,.13],DARK);}
 if(role==='scout'){wing(s,-.14,.84,1.03,IVORY,{feathers:true});s.box([-.33,.87,0],[.30,.29,.38],P);}
 else{for(const z of [-.43,.43]){s.box([-.13,.70,z],[.36,.25,.20],IRON);rotor(s,-.13,1.11,z,WOOD,.53);}s.box([.45,.66,.38],[1,.07,.09],DARK,'recoil');s.part('cylinder',[.26,.78,.38],[.28,.28,.04],DARK,'recoil',[.26,.78,.38],[Math.PI/2,0,0]);}
}
function garden(s,role){
 const petals=role==='striker'?7:role==='fighter'?4:6,radius=role==='striker'?1.05:.70;
 s.orb([0,.74,0],[.54,.44,.49],role==='scout'?IVORY:'#91aa75','bob');s.orb([.26,.76,0],[.055,.25,.25],DARK,'bob');s.orb([.29,.77,0],[.03,.13,.13],P,'bob');
 for(let i=0;i<petals;i++){const a=i*Math.PI*2/petals,x=Math.cos(a)*radius*.65,z=Math.sin(a)*radius*.65,c=role==='striker'?'#d3a7a4':role==='fighter'?IVORY:'#d8d5b6';s.orb([x,.85,z],[.60,.10,.39],c,'sway',[0,.79,0]);s.rod([0,.70,0],[x,.84,z],.035,WOOD);}
 if(role==='scout'){for(const z of [-.37,.37])s.rod([0,.87,z*.5],[0,1.45,z],.025,IVORY);s.orb([0,1.50,0],[.53,.30,.51],IVORY,'spin',[0,1.50,0]);}
 if(role==='fighter'){wing(s,-.08,.88,1.20,'#c1d09d',{swept:true,thin:true});for(const z of [-.32,.32])s.rod([.18,.64,z],[.92,.74,z],.065,WOOD,'strike',[.18,.64,z]);}
 if(role==='striker'){gondola(s,'#718966',1.54);for(const z of [-.38,.38]){s.rod([-.34,.57,z],[-.34,1.19,z],.05,WOOD);s.orb([-.34,1.21,z],[.34,.31,.34],'#d59a9e');bomb(s,.46,.23,z,'#799267');}}
}
const BUILDERS=[kublai,arjuna,templar,atlantis,fleet,manifesto,nullSociety,nations,emu,garden];
// Raised rigging gives each house a distinct planform from the top-down RTS view.
// It supplements, rather than replaces, the role-specific wings and payloads above.
function housePlanform(s,seat,role){
 const y=role==='striker'?.91:.82;
 if(seat===0){for(const z of [-.34,.34])s.taper([-.42,y,z],[.12,y+.36,z],.025,'#493e34','sway',[-.42,y,z]);}
 else if(seat===1){for(const z of [-.47,.47])s.poly([-.30,y,z],[-.31,0,.31,0,0,.26],.035,GOLD);}
 else if(seat===2){s.poly([-.42,y+.13,0],[-.42,0,.42,0,0,.38],.38,RED);cross(s,-.42,y+.15,.20);}
 else if(seat===3){for(const z of [-.36,.36])s.part('cylinder',[-.25,y,z],[.21,.21,.04],GOLD,'spin',[-.25,y,z],[Math.PI/2,0,0]);}
 else if(seat===4){for(const z of [-.44,.44])s.box([-.20,y+.12,z],[.63,.035,.05],'#d5b478');}
 else if(seat===5){for(const z of [-.43,.43])s.box([-.36,y,z],[.47,.13,.18],IRON);}
 else if(seat===6){for(const z of [-.54,.54])s.rod([-.29,y,0],[-.54,y+.23,z],.035,LIGHT);}
 else if(seat===7){s.part('cylinder',[-.33,y+.13,0],[.30,.30,.045],GOLD,'',[-.33,y+.13,0],[Math.PI/2,0,0]);}
 else if(seat===8){for(const z of [-.48,0,.48])s.poly([-.28,y,z],[-.17,0,.17,0,0,.28],.025,'#c7b57e');}
 else{for(const z of [-.37,.37])s.orb([-.18,y,z],[.34,.07,.26],'#d3a7a4','sway',[-.18,y,0]);}
}
export function airSculpt(seat,role='fighter'){
 if(!BUILDERS[seat]||!AIR_ROLES.includes(role))return null;
 const s=new Sculpt();BUILDERS[seat](s,role);housePlanform(s,seat,role);return {parts:s.parts};
}
