// Faction naval miniatures: retained hulls and rigid equipment, animated by the unit shader.
import { Sculpt } from './sculpt.js';
const P='primary',S='secondary',A='accent',WOOD='#785239',DECK='#b59a70',IRON='#788b95',DARK='#26333a',GOLD='#dab866',IVORY='#e6dcc2',RED='#b93b3c',LIGHT='#a3fff0';
export const NAVAL_ROLES=['hull','gun','sub','sonar','carrier'];
// The three fleet functions stay stable; the house SDK gives them these display names.
export const SHIP_TITLES={
 sloop:['Coastal sloop','Gun sloop','Diving boat','Signal sloop','Wing tender','Channel cutter','Sea rover'],
 ger:['Steppe ferry','Khagan warship','Pearl diver','Horse-tail beacon','Falcon roost','Strait outrider','Orkhon raider'],
 peacock:['Lotus barge','Gandiva war galley','Makara diver','Conch oracle','Vimana ark','River sentinel','Garuda raider'],
 cog:['Pilgrim cog','Crusader war cog','Reliquary diver','Vigil lantern','Gryphon tender','Templar watchboat','Turcopole galley'],
 steamer:['Merchant of Light','Salomon ironclad','Bensalem bathysphere','Acoustic laboratory','Flying-machine tender','Lamp surveyor','Experiment cutter'],
 junk:['Tribute junk','Baochuan warjunk','Dragon Turtle','Celestial observatory','Kongming kite tender','Lantern junk','Wavecutter'],
 cruiser:['Workers transport','Red Banner cruiser','Red October','Hydrophone trawler','Proletariat flattop','Convoy sentinel','Shock flotilla'],
 pointer:['Packet ferry','Segfault destroyer','Null-depth stalker','Watchpoint array','Fork carrier','Address beacon','Dangling corsair'],
 indiaman:['Kirkcaldy trader','Wealth of Nations','Invisible Hand','Sounding surveyor','Aerostat tender','Customs cutter','Free-trade privateer'],
 emu:['Wheat scow','Lewis gun launch','Diving egg','Long-neck lookout','Airborne emu roost','Wheat patrol','Campion skimmer'],
 paper:['Blank-page skiff','Thorn ark','Seedpod diver','Awakening eye','Rose garden ark','Sensation lantern','Petal clipper'],
};
export function shipTitle(style,role,variant=''){
 const i=variant==='picket'?5:variant==='raider'?6:NAVAL_ROLES.indexOf(role);
 return (SHIP_TITLES[style]||SHIP_TITLES.sloop)[Math.max(0,i)];
}
export const navalVariant=u=>u.type==='picket'?'picket':u.type==='corsair'?'raider':'';

// Polygon in the water plane, retained for decks and sail platforms.
function plate(s,points,y,depth,c){s.poly([0,y,0],points,depth,c,'',[0,y,0],[Math.PI/2,0,0]);}
function hull(s,{length=1.15,beam=.38,c=WOOD,deck=DECK,profile='round'}={}){
 const shape={
  ferry:[[-1,.12,.38],[-.72,.15,.88],[.20,.18,1],[.74,.25,.86],[1,.46,.30]],
  galley:[[-1,.19,.18],[-.67,.15,.66],[.28,.17,.78],[.91,.36,.23],[1,.52,.04]],
  cog:[[-1,.20,.30],[-.74,.15,.82],[.18,.18,1],[.70,.38,.89],[1,.73,.40]],
  bulbous:[[-1,.20,.22],[-.74,.10,.72],[.12,.12,1],[.75,.19,.78],[1,.40,.28]],
  junk:[[-1,.36,.24],[-.72,.14,.82],[.22,.16,1],[.82,.39,.84],[1,.68,.27]],
  chine:[[-1,.15,.16],[-.68,.12,.61],[.30,.15,.73],[.89,.20,.36],[1,.35,.05]],
  merchant:[[-1,.27,.29],[-.72,.13,.78],[.13,.16,1],[.69,.38,.92],[1,.72,.34]],
  punt:[[-1,.17,.48],[-.72,.15,.94],[.45,.16,1],[.93,.20,.68],[1,.31,.35]],
  frame:[[-1,.16,.18],[-.70,.10,.68],[.24,.12,.82],[.86,.20,.55],[1,.37,.10]],
  open:[[-1,.24,.20],[-.67,.14,.72],[.20,.15,.92],[.82,.27,.66],[1,.48,.18]],
 }[profile]||null;
 // The top of every elliptical cross-section meets the deck underside. `y` adds
 // sheer at the bow and stern, so the deck never reads as a floating flat plate.
 const rings=shape.map(([x,y,z])=>{const half=z*beam,top=.552+Math.max(0,y-.15);return [x*length,top-half*.62,half];});
 // A single closed loft supplies keel, bilge, flare and sheer without stacked slabs.
 s.loft(rings,c);const deckPts=rings.slice(1,-1).flatMap(([x,,z])=>[x,-z]).concat(rings.slice(1,-1).reverse().flatMap(([x,,z])=>[x,z]));
 plate(s,deckPts,.57,.035,deck);
}
function flag(s,x,y,c=P){s.rod([x,.57,0],[x,y,0],.035,WOOD);s.poly([x,y-.26,0],[0,0,.40,.05,.31,.26,0,.26],.035,c,'sway',[x,y,0]);}
function sail(s,x,{height=1.65,width=.62,c=S,lateen=false,batten=false,cross=false}={}){
 s.rod([x,.55,0],[x,height,0],.04,WOOD);
 const points=lateen?[-width*.5,0,width*.65,.03,-width*.42,.75]:[-width/2,0,width/2,.06,width*.44,.60,-width*.42,.66];
 s.poly([x,height-.78,0],points,.035,c,'sway',[x,height,0]);
 s.rod([x-width*.50,height-.08,0],[x+width*.48,height-.14,0],.03,WOOD);
 if(batten)for(const d of [.21,.40,.59])s.box([x,height-d,.027],[width*.87,.025,.025],WOOD,'sway',[x,height,0]);
 // Standing rigging and a boom give the sail an actual load-bearing structure at
 // inspection zoom, while the cloth remains the primary top-down silhouette.
 const rigging=s.parts.length;
 s.rod([x,.58,-.03],[x-width*.50,height-.12,-.03],.014,WOOD,'sway',[x,height,0]);
 s.rod([x,.58,.03],[x+width*.46,height-.12,.03],.014,WOOD,'sway',[x,height,0]);
 s.rod([x-width*.45,height-.77,.02],[x+width*.44,height-.77,.02],.024,WOOD,'sway',[x,height,0]);
 if(cross){s.box([x,height-.43,.028],[.06,.37,.025],RED,'sway',[x,height,0]);s.box([x,height-.35,.029],[.28,.06,.026],RED,'sway',[x,height,0]);}
 s.tag(rigging,{material:'wood',detail:1});
}
function oars(s,beam=.36,n=3){for(const z of [-1,1])for(let i=0;i<n;i++){const x=-.55+i*.43;s.rod([x,.44,z*beam],[x-.22,.22,z*(beam+.40)],.035,WOOD);s.box([x-.24,.21,z*(beam+.40)],[.12,.035,.19],DECK);}}
function gun(s,x,y,z=0,{brass=false,twins=false}={}){
 const start=s.parts.length,pivot=[x,y,z];s.box([x,y-.07,z],[.28,.17,.29],P);for(const side of twins?[-.07,.07]:[0])s.box([x+.27,y+.03,z+side],[.58,.07,.07],brass?GOLD:IRON,'recoil',pivot);s.tag(start,{material:brass?'brass':'iron',aim:'turret',pivot});
}
function tower(s,x,y,c=P){s.capsule([x,.55+y/2,0],[.34,y,.38],c);s.box([x,.57+y,0],[.43,.06,.47],S);s.orb([x+.03,.57+y,0],[.22,.11,.22],c);}
function lantern(s,x,y,z=0){s.rod([x,.55,z],[x,y,z],.035,WOOD);s.box([x,y-.12,z],[.19,.25,.19],GOLD);s.box([x,y-.12,z+.10],[.13,.15,.025],'#fff0ad');}
function cross(s,x,y,z,c=RED){s.box([x,y,z],[.055,.29,.025],c);s.box([x,y+.045,z],[.23,.055,.03],c);}
function landing(s,{c=DECK,beam=.47,length=1.0}={}){
 plate(s,[-length,-beam,length*.88,-beam,length,0,length*.88,beam,-length,beam],.69,.10,c);
 // Parallel launch rails, rather than a modern runway on every ancient vessel.
 for(const z of [-.15,.15])s.box([.16,.76,z],[1.25,.025,.035],IVORY);
}
function wing(s,x,y,c=IVORY,z=0){s.box([x,y,z],[.39,.08,.10],WOOD);s.box([x,y+.04,z],[.16,.045,.58],c);s.box([x-.18,y+.07,z],[.10,.035,.24],c);}
function capsule(s,{c=IRON,length=2.18,beam=.53}={}){s.orb([0,.32,0],[length,.49,beam],c);for(const z of [-1,1])s.box([-.73,.32,z*beam*.6],[.30,.05,.30],P);}
function periscope(s,x=-.13,y=1.0,c=IRON){s.rod([x,.51,0],[x,y,0],.045,c);s.rod([x,y,0],[x+.18,y,0],.045,c);}
function dish(s,x,y,c=P){s.rod([x,.56,0],[x,y,0],.04,IRON);s.part('cylinder',[x,y,0],[.45,.45,.065],c,'spin',[x,y,0],[.35,0,0]);}
function fan(s,x,y,z,c=GOLD){for(const a of [-.65,-.32,0,.32,.65])s.rod([x,y,z],[x-.10,y+.46+Math.cos(a)*.14,z+Math.sin(a)*.54],.065,c);}

function ger(s,role,v){
 if(role==='sub'){capsule(s,{c:'#65564b',beam:.66});s.part('cylinder',[-.18,.64,0],[.44,.44,.29],IVORY,'',[-.18,.64,0],[Math.PI/2,0,0]);s.orb([-.18,.80,0],[.53,.22,.48],P);periscope(s,-.13,1.07,GOLD);for(const z of [-.35,.35])s.box([.58,.37,z],[.50,.10,.13],WOOD);return;}
 hull(s,{beam:v==='raider'?.27:.39,c:'#65503a',profile:'ferry'});
 if(role==='carrier'){landing(s);for(const z of [-.36,.36]){s.rod([-.64,.75,z],[-.64,1.25,z],.04,WOOD);s.box([-.57,1.25,z],[.38,.05,.09],WOOD);s.orb([-.57,1.34,z],[.23,.21,.16],'#725338');}flag(s,-.85,1.6);return;}
 if(v==='raider'){s.box([-.65,.65,0],[.44,.17,.37],P);s.rod([1,.50,0],[1.40,.65,0],.06,WOOD);}
 else{s.part('cylinder',[-.63,.74,0],[.48,.48,.32],IVORY,'',[-.63,.74,0],[Math.PI/2,0,0]);s.orb([-.63,.94,0],[.60,.28,.56],P);}
 sail(s,.18,{height:v==='picket'?1.88:v==='raider'?1.40:1.60,width:v==='raider'?.98:.76,c:'#b6a27b',batten:v!=='raider',lateen:v==='raider'});
 oars(s,v==='raider'?.27:.39,v==='raider'?2:3);
 if(role==='gun'){s.box([.65,.68,0],[.55,.15,.24],WOOD);s.rod([.62,.78,-.31],[.62,.78,.31],.07,IRON);s.box([.89,.81,0],[.55,.045,.04],GOLD);if(!v){s.box([.50,.70,0],[.62,.16,.55],P);sail(s,-.30,{height:1.95,width:.58,c:IVORY,batten:true});}}
 if(role==='sonar'||v==='picket'){flag(s,-.32,1.92);for(const z of [-.08,.08])s.rod([-.22,1.62,z],[-.19,1.22,z],.065,IVORY);}
}
function peacock(s,role,v){
 if(role==='sub'){capsule(s,{c:'#247d83',beam:.68});fan(s,-.47,.38,0,P);s.rod([.72,.47,0],[1.00,.80,0],.15,GOLD);s.orb([1.03,.84,0],[.29,.25,.27],GOLD);s.box([1.17,.81,0],[.22,.07,.14],IVORY);periscope(s,0,1.03,GOLD);return;}
 hull(s,{beam:v==='raider'?.28:.43,c:'#255c70',deck:'#c19b62',profile:'galley'});
 s.rod([.85,.52,0],[1.03,1.08,0],.11,GOLD);s.orb([1.07,1.12,0],[.25,.22,.24],P);s.box([1.20,1.10,0],[.18,.05,.11],GOLD);fan(s,-.82,.62,0);
 if(role==='carrier'){landing(s,{c:'#316b80'});for(const z of [-1,1])s.poly([-.13,.78,z*.31],[-.40,0,.45,0,.13,.37,-.33,.26],.035,GOLD,'',[0,.78,z*.31],[Math.PI/2,0,0]);wing(s,.05,.85,GOLD);flag(s,-.73,1.63);return;}
 oars(s,v==='raider'?.28:.43,3);sail(s,-.18,{height:v==='raider'?1.42:1.65,width:v==='raider'?.98:.69,c:v==='raider'?P:'#e0b65f',lateen:true});
 if(v!=='raider'){s.box([-.61,.94,0],[.52,.10,.57],GOLD);for(const z of [-.20,.20])s.rod([-.61,.60,z],[-.61,.94,z],.04,GOLD);}
 if(role==='gun'){s.rod([.50,.77,-.32],[.50,.77,.32],.06,GOLD);s.rod([.26,.83,0],[.91,.83,0],.055,IVORY);s.box([.32,.68,0],[.20,.29,.22],P);}
 if(role==='sonar'||v==='picket'){s.orb([.27,1.04,0],[.55,.35,.37],IVORY,'spin',[.27,1.04,0]);s.orb([.51,1.05,0],[.14,.18,.20],GOLD);}
}
function cog(s,role,v){
 if(role==='sub'){capsule(s,{c:'#88715a',beam:.64});s.box([-.12,.66,0],[.52,.39,.37],IVORY);s.poly([-.12,.88,0],[-.35,0,.35,0,0,.28],.45,GOLD);cross(s,-.12,.68,.20);periscope(s,.16,1.17,GOLD);return;}
 hull(s,{beam:v==='raider'?.27:.44,c:'#62402f',profile:'cog'});
 for(const x of v?[-.81]:[-.81,.72]){tower(s,x,v==='raider'?.24:.40,IVORY);for(const z of [-.18,0,.18])s.box([x,v==='raider'?.83:.99,z],[.41,.13,.09],IVORY);cross(s,x,.79,.20);}
 if(role==='carrier'){landing(s,{c:'#a88c6a'});s.box([-.48,.96,0],[.38,.40,.37],IVORY);s.poly([-.48,1.16,0],[-.27,0,.27,0,0,.28],.47,RED);wing(s,.42,.85,IVORY);cross(s,-.48,.98,.21);return;}
 sail(s,-.12,{height:v==='picket'?1.94:v==='raider'?1.53:1.78,width:v==='raider'?.94:.83,c:IVORY,cross:true,lateen:v==='raider'});
 if(role==='gun'){s.box([.48,1.04,0],[.52,.10,.13],WOOD);s.rod([.55,1.12,-.34],[.55,1.12,.34],.06,DARK);s.box([.80,1.15,0],[.57,.035,.035],GOLD);}
 if(role==='sonar'||v==='picket'){tower(s,.66,.61,IVORY);lantern(s,.66,1.70);}
}
function steamer(s,role,v){
 if(role==='sub'){capsule(s,{c:'#a58b50',length:1.94,beam:.86});s.part('cylinder',[.10,.65,0],[.62,.62,.28],GOLD,'',[.10,.65,0],[Math.PI/2,0,0]);s.orb([.10,.83,0],[.47,.30,.47],'#93cbd3');for(const z of [-.39,.39])s.orb([.45,.34,z],[.16,.18,.04],LIGHT);periscope(s,-.32,1.14,GOLD);return;}
 hull(s,{beam:v==='raider'?.29:.42,c:'#50636b',deck:'#bba16c',profile:'bulbous'});
 for(const z of [-.47,.47]){if(v==='raider'){s.box([-.54,.39,z],[.56,.13,.15],GOLD);s.box([-.85,.39,z],[.045,.34,.28],DARK);}else{s.wheel([-.26,.37,z],.28,GOLD);s.box([-.26,.61,z],[.69,.09,.20],P);}}
 tower(s,-.63,.34,WOOD);s.part('cylinder',[.02,.78,0],[.39,.39,.60],GOLD,'',[.02,.78,0],[0,Math.PI/2,0]);s.rod([-.02,.78,0],[-.02,1.43,0],.14,DARK);s.box([-.02,1.43,0],[.23,.08,.23],GOLD);
 if(role==='carrier'){landing(s,{c:'#887653'});wing(s,.36,.87,IVORY);s.rod([-.67,.91,0],[-.67,1.42,0],.05,GOLD);s.box([-.45,1.42,0],[.52,.04,.07],GOLD);return;}
 if(role==='gun')gun(s,.55,.73,0,{brass:true});
 if(role==='sonar'||v==='picket'){s.part('cylinder',[.49,.94,0],[.55,.55,.09],GOLD,'spin',[.49,.94,0]);s.part('cylinder',[.49,.94,.06],[.39,.39,.02],DARK,'spin',[.49,.94,0]);s.box([.49,.69,0],[.22,.24,.25],P);}
 if(v==='raider')s.box([.61,.72,0],[.54,.15,.25],P);flag(s,-.80,1.57);
}
function junk(s,role,v){
 if(role==='sub'){s.orb([0,.35,0],[1.83,.64,1.02],'#457a62');plate(s,[-.80,-.40,.58,-.40,.91,0,.58,.40,-.80,.40],.55,.12,P);for(const x of [-.45,0,.45])s.box([x,.65,0],[.035,.06,.70],GOLD);for(const z of [-.42,.42])s.box([0,.29,z],[1.25,.07,.38],DARK);s.rod([.67,.44,0],[.95,.75,0],.11,GOLD);s.orb([1,.75,0],[.25,.23,.28],GOLD);periscope(s,-.25,1.03);return;}
 hull(s,{length:1.18,beam:v==='raider'?.29:.47,c:'#6a3d2d',deck:'#c7a56c',profile:'junk'});
 tower(s,-.80,.34,'#934b3c');s.box([-.80,.97,0],[.54,.08,.57],DARK);for(const z of [-.27,.27])s.box([-.80,1.01,z],[.61,.05,.08],GOLD);
 if(role==='carrier'){landing(s,{c:'#8c4d3b',beam:.51});for(const z of [-.28,.28]){wing(s,.14,.86,'#d8b46d',z);s.rod([-.37,.76,z],[.30,1.28,z],.035,WOOD);}sail(s,-.77,{height:1.73,width:.49,batten:true,c:'#d9b77c'});return;}
 const masts=v==='raider'?[.05]:v==='picket'?[-.42]:role==='gun'?[-.50,.10,.62]:[-.40,.34];for(const x of masts)sail(s,x,{height:v==='raider'?1.48:x<0?1.90:x>.5?1.40:1.75,width:v==='raider'?.89:x<0?.70:.56,c:v==='raider'?P:'#d9b77c',batten:true});
 for(const z of [-.48,.48])s.orb([.83,.55,z*.72],[.11,.12,.035],IVORY);
 if(role==='gun')for(const z of [-.35,.35]){s.box([.44,.71,z],[.40,.20,.17],RED);for(const x of [.30,.46,.62])s.rod([x,.77,z],[x+.22,1.01,z],.045,IRON);}
 if(role==='sonar'||v==='picket'){tower(s,.54,.48,'#934b3c');lantern(s,.54,1.61);s.part('cylinder',[.54,1.23,0],[.45,.45,.055],GOLD,'spin',[.54,1.23,0]);}
}
function cruiser(s,role,v){
 if(role==='sub'){capsule(s,{c:'#485954',length:2.40,beam:.50});s.box([-.17,.61,0],[.51,.38,.27],DARK);periscope(s,-.20,1.07);periscope(s,.02,.94);s.box([-.16,.68,.15],[.22,.14,.025],RED);gun(s,.45,.49);return;}
 hull(s,{length:1.28,beam:v==='raider'?.26:.37,c:'#54625a',deck:'#869084',profile:'chine'});
 if(role==='carrier'){landing(s,{c:'#5b6868',beam:.53,length:1.23});s.box([-.42,.94,.40],[.58,.42,.20],IRON);s.box([-.44,1.26,.40],[.13,.30,.14],DARK);for(const x of [-.62,.40])wing(s,x,.83,'#aebba1',-.18);s.box([.15,.76,0],[1.7,.025,.04],IVORY);return;}
 tower(s,-.13,v==='raider'?.23:.42,IRON);for(const x of v?[-.48]:[-.63,.22]){s.box([x,v==='raider'?.79:.94,0],[.16,v==='raider'?.34:.64,.18],DARK);s.box([x,v==='raider'?.93:1.18,0],[.18,.14,.20],RED);}flag(s,-.37,v==='raider'?1.20:1.55,RED);
 if(role==='gun'){gun(s,.65,.68,0,{twins:!v});if(!v)gun(s,-.87,.65,0,{twins:true});if(v==='raider')for(const z of [-.24,.24])s.box([-.2,.65,z],[.78,.12,.12],IRON);}
 if(role==='sonar'||v==='picket'){dish(s,.52,1.24);for(const z of [-.36,.36])s.orb([.69,.33,z],[.18,.28,.18],DARK);}
}
function pointer(s,role,v){
 if(role==='sub'){for(const z of [-.29,.29]){s.orb([0,.31,z],[2.05,.33,.25],DARK);s.box([-.14,.49,z],[1.25,.035,.035],LIGHT);}s.box([-.12,.44,0],[.46,.12,.67],IRON);s.poly([-.13,.50,0],[-.34,0,.34,0,.16,.42,-.18,.30],.20,DARK);s.box([.05,.80,.12],[.19,.06,.025],LIGHT);return;}
 const beam=v==='raider'?.26:.37;
 hull(s,{length:1.12,beam,c:DARK,deck:IRON,profile:'frame'});
 for(const z of [-beam,beam])s.rod([-.68,.38,z],[.78,.56,z],.028,P);
 if(role==='carrier'){landing(s,{c:DARK,beam:.54,length:1.13});for(const z of [-.46,.46]){s.box([-.55,1.05,z],[.18,.70,.15],DARK);s.box([-.55,1.05,z-Math.sign(z)*.09],[.07,.49,.025],LIGHT);}s.box([-.55,1.39,0],[.18,.13,1.05],P);for(const z of [-.22,.22])wing(s,.32,.83,LIGHT,z);return;}
 s.poly([-.19,.55,0],[-.51,0,.45,0,.13,.42,-.32,.27],.40,DARK);s.box([.01,.81,.21],[.27,.06,.025],LIGHT);
 if(role==='gun')for(const z of v==='raider'?[0]:[-.22,.22]){s.box([.63,.71,z],[v==='raider'?1.02:.68,.12,.12],P);s.box([v==='raider'?1.16:.99,.71,z],[.04,.055,.065],LIGHT);}
 if(v==='raider')for(const z of [-.36,.36])plate(s,[-.93,z-.13,-.17,z-.13,-.72,z+.22],.57,.04,P);
 if(role==='sonar'||v==='picket'){for(const z of [-.26,.26]){s.rod([-.50,.55,z],[-.50,1.23,z],.04,IRON);s.orb([-.50,1.23,z],[.23,.23,.23],LIGHT,'spin',[-.50,1.23,z]);}s.box([-.50,1.23,0],[.08,.06,.50],P);}
}
function indiaman(s,role,v){
 if(role==='sub'){capsule(s,{c:'#6c4b35',beam:.65});s.box([-.21,.59,0],[.61,.21,.37],DECK);s.orb([-.21,.78,0],[.41,.35,.39],GOLD);s.orb([.67,.41,0],[.32,.32,.56],GOLD);periscope(s,-.20,1.08,GOLD);for(const z of [-.36,.36])s.box([-.73,.36,z],[.43,.06,.23],WOOD);return;}
 hull(s,{length:1.19,beam:v==='raider'?.30:.44,c:'#533b2f',profile:'merchant'});tower(s,-.83,.38,'#79503a');s.box([-.99,.84,0],[.035,.18,.42],GOLD);
 for(const z of [-.44,.44]){s.box([-.10,.57,z],[1.65,.075,.035],GOLD);for(const x of [-.50,0,.50])s.box([x,.44,z],[.14,.10,.035],DARK);}
 if(role==='carrier'){landing(s,{c:DECK});s.orb([-.30,1.30,0],[1.20,.48,.49],IVORY,'bob');for(const z of [-.20,.20])s.rod([-.30,.76,z],[-.30,1.1,z],.025,WOOD);wing(s,.47,.84,S);flag(s,-.93,1.50);return;}
 for(const x of v?[.08]:[-.48,.13,.66])sail(s,x,{height:v==='raider'?1.49:v==='picket'?1.76:x===.13?1.94:1.65,width:v?.94:.56,c:v==='raider'?'#c0b18b':IVORY,lateen:!!v});
 if(v==='raider')s.rod([.92,.53,0],[1.52,.72,0],.045,WOOD);
 if(role==='gun')for(const z of [-.37,.37])gun(s,.47,.69,z,{brass:true});
 if(role==='sonar'||v==='picket'){s.rod([.54,.58,.34],[.54,1.15,.34],.04,GOLD);s.box([.54,1.18,.34],[.61,.09,.10],GOLD);s.orb([.86,1.18,.34],[.08,.17,.17],DARK);}
}
function emu(s,role,v){
 if(role==='sub'){s.orb([-.08,.39,0],[1.45,.83,.77],'#d1c6a4');s.box([.35,.68,0],[.26,.18,.27],DARK);periscope(s,.36,1.16,WOOD);for(const z of [-.48,.48])s.poly([-.20,.28,z],[-.45,0,.27,0,-.20,.28],.055,'#85775c');s.box([1,.31,0],[.32,.06,.16],GOLD);return;}
 // A captured shallow-draught gun punt: tall bird lookout and broad exposed machinery.
 hull(s,{length:v==='raider'?1.24:1.03,beam:v==='raider'?.25:.49,c:'#7e8060',profile:'punt'});s.box([-.61,.71,0],[.46,.32,.52],WOOD);if(v!=='raider')s.box([-.61,.90,0],[.59,.07,.61],P);else{s.box([-.85,.86,0],[.14,.12,.21],DARK);for(const z of [-.49,.49])s.box([-.35,.31,z],[1.35,.16,.13],WOOD);}
 const neck=role==='sonar'||v==='picket'?1.46:1.08;s.rod([.70,.54,0],[.79,neck,0],.13,'#8a7d61');s.orb([.83,neck+.13,0],[.28,.28,.23],DARK);s.box([1.02,neck+.10,0],[.23,.065,.13],GOLD);
 for(const z of [-.54,.54])s.box([-.15,.31,z],[.59,.28,.12],DARK);
 if(role==='carrier'){landing(s,{c:'#8c8060',beam:.55});s.orb([-.21,.83,0],[.71,.24,.63],WOOD);s.orb([-.21,.95,0],[.30,.42,.29],IVORY);for(const z of [-.30,.30])wing(s,.35,.87,'#ad9a72',z);return;}
 if(role==='gun'){s.rod([.18,.60,-.25],[.18,.98,-.25],.055,IRON);s.box([.50,1.01,-.25],[.86,.07,.09],DARK,'recoil');s.part('cylinder',[.28,1.09,-.25],[.28,.28,.045],DARK,'recoil',[.28,1.09,-.25],[Math.PI/2,0,0]);}
 if(role==='sonar'||v==='picket')for(const z of [-.09,.09])s.box([1.01,neck+.20,z],[.25,.09,.08],IRON);else flag(s,-.80,1.33);
}
function paper(s,role,v){
 if(role==='sub'){s.orb([0,.33,0],[1.62,.51,.79],'#71916e');for(const z of [-.19,0,.19])s.poly([-.10,.47,z],[-.74,0,.62,0,.18,.45,-.35,.35],.035,z===0?IVORY:'#a8bf8b');s.rod([.29,.56,0],[.29,1.05,0],.045,WOOD);s.orb([.29,1.05,0],[.30,.27,.29],'#d3a7a4');return;}
 hull(s,{length:1.10,beam:.40,c:'#b0b69a',deck:'#819b76',profile:'open'});
 for(const z of [-1,1])s.poly([0,.41,z*.18],[-1.05,0,1.07,0,.55,.39,-.65,.28],.045,z<0?IVORY:'#bdc8ab','',[0,.41,z*.18],[z*.65,0,0]);
 if(role==='carrier'){plate(s,[-.94,-.54,.81,-.42,1.02,0,.81,.42,-.94,.54],.59,.07,'#819b76');for(const z of [-.27,.27]){s.rod([-.24,.64,z],[-.24,1.06,z],.035,WOOD);for(const a of [0,2.1,4.2])s.orb([-.24+Math.cos(a)*.13,1.08,z+Math.sin(a)*.13],[.30,.20,.27],'#d5a0a2');}wing(s,.53,.72,IVORY);return;}
 if(v!=='picket'){s.poly([-.11,.56,0],[-.56,0,.65,0,-.13,.90],.035,IVORY,'sway',[-.11,1.46,0]);s.rod([-.24,.55,0],[-.24,1.53,0],.025,WOOD);}
 else{s.rod([-.12,.53,0],[-.12,1.47,0],.06,WOOD);s.orb([-.12,1.49,0],[.40,.35,.35],IVORY);s.orb([.07,1.49,0],[.07,.23,.23],P);}
 if(role==='gun'){for(const z of [-.23,.23]){s.rod([.42,.54,z],[.74,1.04,z],.065,'#587450');s.poly([.74,1.04,z],[-.14,-.06,.16,0,-.06,.25],.06,IVORY,'strike',[.42,.54,z]);}}
 if(role==='sonar'||v==='picket'){s.orb([.47,1.02,0],[.50,.33,.39],IVORY,'spin',[.47,1.02,0]);s.orb([.69,1.03,.04],[.06,.22,.22],P,'spin',[.47,1.02,0]);}
 if(v==='raider')for(const z of [-.45,.45])s.poly([-.51,.46,z],[-.37,0,.35,0,0,.39],.025,'#c1d09d');
}
function sloop(s,role,v){
 if(role==='sub'){capsule(s);s.box([-.18,.59,0],[.43,.30,.27],P);periscope(s);return;}
 hull(s,{beam:v==='raider'?.27:.36,profile:'open'});
 if(role==='carrier'){landing(s);wing(s,.28,.83);tower(s,-.67,.35);return;}
 sail(s,-.07,{height:1.70,width:.77,lateen:true});flag(s,-.78,1.2);
 if(role==='gun')gun(s,.51,.71);if(role==='sonar'||v==='picket')dish(s,.47,1.15);
}
const BUILDERS={sloop,ger,peacock,cog,steamer,junk,cruiser,pointer,indiaman,emu,paper};
// House-specific high silhouette. These are kept above the waterline so each hull
// stays recognizable in a monochrome fleet while preserving its role equipment.
function houseRig(s,style){
 if(style==='ger'){s.part('cylinder',[-.54,.82,0],[.52,.52,.28],IVORY,'',[-.54,.82,0],[Math.PI/2,0,0]);s.orb([-.54,1.02,0],[.58,.24,.52],P);}
 else if(style==='peacock'){fan(s,-.72,.78,0,GOLD);s.orb([.72,.82,0],[.30,.27,.28],P);}
 else if(style==='cog'){s.poly([-.56,1.08,0],[-.33,0,.33,0,0,.42],.36,RED);cross(s,-.56,.99,.19);}
 else if(style==='steamer'){s.part('cylinder',[-.22,1.00,0],[.36,.36,.16],GOLD,'spin',[-.22,1.00,0],[Math.PI/2,0,0]);s.orb([-.22,1.00,.10],[.10,.20,.12],'#9cccd3');}
 else if(style==='junk'){for(const z of [-.26,.26])s.box([-.72,1.20,z],[.58,.04,.055],GOLD);s.poly([-.72,1.23,0],[-.34,0,.34,0,0,.25],.42,'#934b3c');}
 else if(style==='cruiser'){for(const z of [-.15,.15])s.box([-.25,1.12,z],[.18,.54,.04],IRON);s.box([-.25,1.40,0],[.48,.06,.43],RED);}
 else if(style==='pointer'){for(const z of [-.31,.31])s.rod([-.40,.57,z],[-.40,1.32,z],.035,IRON);s.orb([-.40,1.32,0],[.46,.12,.46],LIGHT);}
 else if(style==='indiaman'){s.part('cylinder',[-.62,1.02,0],[.34,.34,.045],GOLD,'',[-.62,1.02,0],[Math.PI/2,0,0]);s.rod([-.62,.66,0],[-.62,1.39,0],.035,GOLD);}
 // Emu hulls already carry their long-neck lookout as their main superstructure.
 // Do not add a second head merely to satisfy a house signature.
 else if(style==='emu'){}
 else if(style==='paper'){for(const z of [-.28,0,.28])s.orb([-.30,.86,z],[.32,.08,.29],z?'#d3a7a4':IVORY,'sway',[-.30,.63,0]);}
}
export function navalSculpt(style,role='hull',riders=0,variant=''){
 const s=new Sculpt();(BUILDERS[style]||sloop)(s,NAVAL_ROLES.includes(role)?role:'hull',variant);
 houseRig(s,style);
 // Only carriers display parked aircraft. Passenger count stays out of all other cache variants.
 if(role==='carrier')for(let i=0;i<Math.min(4,Math.max(0,riders));i++)wing(s,-.65+i*.40,.88,P,i%2?.25:-.25);
 return {parts:s.parts,rig:'ship'};
}
