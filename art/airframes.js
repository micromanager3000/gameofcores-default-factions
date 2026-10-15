// Aircraft metadata shared by the viewer labels, presets and development gallery.
export const AIR_ROLES=['scout','fighter','striker'];
export const AIR_TITLES=[
 ['Steppe falcon','Sky keshig','Thunderdrum airship'],
 ['Garuda eye','Gandiva vimana','Indra thunderchariot'],
 ['Pilgrim gryphon','Crusader gryphon','Sanctum sky chapel'],
 ['Lens glider','Salomon ornithopter','Bensalem sky laboratory'],
 ['Kongming eye','Swallow fire kite','Heaven-sailing junk'],
 ['Red Star spotter','Red Banner interceptor','Hammer bomber'],
 ['Watchpoint mote','Null interceptor','Fork bomber'],
 ['Survey balloon','Invisible Hand glider','Company sky freighter'],
 ['Airborne emu','Lewis gyro-emu','Wheat-front cropduster'],
 ['Dandelion eye','Petal glider','Rose garden ark'],
];
// hp, speed, strength, carry, vision, range, wings, stealth. Each costs ten build points.
export const AIR_BUILDS={scout:[1,3,0,0,3,0,1,0],fighter:[2,3,2,0,0,0,1,0],striker:[2,1,2,0,0,1,1,0]};
export const airRole=build=>build?.[5]>0?'striker':build?.[4]>=2?'scout':'fighter';
export function airDefinition(seat,role){const i=AIR_ROLES.indexOf(role);return AIR_TITLES[seat]&&i>=0?{name:AIR_TITLES[seat][i],role,build:AIR_BUILDS[role].slice(),requires:role==='striker'?['flight','ranged','airWeapons']:['flight']}:null;}

