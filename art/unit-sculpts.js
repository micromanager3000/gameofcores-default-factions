import { Sculpt } from './sculpt.js';
import { airSculpt } from './air-sculpts.js';
import { navalSculpt } from './naval-sculpts.js';
export { navalSculpt } from './naval-sculpts.js';
// House-authored RTS miniatures. Generated once per look/role, retained by GPUUnits.
// +x is forward, +y is up, z separates the left/right limbs. No textures or per-frame JS rigs.
const P='primary',S='secondary',A='accent',IRON='#788b95',DARK='#26333a',WOOD='#785239',GOLD='#dab866',IVORY='#e6dcc2',LIGHT='#a3fff0';
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

function human(s,{coat=P,skin='#d6a17e',legs=DARK,hat='cap',trim=S,armor=false,robe=false,lean=0}={}){
 const start=s.parts.length;
 for(const z of [-.16,.16]){
  const motion=z<0?'strideL':'strideR',pivot=[0,.65,z];
  // Separate thigh, knee and calf give every walking unit a readable gait from the
  // strategy camera without adding a skinned mesh or a per-frame rig.
  s.capsule([-.04,.51,z],[.20,.34,.20],legs,motion,pivot);
  const knee=z<0?'kneeL':'kneeR';s.orb([.02,.33,z],[.17,.14,.17],legs,knee,pivot);
  s.capsule([.06,.20,z],[.16,.29,.17],legs,knee,pivot);
  s.poly([.15,.075,z],[-.18,-.06,.20,-.06,.30,.045,.08,.10,-.22,.06],.23,DARK,knee,pivot);
 }
 s.capsule([lean,.92,0],[.46,.66,.43],coat,'',[lean,.92,0]);
 s.poly([lean+.12,.88,0],[-.12,-.20,.23,-.12,.28,.14,.14,.25,-.18,.18],.45,coat);
 if(robe)s.poly([0,.19,0],[-.33,0,.30,0,.22,.64,-.22,.64],.51,coat);
 s.box([lean,.80,0],[.47,.052,.51],trim);
 if(armor){s.poly([lean+.03,.89,0],[-.16,0,.16,0,.22,.32,.14,.39,-.14,.39,-.22,.32],.49,coat);for(const z of [-.30,.30]){s.orb([lean,1.15,z],[.29,.22,.26],IRON);s.box([lean+.05,1.16,z],[.20,.07,.12],coat);}}
 s.orb([lean+.03,1.40,0],[.30,.34,.29],skin);
 // A nose and a dark brow make the facing readable without a face texture.
 s.box([lean+.193,1.40,0],[.075,.09,.11],skin);
 s.box([lean+.174,1.46,0],[.035,.035,.24],DARK);
 for(const z of [-.31,.31]){
  const m=z<0?'strideR':'strideL',pivot=[lean,1.15,z];
  s.taper([lean,1.15,z],[.17,.84,z],.135,coat,m,pivot);
  s.orb([.18,.84,z],[.15,.16,.16],skin,m,pivot);
 }
 headwear(s,hat,lean,trim);
 return start;
}
function headwear(s,hat,x=0,trim=S){
 if(hat==='none')return;
 if(hat==='hood'){s.orb([x,1.46,-.04],[.39,.44,.38],P);s.box([x+.20,1.40,0],[.04,.20,.23],DARK);}
 else if(hat==='greathelm'){s.box([x+.01,1.46,0],[.38,.44,.36],IRON);s.box([x+.208,1.51,0],[.035,.045,.30],DARK);s.box([x+.215,1.42,0],[.035,.18,.035],GOLD);}
 else if(hat==='conical'){s.poly([x,1.53,0],[-.24,0,.24,0,.04,.31,-.05,.31],.39,IRON);s.box([x,1.52,0],[.46,.065,.43],trim);}
 else if(hat==='turban'){s.orb([x,1.58,0],[.43,.25,.41],IVORY);s.box([x+.01,1.61,0],[.39,.055,.43],trim);s.poly([x-.12,1.62,0],[-.10,0,.12,0,0,.22],.10,GOLD);}
 else if(hat==='tricorn'){s.poly([x,1.57,0],[-.32,0,.32,0,.10,.20,-.20,.20],.52,DARK);s.box([x,1.56,0],[.66,.045,.57],trim);}
 else if(hat==='fur'){s.orb([x,1.56,0],[.43,.20,.43],'#493e34');s.poly([x,1.62,0],[-.18,0,.18,0,0,.23],.29,P);}
 else if(hat==='straw'){s.poly([x,1.52,0],[-.40,0,.40,0,0,.28],.64,'#c5a86c');}
 else if(hat==='crown'){s.box([x,1.58,0],[.38,.12,.35],GOLD);for(const z of [-.15,.15])s.poly([x,1.64,z],[-.2,0,.2,0,.17,.20,0,.08,-.17,.20],.05,GOLD);}
 else {s.orb([x,1.56,0],[.39,.20,.37],hat==='helmet'?IRON:hat==='beret'?P:DARK);s.box([x+.21,1.55,0],[.19,.045,.32],hat==='helmet'?IRON:DARK);}
}
function tool(s,kind,{x=.25,y=.9,z=.37}={}){
 const first=s.parts.length;
 if(['bow','crossbow','rifle','musket'].includes(kind))s.muzzle=[x+(kind==='bow'?.44:kind==='crossbow'?.45:.79),y+(kind==='rifle'||kind==='musket'?.035:0),z];
 if(kind==='bow'){
  s.poly([x+.25,y,z],[0,-.43,.14,-.27,.19,0,.14,.28,0,.44,.06,.20,.08,0,.04,-.20],.065,WOOD);
  s.rod([x+.25,y-.43,z],[x+.25,y+.44,z],.022,IVORY);
  s.box([-.30,1.13,-z],[.16,.53,.18],'#594332');
 }
 else if(kind==='crossbow'){s.box([x+.1,y,z],[.70,.09,.10],WOOD);s.rod([x+.35,y,z-.32],[x+.35,y,z+.32],.07,IRON);}
 else if(kind==='rifle'||kind==='musket'){s.box([x,y,z],[.43,.13,.12],WOOD);s.box([x+.44,y+.035,z],[.70,.055,.06],IRON);if(kind==='rifle')s.box([x+.17,y+.13,z],[.26,.055,.07],DARK);for(const p of s.parts.slice(first))p.motion='recoil';}
 else if(kind==='spear'||kind==='staff'){s.rod([x,y-.62,z],[x+.24,y+.90,z],.055,WOOD);s.poly([x+.24,y+.90,z],[-.07,-.02,.07,-.02,0,.28],.065,IRON);}
 else if(kind==='sword'){s.box([x+.32,y+.20,z],[.065,.70,.045],IRON,'strike',[x,y,z],[0,0,-.4]);s.box([x+.19,y-.10,z],[.29,.055,.065],GOLD,'strike',[x,y,z]);}
 else if(kind==='pick'){s.rod([x,y-.45,z],[x+.27,y+.26,z],.055,WOOD,'strike',[x,y,z]);s.rod([x+.01,y+.20,z],[x+.51,y+.35,z],.075,IRON,'strike',[x,y,z]);}
 else if(kind==='hammer'){s.rod([x,y-.22,z],[x+.16,y+.40,z],.07,WOOD,'strike',[x,y,z]);s.box([x+.16,y+.38,z],[.32,.20,.23],IRON,'strike',[x,y,z]);}
 else if(kind==='mace'){s.rod([x,y-.20,z],[x+.16,y+.46,z],.075,WOOD,'strike',[x,y,z]);s.orb([x+.16,y+.48,z],[.31,.30,.31],GOLD,'strike',[x,y,z]);}
 else if(kind==='book'){s.box([x+.10,y,z],[.30,.08,.39],WOOD);s.box([x+.10,y+.04,z],[.25,.045,.33],IVORY);}
 else if(kind==='lantern'){s.rod([x,y,z],[x,y+.5,z],.045,DARK);s.box([x,y-.12,z],[.22,.30,.22],GOLD,'sway',[x,y+.3,z]);s.box([x+.12,y-.10,z],[.025,.18,.13],'#fff1b4','sway',[x,y+.3,z]);}
}
function shield(s,c=P,cross=false){
 s.poly([.10,.92,-.46],[-.27,.25,.27,.25,.23,-.13,0,-.35,-.23,-.13],.10,c);
 if(cross){s.box([.11,.94,-.52],[.065,.40,.02],'#ae303d');s.box([.11,1,-.53],[.30,.065,.02],'#ae303d');}
}
function banner(s,c=P,x=-.20,height=2.1){s.rod([x,.65,-.35],[x,height,-.35],.035,WOOD);s.box([x+.27,height-.19,-.35],[.55,.34,.04],c,'sway',[x,height,-.35]);}
function animal(s,{coat='#926044',camel=false,elephant=false}={}){
 const h=elephant?.9:.65,L=elephant?1.45:1.25;
 s.orb([-.05,h,0],[L,elephant?.90:.55,elephant?.80:.52],coat);
 for(const x of [-.45,.43])for(const z of [-.23,.23]){
  const motion=(x*z>0)?'strideL':'strideR',pivot=[x,h-.1,z];
  s.capsule([x,h/2-.05,z],[elephant?.22:.14,h-.10,elephant?.24:.15],coat,motion,pivot);
  s.orb([x+.03,.14,z],[elephant?.22:.16,.16,elephant?.23:.17],coat,motion,pivot);
  s.poly([x+.13,.055,z],[-.15,-.05,.22,-.05,.30,.05,.06,.10,-.19,.06],elephant?.27:.15,elephant?coat:DARK,motion,pivot);
 }
 s.taper([.45,h+.10,0],[.65,h+(elephant?.16:.65),0],elephant?.5:.24,coat);
 s.orb([.80,h+(elephant?.12:.61),0],[elephant?.64:.46,elephant?.62:.28,elephant?.65:.30],coat);
 s.rod([-.62,h+.1,0],[-.82,h-.25,0],.08,DARK,'sway',[-.62,h+.1,0]);
 if(camel){s.orb([-.24,h+.34,0],[.5,.45,.4],coat);s.orb([.19,h+.32,0],[.43,.42,.4],coat);}
 if(elephant){s.orb([.65,h+.15,.36],[.12,.59,.35],coat);s.orb([.65,h+.15,-.36],[.12,.59,.35],coat);s.taper([1.02,h+.1,0],[1.12,.18,0],.19,coat);for(const z of [-.23,.23])s.taper([1,h+.35,z],[1.35,h+.28,z],.07,IVORY);}
 else {s.box([.79,h+.66,0],[.33,.06,.36],WOOD);s.box([.42,h+.28,0],[.55,.065,.57],P);}
}
function mounted(s,{coat,camel=false,rider={},weapon='spear',standard=false}={}){
 animal(s,{coat,camel});
 const start=human(s,rider);s.transform(start,.74,[-.17,.72,0]);
 tool(s,weapon,{x:.05,y:1.30,z:.28});if(standard)banner(s,P,-.34,2.20);
}
function cart(s,{tank=false,truck=false,cannon=false,rockets=false,culture=0}={}){
 const c=tank?'#60704c':culture===3?'#947855':WOOD;
 if(tank){
  for(const z of [-.45,.45]){s.box([0,.28,z],[1.65,.43,.28],DARK);for(const x of [-.60,-.20,.20,.60])s.wheel([x,.27,z+(z<0?-.15:.15)],.15,IRON);}
  s.poly([0,.42,0],[-.83,0,.83,0,.57,.38,-.64,.38],.85,c);s.poly([-.1,.79,0],[-.39,0,.44,0,.27,.28,-.25,.28],.64,c);s.box([.67,.99,0],[1.05,.09,.11],IRON);s.box([-.12,1.10,0],[.30,.035,.29],DARK);s.box([.04,.89,-.33],[.16,.12,.02],P);return;
 }
 s.box([0,.30,0],[1.48,.16,.80],c);
 for(const x of [-.48,.48])for(const z of [-.47,.47])s.wheel([x,.22,z],.22,DARK);
 if(truck||rockets){s.box([.42,.67,0],[.56,.56,.65],culture===8?'#8c8a5b':'#68754d');s.box([.72,.74,0],[.025,.22,.55],'#98c7cf');s.box([-.30,.48,0],[.81,.17,.72],c);}
 if(rockets){for(const z of [-.23,0,.23]){s.rod([-.73,.72,z],[.14,1.05,z],.10,IRON);s.rod([-.75,.86,z],[.12,1.19,z],.10,IRON);}s.box([-.39,.70,0],[.12,.43,.72],DARK);}
 if(truck&&culture===8){s.rod([-.3,.5,0],[-.3,1.02,0],.07,IRON);s.box([.01,1.04,0],[.95,.075,.09],DARK);s.part('cylinder',[-.12,1.12,0],[.27,.27,.05],DARK,'recoil',[-.12,1.12,0],[Math.PI/2,0,0]);}
 if(cannon){s.part('cylinder',[.03,.72,0],[.39,.39,1.10],DARK,'strike',[0,.63,0],[0,Math.PI/2,0],.08);s.box([-.30,.55,0],[.37,.16,.63],GOLD);}
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
  mounted(s,{coat:body==='keshig'?'#513c30':body==='camel'?'#b39a65':'#986646',camel:body==='camel',rider:{coat:body==='keshig'?IRON:P,hat:body==='keshig'?'conical':'fur',armor:body==='keshig'},weapon:body==='archer'?'bow':body==='horseman'?'sword':body==='camel'?'staff':'spear',standard:body==='keshig'});
  if(body==='camel'){for(const z of [-.40,.40])s.box([-.25,.80,z],[.45,.42,.22],'#a27b50');}return;
 }
 if(body==='trebuchet'){
  cart(s);for(const z of [-.29,.29]){s.rod([-.51,.38,z],[0,1.39,z],.11,WOOD);s.rod([.55,.38,z],[0,1.39,z],.11,WOOD);}
  s.rod([-.40,1.03,0],[.65,1.85,0],.11,WOOD,'strike',[0,1.39,0]);s.box([-.40,.96,0],[.36,.43,.37],'#8b8b7d','strike',[0,1.39,0]);s.rod([.65,1.85,0],[.94,1.25,0],.025,IVORY,'strike',[0,1.39,0]);banner(s,P,-.54,1.3);return;
 }
 human(s,{coat:body==='shaman'?'#354b69':body==='artisan'?'#8b7860':P,hat:'fur',robe:body==='shaman'});
 if(body==='artisan')tool(s,'pick');
 if(body==='falconer'){s.rod([.07,1.14,.31],[.52,1.23,.31],.14,WOOD);s.orb([.52,1.36,.31],[.25,.30,.20],'#654d37');s.poly([.48,1.40,.31],[-.19,0,.22,0,.11,.13],.15,IVORY);s.box([.68,1.46,.31],[.1,.05,.07],GOLD);}
 if(body==='shaman'){s.part('cylinder',[.37,.98,.44],[.43,.43,.15],IVORY);s.part('cylinder',[.37,.98,.42],[.48,.48,.07],WOOD);tool(s,'staff',{x:-.25,y:.9,z:-.37});}
}
function arjuna(s,body){
 if(body==='chariot'){
  animal(s,{coat:'#e9e1c9'});const start=s.parts.length;animal(s,{coat:'#d5cdbb'});s.transform(start,.82,[.15,0,-.47]);
  const a=s.parts.length;human(s,{coat:P,hat:'crown',trim:GOLD,skin:'#b77f59'});s.transform(a,.67,[-.95,.54,0]);
  s.box([-.94,.43,0],[.78,.17,.86],GOLD);for(const z of [-.48,.48]){s.wheel([-.98,.28,z],.30,GOLD);s.poly([-1.15,.51,z],[-.18,0,.52,0,.43,.45,-.18,.34],.07,P);}
  s.rod([-.80,.48,0],[.50,.60,0],.055,GOLD);tool(s,'bow',{x:-.6,y:1.40,z:.35});banner(s,S,-1.23,2.10);return;
 }
 if(body==='elephant'){
  animal(s,{coat:'#7e898d',elephant:true});s.box([-.05,1.40,0],[.98,.08,.94],P);s.box([-.10,1.62,0],[.65,.35,.66],GOLD);s.box([-.10,1.80,0],[.56,.04,.56],P);const a=s.parts.length;human(s,{hat:'turban',coat:P,trim:GOLD,skin:'#b77f59'});s.transform(a,.42,[-.08,1.64,0]);return;
 }
 human(s,{coat:body==='sage'?'#dfb76d':body==='healer'?'#dfd4bb':body==='servant'?'#b5a586':P,skin:body==='giant'?'#625377':'#b77f59',hat:body==='servant'?'turban':body==='sage'?'none':'crown',trim:GOLD,robe:['sage','healer','servant'].includes(body)});
 if(body==='archer')tool(s,'bow');
 if(body==='mace')tool(s,'mace');
 if(body==='sage'){tool(s,'staff');s.orb([-.1,1.62,0],[.31,.11,.32],DARK);}
 if(body==='healer'){s.orb([.42,.85,.37],[.30,.36,.30],'#a8683f');s.box([.42,1.05,.37],[.15,.05,.15],GOLD);}
 if(body==='servant')tool(s,'pick');
 if(body==='giant'){s.transform(0,1.22,[0,0,0]);s.orb([.57,1.51,.36],[.65,.60,.55],'#777c83','strike',[.12,1.36,.3]);}
}
function templar(s,body){
 if(body==='ballista'){
  cart(s);s.box([0,.62,0],[1.17,.19,.28],WOOD);s.rod([.35,.69,-.65],[.35,.69,.65],.13,DARK);s.rod([-.55,.73,0],[.88,.73,0],.055,IRON);s.rod([.35,.69,-.65],[-.48,.73,0],.022,IVORY);s.rod([-.48,.73,0],[.35,.69,.65],.022,IVORY);s.poly([.91,.73,0],[-.13,-.06,-.13,.06,.15,0],.06,GOLD);return;
 }
 if(['marshal','turcopole'].includes(body)){mounted(s,{coat:body==='marshal'?'#c7c7ba':'#946c4c',rider:{coat:body==='marshal'?IVORY:'#9a7e5d',hat:body==='marshal'?'greathelm':'conical',armor:true},weapon:body==='marshal'?'spear':'bow',standard:body==='marshal'});s.box([-.05,.85,-.32],[.48,.34,.03],'#b7323f');return;}
 const healer=body==='hospitaller',worker=body==='acolyte',c=healer?'#27333c':body==='sergeant'?'#7f7767':IVORY;
 human(s,{coat:c,hat:worker?'hood':body==='crusader'?'greathelm':'helmet',trim:healer?IVORY:S,armor:!worker,robe:worker||healer});
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
 if(body==='automaton'){human(s,{coat:'#b89a63',skin:IRON,hat:'helmet',armor:true});s.part('cylinder',[.25,1.05,0],[.24,.24,.06],GOLD,'spin',[.25,1.05,0],[0,Math.PI/2,0]);s.box([.20,1.48,0],[.04,.05,.26],'#9fe8db');tool(s,'spear');return;}
 human(s,{coat:body==='mystery'?'#414c56':body==='pioneer'?'#957b58':'#466d7d',hat:body==='mystery'?'hood':'tricorn',trim:GOLD,robe:body==='fellow'});
 if(body==='pioneer')tool(s,'hammer');
 if(body==='gunner'){tool(s,'musket');s.box([-.22,.85,-.37],[.32,.42,.20],WOOD);}
 if(body==='fellow'){tool(s,'book');s.orb([.34,1.20,.37],[.18,.26,.18],'#b6e9e2');}
 if(body==='mystery'){tool(s,'lantern');s.box([-.20,1.05,-.36],[.23,.37,.18],DARK);}
}
function fleet(s,body){
 if(body==='junk'){vessel(s,'junk');return;}
 if(body==='qilin'){
  // The fleet brought a giraffe home: a real animal, rather than a generic fantasy unicorn.
  animal(s,{coat:'#bb9957'});s.rod([.40,.9,0],[.64,1.85,0],.20,'#bb9957');s.orb([.73,1.94,0],[.35,.28,.24],'#bb9957');s.rod([.65,2.03,-.07],[.63,2.2,-.07],.045,DARK);s.rod([.65,2.03,.07],[.63,2.2,.07],.045,DARK);for(const x of [-.4,-.1,.2])for(const z of [-.27,.27])s.box([x,.8,z],[.12,.18,.025],'#705335');s.box([-.1,.96,0],[.65,.10,.62],P);return;
 }
 if(body==='rockets'){cart(s);s.box([-.1,.77,0],[.86,.55,.72],'#a34c36');for(const z of [-.23,0,.23])for(const y of [.65,.85,1.05])s.rod([-.55,y,z],[.70,y+.25,z],.07,WOOD);s.box([.50,.79,0],[.035,.54,.68],DARK);return;}
 const worker=body==='porter';human(s,{coat:worker?'#a58b65':body==='chronicler'?'#374e65':P,skin:'#c9a078',hat:worker?'straw':body==='chronicler'?'cap':'conical',trim:'#7b3338',armor:body==='marine',robe:body==='chronicler'});
 if(worker){tool(s,'pick');s.rod([-.43,1.1,-.4],[.45,1.1,-.4],.055,WOOD);for(const x of [-.4,.4])s.box([x,.72,-.4],[.27,.35,.28],'#aa865b');}
 if(body==='marine'){tool(s,'spear');shield(s,'#9d513d');}
 if(body==='crossbow')tool(s,'crossbow');
 if(body==='signal'){tool(s,'lantern');banner(s,S,-.25,1.9);}
 if(body==='chronicler'){tool(s,'book');s.box([-.23,1,-.4],[.21,.42,.18],WOOD);}
}
function manifesto(s,body){
 if(body==='tank'){cart(s,{tank:true});return;}
 if(body==='katyusha'){cart(s,{truck:true,rockets:true});return;}
 if(body==='cavalry'){mounted(s,{coat:'#6c4c35',rider:{coat:'#756e4b',hat:'cap',trim:P},weapon:'sword',standard:true});return;}
 human(s,{coat:body==='stakhanovite'?'#637581':body==='mole'?'#514c43':'#727650',hat:body==='mole'?'hood':body==='soldier'?'helmet':'cap',trim:body==='mole'?DARK:P,robe:body==='mole'});
 if(body==='stakhanovite'){tool(s,'pick');s.box([.21,1.57,0],[.08,.08,.12],'#ffe2a1');s.box([-.3,.88,-.32],[.28,.4,.22],WOOD);}
 else if(body==='agitator'){banner(s,P,-.29,1.95);tool(s,'book');s.box([.12,1.1,.32],[.06,.20,.045],GOLD);}
 else if(body==='mole')tool(s,'book');
 else {tool(s,'rifle');s.box([-.18,.90,-.35],[.27,.34,.17],WOOD);if(body==='sniper')s.box([-.24,1.04,0],[.12,.47,.56],'#9a9d7a');}
}
function nullSociety(s,body){
 if(body==='bug'){
  s.orb([0,.4,0],[.79,.34,.49],DARK);s.orb([.28,.46,0],[.37,.25,.32],P);
  for(const x of [-.25,0,.25])for(const z of [-1,1]){const m=x*z<0?'strideL':'strideR';s.taper([x,.41,z*.18],[x-.14,.05,z*.48],.042,IRON,m,[x,.41,z*.18]);}for(const z of [-.24,.24])s.poly([-.08,.54,z],[-.32,-.05,.26,-.05,.43,.11,.08,.18,-.30,.10],.035,IRON,'sway',[-.08,.54,z]);s.taper([.37,.59,-.12],[.55,.79,-.22],.022,IRON,'sway',[.37,.59,-.12]);s.taper([.37,.59,.12],[.55,.79,.22],.022,IRON,'sway',[.37,.59,.12]);s.box([.46,.48,0],[.06,.07,.23],'#bdfcf4');return;
 }
 if(body==='bomb'){
  s.orb([0,.60,0],[.87,.87,.80],DARK,'bob');s.box([0,.70,0],[.93,.08,.08],P);for(const z of [-1,1])s.box([0,.70,z*.32],[.12,.43,.1],IRON);s.rod([0,1,0],[.2,1.35,0],.07,GOLD,'sway',[0,1,0]);s.orb([.22,1.38,0],[.14,.14,.14],'#d5fff7','bob');return;
 }
 if(body==='watch'){
  s.orb([0,.12,0],[.72,.20,.69],DARK);s.capsule([0,.67,0],[.20,.95,.20],IRON);s.part('cylinder',[0,1.30,0],[.75,.75,.19],P,'sway',[0,1.30,0]);for(const z of [-.23,.23])s.taper([0,1.30,z],[.28,1.30,z],.025,LIGHT,'spin',[0,1.30,0]);s.orb([.04,1.30,.13],[.27,.27,.12],'#d5fff7');return;
 }
 if(body==='collector'){
  cart(s);s.box([-.12,.66,0],[.80,.51,.59],DARK);s.box([-.1,.95,0],[.83,.08,.63],P);s.rod([.28,.81,0],[.70,1.25,0],.12,IRON,'strike',[.28,.81,0]);s.rod([.70,1.25,0],[1,.91,0],.10,IRON,'strike',[.28,.81,0]);return;
 }
 human(s,{coat:DARK,skin:'#a6c7c8',hat:body==='exception'?'hood':'helmet',trim:P,armor:body==='zero'});
 s.box([.26,1.03,0],[.035,.27,.26],P);s.box([.29,1.04,0],[.035,.15,.15],DARK);s.box([.19,1.46,0],[.045,.065,.26],'#c2fff2');
 if(body==='exception')tool(s,'rifle');else if(body==='pointer')tool(s,'spear');else if(body==='crash')tool(s,'sword');else shield(s,P);
}
function nations(s,body){
 if(body==='carronade'){cart(s,{cannon:true});return;}
 if(body==='merchantman'){vessel(s,'indiaman');return;}
 if(body==='dragoon'){mounted(s,{coat:'#c4c8c4',rider:{coat:'#a54a43',hat:'tricorn',trim:IVORY},weapon:'musket'});return;}
 human(s,{coat:body==='highlander'?'#b65b4d':body==='pinmaker'?'#927858':body==='smuggler'?'#4d625c':'#597382',hat:body==='highlander'?'beret':body==='pinmaker'?'cap':'tricorn',trim:IVORY,robe:body==='smuggler'});
 if(body==='highlander'){
  s.box([0,.58,0],[.49,.35,.56],'#496254');for(const z of [-.31,.31]){s.box([0,.6,z],[.5,.065,.03],'#b85e4b');s.box([.04,.6,z],[.065,.33,.035],IVORY);}tool(s,'musket');
 }else if(body==='exciseman')tool(s,'musket');
 else if(body==='pinmaker'){tool(s,'hammer');s.box([-.25,.76,.30],[.04,.40,.07],GOLD);}
 else if(body==='shopkeeper'){tool(s,'book');s.box([-.27,.80,-.35],[.28,.28,.2],WOOD);}
 else{tool(s,'sword');s.box([-.24,.83,.35],[.32,.35,.23],WOOD);}
}
function garden(s,body){
 if(body==='eye'){s.orb([0,1,0],[.65,.42,.44],IVORY,'bob');s.orb([.32,1,0],[.06,.29,.27],P,'bob');s.orb([.36,1,0],[.045,.16,.14],DARK,'bob');for(const z of [-1,1]){s.poly([-.12,.95,z*.23],[-.4,0,0,.12,.25,0,0,-.14],.08,S,'sway',[-.12,.95,z*.23]);s.poly([-.16,1.15,z*.27],[-.28,0,.12,.08,.31,.02,.03,-.14],.025,'#d6a0a8','sway',[-.12,.95,z*.23]);}return;}
 if(body==='thorn'){s.orb([0,.22,0],[.75,.43,.66],'#6a8760');for(const z of [-.23,.23])s.taper([-.24,.27,z],[.15,.95,z],.12,'#587450');s.taper([.15,.95,0],[.73,1.28,0],.17,P,'strike',[.15,.95,0]);s.poly([.73,1.28,0],[0,0,-.09,.28,.12,.10,.30,.12],.17,IVORY);for(const z of [-.31,.31])s.poly([.15,.62,z],[-.30,0,.14,.08,.34,.22,.02,.16],.028,'#91aa75','sway',[.15,.95,z]);s.orb([-.18,.71,0],[.60,.50,.52],P);return;}
 if(body==='tree'){
  for(const z of [-.22,.22]){s.rod([0,.7,z],[z*.4,.08,z],.21,'#72523b',z<0?'strideL':'strideR',[0,.7,z]);s.rod([0,1.1,z],[.45,.68,z*2],.14,WOOD,'strike',[0,1.1,z]);}
  s.poly([0,.56,0],[-.24,0,.24,0,.19,.81,-.16,.81],.47,WOOD);s.orb([-.04,1.53,0],[.98,.65,.86],P);s.orb([.32,1.45,.17],[.58,.44,.55],'#91aa75');s.box([.22,1.25,-.12],[.035,.06,.08],GOLD);s.box([.22,1.25,.12],[.035,.06,.08],GOLD);return;
 }
 human(s,{coat:body==='statue'?IVORY:'#8b9a72',skin:body==='statue'?IVORY:'#bd9d7d',hat:body==='statue'?'none':'straw',trim:P,robe:body==='statue'});
 if(body==='gardener')tool(s,'pick');
 else {s.orb([.35,1.03,.35],[.16,.15,.16],'#d68c8c','sway',[.25,.9,.35]);s.rod([.25,.8,.35],[.35,1.03,.35],.025,'#526f4e');s.box([-.25,1.03,.30],[.09,.44,.41],IVORY);}
}
function vessel(s,style,role='hull'){s.parts.push(...navalSculpt(style,role).parts);}
export function houseSculpt(id,flags=0,airRole='fighter'){
 if(!hasSculpt(id))return null;
 const [a,body]=id.split(':'),seat=Number(a),s=new Sculpt();
 if(flags&1){const model=airSculpt(seat,airRole);if(flags&2)model.parts.push({shape:'box',at:[-.33,.53,-.36],size:[.29,.30,.24],pivot:[-.33,.53,-.36],motion:'',color:'#b89458',on:[]});return model;}
 [kublai,arjuna,templar,atlantis,fleet,manifesto,nullSociety,nations,(s,b)=>b==='truck'?cart(s,{truck:true,culture:8}):emu(s,{forager:b==='forager',gunner:b==='gunner'}),garden][seat](s,body);
 if(flags&2)s.box([-.33,.83,-.36],[.29,.30,.24],'#b89458');
 return {parts:s.parts,sculpt:id,muzzle:s.muzzle};
}
